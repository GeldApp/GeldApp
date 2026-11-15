use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};
use zeroize::Zeroizing;

const MAX_LEDGER: usize = 30_000_000;
const KDF_MEMORY_KIB: u32 = 65_536;
const KDF_ITERATIONS: u32 = 3;
const VAULT_FORMAT: &str = "geldapp-vault";
const KEY_AAD: &[u8] = b"geldapp-vault-key-v1";
const DATA_AAD: &[u8] = b"geldapp-ledger-v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultFile {
    format: String,
    version: u32,
    kdf: String,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    salt: String,
    key_nonce: String,
    wrapped_key: String,
    data_nonce: String,
    ciphertext: String,
    #[serde(default)]
    pub biometric_enabled: bool,
}

#[derive(Serialize)]
pub struct Status {
    pub vault_exists: bool,
    pub unlocked: bool,
    pub legacy_files: Vec<String>,
    pub biometric_enabled: bool,
    pub biometric_available: bool,
    pub biometric_supported: bool,
}

pub struct VaultSession {
    operation: Mutex<()>,
    key: Mutex<Option<Zeroizing<[u8; 32]>>>,
}

impl VaultSession {
    pub fn new() -> Self {
        Self {
            operation: Mutex::new(()),
            key: Mutex::new(None),
        }
    }

    pub fn status(&self, directory: &Path) -> Result<Status, String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        let envelope = read_vault(directory).ok();
        let key = self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())?;
        let biometric_supported = cfg!(target_os = "macos");
        Ok(Status {
            vault_exists: vault_exists(directory),
            unlocked: key.is_some(),
            legacy_files: legacy_files(directory),
            biometric_enabled: envelope.as_ref().is_some_and(|v| v.biometric_enabled),
            biometric_available: platform_biometry_available(),
            biometric_supported,
        })
    }

    pub fn setup(&self, directory: &Path, passphrase: &str, data: &str) -> Result<(), String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        if vault_exists(directory) {
            return Err("An encrypted vault already exists. It was left unchanged.".into());
        }
        validate_data(data)?;
        let key = random_key();
        let envelope = create_envelope(passphrase, &key, data.as_bytes(), false)?;
        write_vault(directory, &envelope)?;
        *self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())? = Some(key);
        Ok(())
    }

    pub fn migrate(&self, directory: &Path, passphrase: &str, source: &str) -> Result<(), String> {
        if vault_exists(directory) {
            return Err("An encrypted vault already exists. It was left unchanged.".into());
        }
        let allowed = ["ledger.json", "ledger.previous.json", "ledger.next.json"];
        if !allowed.contains(&source) {
            return Err("Choose a listed legacy ledger file.".into());
        }
        let data = fs::read_to_string(directory.join(source)).map_err(|_| {
            "Could not read the selected legacy ledger. It was left unchanged.".to_string()
        })?;
        validate_data(&data)?;
        self.setup(directory, passphrase, &data)
    }

    pub fn unlock(&self, directory: &Path, passphrase: &str) -> Result<(), String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        let envelope = read_vault(directory)?;
        let key = unwrap_key(passphrase, &envelope)
            .map_err(|_| "Could not unlock vault. Check your passphrase.".to_string())?;
        let data = decrypt_data(&key, &envelope).map_err(|_| {
            "Vault authentication failed. The encrypted data was not changed.".to_string()
        })?;
        validate_data(
            std::str::from_utf8(&data).map_err(|_| "Vault data is invalid".to_string())?,
        )?;
        *self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())? = Some(key);
        Ok(())
    }

    pub fn unlock_biometric(&self, directory: &Path) -> Result<(), String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        let envelope = read_vault(directory)?;
        if !envelope.biometric_enabled {
            return Err("Biometric unlocking is not enabled for this vault.".into());
        }
        let key = platform_load_biometric_key()?;
        let data = decrypt_data(&key, &envelope).map_err(|_| {
            "Biometric key does not match this vault. Use your passphrase.".to_string()
        })?;
        validate_data(
            std::str::from_utf8(&data).map_err(|_| "Vault data is invalid".to_string())?,
        )?;
        *self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())? = Some(key);
        Ok(())
    }

    pub fn load(&self, directory: &Path) -> Result<Option<String>, String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        let key = self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())?;
        let Some(key) = key.as_ref() else {
            return Err("Vault is locked.".into());
        };
        if !vault_exists(directory) {
            return Ok(None);
        }
        let envelope = read_vault(directory)?;
        let data = decrypt_data(key, &envelope).map_err(|_| {
            "Vault authentication failed. The encrypted data was not changed.".to_string()
        })?;
        String::from_utf8(data.to_vec())
            .map(Some)
            .map_err(|_| "Vault data is invalid".into())
    }

    pub fn save(&self, directory: &Path, data: &str) -> Result<(), String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        validate_data(data)?;
        let key = self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())?;
        let Some(key) = key.as_ref() else {
            return Err("Vault is locked.".into());
        };
        let mut envelope = read_vault(directory)?;
        encrypt_data(key, data.as_bytes(), &mut envelope)?;
        write_vault(directory, &envelope)
    }

    pub fn set_biometric(&self, directory: &Path, enabled: bool) -> Result<(), String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        let key = self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())?;
        let Some(key) = key.as_ref() else {
            return Err("Unlock the vault first.".into());
        };
        let mut envelope = read_vault(directory)?;
        if enabled {
            platform_store_biometric_key(key)?;
            envelope.biometric_enabled = true;
            if let Err(error) = write_vault(directory, &envelope) {
                let _ = platform_delete_biometric_key();
                return Err(error);
            }
        } else {
            platform_delete_biometric_key()?;
            envelope.biometric_enabled = false;
            write_vault(directory, &envelope)?;
        }
        Ok(())
    }

    pub fn lock(&self) -> Result<(), String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        *self
            .key
            .lock()
            .map_err(|_| "Vault session is unavailable".to_string())? = None;
        Ok(())
    }
}

fn validate_data(data: &str) -> Result<(), String> {
    if data.len() > MAX_LEDGER {
        return Err("Ledger exceeds 30 MB".into());
    }
    serde_json::from_str::<Value>(data)
        .map(|_| ())
        .map_err(|_| "Ledger data is not valid JSON".into())
}

fn random_key() -> Zeroizing<[u8; 32]> {
    let mut key = Zeroizing::new([0_u8; 32]);
    OsRng.fill_bytes(key.as_mut());
    key
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, String> {
    if passphrase.chars().count() < 12 {
        return Err("Use a passphrase with at least 12 characters.".into());
    }
    let params = Params::new(KDF_MEMORY_KIB, KDF_ITERATIONS, 1, Some(32))
        .map_err(|_| "Could not initialize vault encryption".to_string())?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; 32]);
    argon
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|_| "Could not derive vault key".to_string())?;
    Ok(key)
}

fn encrypt(key: &[u8; 32], nonce: &[u8; 12], data: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    Aes256Gcm::new_from_slice(key)
        .map_err(|_| "Invalid encryption key".to_string())?
        .encrypt(Nonce::from_slice(nonce), Payload { msg: data, aad })
        .map_err(|_| "Could not encrypt vault data".to_string())
}

fn decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    data: &[u8],
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, String> {
    let plaintext = Aes256Gcm::new_from_slice(key)
        .map_err(|_| "Invalid encryption key".to_string())?
        .decrypt(Nonce::from_slice(nonce), Payload { msg: data, aad })
        .map_err(|_| "Encrypted data authentication failed".to_string())?;
    Ok(Zeroizing::new(plaintext))
}

fn create_envelope(
    passphrase: &str,
    key: &[u8; 32],
    data: &[u8],
    biometric_enabled: bool,
) -> Result<VaultFile, String> {
    let mut salt = [0_u8; 16];
    let mut key_nonce = [0_u8; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut key_nonce);
    let wrapping_key = derive_key(passphrase, &salt)?;
    let wrapped_key = encrypt(&wrapping_key, &key_nonce, key, KEY_AAD)?;
    let mut envelope = VaultFile {
        format: VAULT_FORMAT.into(),
        version: 1,
        kdf: "argon2id".into(),
        memory_kib: KDF_MEMORY_KIB,
        iterations: KDF_ITERATIONS,
        parallelism: 1,
        salt: B64.encode(salt),
        key_nonce: B64.encode(key_nonce),
        wrapped_key: B64.encode(wrapped_key),
        data_nonce: String::new(),
        ciphertext: String::new(),
        biometric_enabled,
    };
    encrypt_data(key, data, &mut envelope)?;
    Ok(envelope)
}

fn unwrap_key(passphrase: &str, envelope: &VaultFile) -> Result<Zeroizing<[u8; 32]>, String> {
    check_envelope(envelope)?;
    let salt = decode_exact::<16>(&envelope.salt)?;
    let nonce = decode_exact::<12>(&envelope.key_nonce)?;
    let ciphertext = B64
        .decode(&envelope.wrapped_key)
        .map_err(|_| "Vault header is invalid".to_string())?;
    let wrapping_key = derive_key(passphrase, &salt)?;
    let decoded = decrypt(&wrapping_key, &nonce, &ciphertext, KEY_AAD)?;
    let mut key = Zeroizing::new([0_u8; 32]);
    if decoded.len() != 32 {
        return Err("Vault key is invalid".into());
    }
    key.copy_from_slice(&decoded);
    Ok(key)
}

fn encrypt_data(key: &[u8; 32], data: &[u8], envelope: &mut VaultFile) -> Result<(), String> {
    let mut nonce = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = encrypt(key, &nonce, data, DATA_AAD)?;
    envelope.data_nonce = B64.encode(nonce);
    envelope.ciphertext = B64.encode(ciphertext);
    Ok(())
}

fn decrypt_data(key: &[u8; 32], envelope: &VaultFile) -> Result<Zeroizing<Vec<u8>>, String> {
    check_envelope(envelope)?;
    let nonce = decode_exact::<12>(&envelope.data_nonce)?;
    let ciphertext = B64
        .decode(&envelope.ciphertext)
        .map_err(|_| "Vault data is invalid".to_string())?;
    decrypt(key, &nonce, &ciphertext, DATA_AAD)
}

fn check_envelope(envelope: &VaultFile) -> Result<(), String> {
    if envelope.format != VAULT_FORMAT
        || envelope.version != 1
        || envelope.kdf != "argon2id"
        || envelope.memory_kib != KDF_MEMORY_KIB
        || envelope.iterations != KDF_ITERATIONS
        || envelope.parallelism != 1
    {
        return Err("Unsupported or invalid vault format. The file was left unchanged.".into());
    }
    Ok(())
}

fn decode_exact<const N: usize>(text: &str) -> Result<[u8; N], String> {
    let decoded = B64
        .decode(text)
        .map_err(|_| "Vault header is invalid".to_string())?;
    decoded
        .try_into()
        .map_err(|_| "Vault header is invalid".to_string())
}

fn vault_path(directory: &Path) -> PathBuf {
    directory.join("vault.enc")
}
fn previous_path(directory: &Path) -> PathBuf {
    directory.join("vault.previous.enc")
}
fn vault_exists(directory: &Path) -> bool {
    vault_path(directory).exists() || previous_path(directory).exists()
}

fn read_vault(directory: &Path) -> Result<VaultFile, String> {
    for path in [vault_path(directory), previous_path(directory)] {
        match fs::read_to_string(path) {
            Ok(raw) => {
                return serde_json::from_str(&raw).map_err(|_| {
                    "Vault file is invalid. Existing files were left unchanged.".into()
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => {
                return Err("Could not read vault file. Existing files were left unchanged.".into())
            }
        }
    }
    Err("No encrypted vault exists.".into())
}

fn write_vault(directory: &Path, envelope: &VaultFile) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|_| "Could not prepare vault storage".to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
            .map_err(|_| "Could not secure vault directory".to_string())?;
    }
    let bytes = serde_json::to_vec(envelope).map_err(|_| "Could not encode vault".to_string())?;
    let next = directory.join("vault.next.enc");
    let current = vault_path(directory);
    let previous = previous_path(directory);
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&next)
        .map_err(|_| "Could not write encrypted vault".to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Could not save encrypted vault".to_string())?;
    drop(file);
    if current.exists() {
        fs::rename(&current, &previous)
            .map_err(|_| "Could not preserve previous encrypted vault".to_string())?;
    }
    fs::rename(&next, &current).map_err(|_| {
        "Could not finalize encrypted vault. The prior encrypted copy was preserved.".to_string()
    })?;
    if let Ok(dir) = fs::File::open(directory) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn legacy_files(directory: &Path) -> Vec<String> {
    ["ledger.json", "ledger.previous.json", "ledger.next.json"]
        .into_iter()
        .filter(|name| directory.join(name).is_file())
        .map(str::to_string)
        .collect()
}

#[cfg(target_os = "macos")]
mod keychain {
    use super::Zeroizing;
    unsafe extern "C" {
        fn geld_biometry_available() -> i32;
        fn geld_store_biometric_key(bytes: *const u8, count: i32) -> i32;
        fn geld_load_biometric_key(bytes: *mut u8, capacity: i32, length: *mut i32) -> i32;
        fn geld_delete_biometric_key() -> i32;
    }
    pub fn available() -> bool {
        unsafe { geld_biometry_available() == 1 }
    }
    pub fn store(key: &[u8; 32]) -> Result<(), String> {
        let status = unsafe { geld_store_biometric_key(key.as_ptr(), 32) };
        status_result(status)
    }
    pub fn load() -> Result<Zeroizing<[u8; 32]>, String> {
        let mut key = Zeroizing::new([0_u8; 32]);
        let mut length = 0;
        let status = unsafe { geld_load_biometric_key(key.as_mut_ptr(), 32, &mut length) };
        status_result(status)?;
        if length != 32 {
            return Err("Keychain returned an invalid vault key. Use your passphrase.".into());
        }
        Ok(key)
    }
    pub fn delete() -> Result<(), String> {
        let status = unsafe { geld_delete_biometric_key() };
        status_result(status)
    }
    fn status_result(status: i32) -> Result<(), String> {
        if status == 0 {
            Ok(())
        } else {
            Err(format!(
                "Keychain biometric operation failed (status {status}). Use your passphrase."
            ))
        }
    }
}

#[cfg(target_os = "macos")]
fn platform_biometry_available() -> bool {
    keychain::available()
}
#[cfg(not(target_os = "macos"))]
fn platform_biometry_available() -> bool {
    false
}
#[cfg(target_os = "macos")]
fn platform_store_biometric_key(key: &[u8; 32]) -> Result<(), String> {
    keychain::store(key)
}
#[cfg(not(target_os = "macos"))]
fn platform_store_biometric_key(_: &[u8; 32]) -> Result<(), String> {
    Err("Biometric unlocking is supported only in the macOS app.".into())
}
#[cfg(target_os = "macos")]
fn platform_load_biometric_key() -> Result<Zeroizing<[u8; 32]>, String> {
    keychain::load()
}
#[cfg(not(target_os = "macos"))]
fn platform_load_biometric_key() -> Result<Zeroizing<[u8; 32]>, String> {
    Err("Biometric unlocking is supported only in the macOS app.".into())
}
#[cfg(target_os = "macos")]
fn platform_delete_biometric_key() -> Result<(), String> {
    keychain::delete()
}
#[cfg(not(target_os = "macos"))]
fn platform_delete_biometric_key() -> Result<(), String> {
    Err("Biometric unlocking is supported only in the macOS app.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "correct horse battery staple";
    const LEDGER: &str = r#"{"accounts":[],"transactions":[],"budgets":[],"drafts":[]}"#;

    #[test]
    fn setup_encrypts_and_unlocks_and_lock_clears_session() {
        let directory = tempfile::tempdir().unwrap();
        let session = VaultSession::new();
        session.setup(directory.path(), PASS, LEDGER).unwrap();
        let raw = fs::read_to_string(vault_path(directory.path())).unwrap();
        assert!(!raw.contains("accounts"));
        session.lock().unwrap();
        assert!(session
            .load(directory.path())
            .unwrap_err()
            .contains("locked"));
        session.unlock(directory.path(), PASS).unwrap();
        assert_eq!(session.load(directory.path()).unwrap().unwrap(), LEDGER);
    }

    #[test]
    fn wrong_passphrase_and_tampering_fail_closed() {
        let directory = tempfile::tempdir().unwrap();
        let session = VaultSession::new();
        session.setup(directory.path(), PASS, LEDGER).unwrap();
        session.lock().unwrap();
        assert!(session
            .unlock(directory.path(), "wrong passphrase entirely")
            .is_err());
        assert!(!session.status(directory.path()).unwrap().unlocked);
        let mut envelope = read_vault(directory.path()).unwrap();
        envelope.ciphertext = B64.encode(b"tampered ciphertext");
        fs::write(
            vault_path(directory.path()),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
        assert!(session.unlock(directory.path(), PASS).is_err());
        assert!(!session.status(directory.path()).unwrap().unlocked);
    }

    #[test]
    fn migration_requires_explicit_source_and_preserves_all_legacy_files() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("ledger.json"), LEDGER).unwrap();
        fs::write(directory.path().join("ledger.previous.json"), LEDGER).unwrap();
        fs::write(directory.path().join("ledger.next.json"), LEDGER).unwrap();
        let session = VaultSession::new();
        assert!(session
            .migrate(directory.path(), PASS, "../ledger.json")
            .is_err());
        session
            .migrate(directory.path(), PASS, "ledger.previous.json")
            .unwrap();
        assert_eq!(legacy_files(directory.path()).len(), 3);
        assert_eq!(session.load(directory.path()).unwrap().unwrap(), LEDGER);
    }

    #[test]
    fn invalid_json_does_not_create_or_replace_vault() {
        let directory = tempfile::tempdir().unwrap();
        let session = VaultSession::new();
        assert!(session.setup(directory.path(), PASS, "not-json").is_err());
        assert!(!vault_exists(directory.path()));
    }
}
