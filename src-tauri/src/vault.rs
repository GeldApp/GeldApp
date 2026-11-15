use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use fs2::FileExt;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};
use zeroize::{Zeroize, Zeroizing};

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
    pub previous_vault_exists: bool,
}

pub struct VaultSession {
    operation: Mutex<()>,
    key: Mutex<Option<Zeroizing<[u8; 32]>>>,
    _process_lock: Option<File>,
}

impl VaultSession {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            operation: Mutex::new(()),
            key: Mutex::new(None),
            _process_lock: None,
        }
    }

    pub fn open(directory: &Path) -> Result<Self, String> {
        fs::create_dir_all(directory)
            .map_err(|_| "Could not prepare GeldApp storage".to_string())?;
        let lock_path = directory.join(".vault.lock");
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(lock_path)
            .map_err(|_| "Could not open the vault session lock".to_string())?;
        file.try_lock_exclusive().map_err(|_| "Another GeldApp process already owns this vault. Close it before opening this workspace.".to_string())?;
        Ok(Self {
            operation: Mutex::new(()),
            key: Mutex::new(None),
            _process_lock: Some(file),
        })
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
            previous_vault_exists: previous_path(directory).is_file(),
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
        let data = Zeroizing::new(fs::read_to_string(directory.join(source)).map_err(|_| {
            "Could not read the selected legacy ledger. It was left unchanged.".to_string()
        })?);
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

    pub fn recover_previous(&self, directory: &Path, passphrase: &str) -> Result<(), String> {
        self.recover_previous_with_cleanup(directory, passphrase, platform_delete_biometric_key)
    }

    fn recover_previous_with_cleanup(
        &self,
        directory: &Path,
        passphrase: &str,
        delete_biometric_key: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        let _operation = self
            .operation
            .lock()
            .map_err(|_| "Vault is busy".to_string())?;
        let previous = previous_path(directory);
        let raw = fs::read_to_string(&previous)
            .map_err(|_| "No readable previous encrypted vault exists.".to_string())?;
        let mut envelope: VaultFile = serde_json::from_str(&raw).map_err(|_| {
            "Previous encrypted vault is invalid and was left unchanged.".to_string()
        })?;
        let key = unwrap_key(passphrase, &envelope)
            .map_err(|_| "Could not unlock previous vault. Check your passphrase.".to_string())?;
        let data = decrypt_data(&key, &envelope).map_err(|_| {
            "Previous vault authentication failed. Existing files were left unchanged.".to_string()
        })?;
        validate_data(
            std::str::from_utf8(&data).map_err(|_| "Previous vault data is invalid".to_string())?,
        )?;
        let biometric_cleanup_needed = delete_biometric_key().is_err();
        // If Keychain refused removal, keep the recovery screen's opt-out visible
        // so the user can retry cleanup from Settings after unlocking by passphrase.
        envelope.biometric_enabled = biometric_cleanup_needed;
        let current = vault_path(directory);
        let rejected = directory.join(format!("vault.rejected-{}.enc", rand::random::<u64>()));
        let recovery_next = directory.join("vault.recovery.next.enc");
        write_exact_atomic_stage(
            &recovery_next,
            &serde_json::to_vec(&envelope)
                .map_err(|_| "Could not encode previous vault".to_string())?,
        )?;
        if current.exists() {
            fs::rename(&current, &rejected)
                .map_err(|_| "Could not preserve the current vault before recovery.".to_string())?;
        }
        fs::rename(&recovery_next, &current).map_err(|_| {
            "Could not restore previous vault. The previous encrypted file remains available."
                .to_string()
        })?;
        if let Ok(dir) = File::open(directory) {
            let _ = dir.sync_all();
        }
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
        self.set_biometric_with(
            directory,
            enabled,
            platform_store_biometric_key,
            platform_delete_biometric_key,
            write_vault,
        )
    }

    fn set_biometric_with(
        &self,
        directory: &Path,
        enabled: bool,
        store_key: impl FnOnce(&[u8; 32]) -> Result<(), String>,
        delete_key: impl FnOnce() -> Result<(), String>,
        persist: impl FnOnce(&Path, &VaultFile) -> Result<(), String>,
    ) -> Result<(), String> {
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
            store_key(key)?;
            envelope.biometric_enabled = true;
            if let Err(persist_error) = persist(directory, &envelope) {
                return match delete_key() {
                    Ok(()) => Err(format!(
                        "Could not save the Touch ID setting; the saved Keychain key was removed. {persist_error}"
                    )),
                    Err(cleanup_error) => Err(format!(
                        "Could not save the Touch ID setting ({persist_error}) or remove its saved Keychain key ({cleanup_error}). Use “Remove saved Touch ID key” in Settings before retrying."
                    )),
                };
            }
        } else {
            delete_key()?;
            envelope.biometric_enabled = false;
            persist(directory, &envelope)?;
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
    let ledger = Zeroizing::new(
        serde_json::from_str::<LedgerData>(data)
            .map_err(|_| "Ledger is not a valid GeldApp ledger".to_string())?,
    );
    if ledger.version != 1 {
        return Err("Unsupported ledger version".into());
    }
    let mut account_ids = HashSet::new();
    for account in &ledger.accounts {
        if !account_ids.insert(&account.id)
            || !["Current account", "Savings", "Cash"].contains(&account.kind.as_str())
            || !safe_integer(account.balance)
        {
            return Err("Invalid account data".into());
        }
    }
    let mut transaction_ids = HashSet::new();
    for transaction in &ledger.transactions {
        if !transaction_ids.insert(&transaction.id)
            || !account_ids.contains(&transaction.account_id)
            || !valid_category(&transaction.category)
            || !valid_date(&transaction.date)
            || !safe_integer(transaction.amount)
        {
            return Err("Invalid transaction data".into());
        }
    }
    for budget in &ledger.budgets {
        if !valid_category(&budget.category) || !safe_integer(budget.limit) || budget.limit <= 0 {
            return Err("Invalid budget data".into());
        }
    }
    for draft in &ledger.drafts {
        if !account_ids.contains(&draft.account_id)
            || !safe_integer(draft.amount)
            || draft.amount <= 0
        {
            return Err("Invalid payment draft".into());
        }
    }
    Ok(())
}

#[allow(dead_code)]
#[derive(Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
struct LedgerData {
    version: u8,
    accounts: Vec<AccountData>,
    transactions: Vec<TransactionData>,
    budgets: Vec<BudgetData>,
    drafts: Vec<DraftData>,
}
#[allow(dead_code)]
#[derive(Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
struct AccountData {
    id: String,
    name: String,
    bank: String,
    iban: String,
    balance: i64,
    color: String,
    kind: String,
}
#[allow(dead_code)]
#[derive(Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
struct TransactionData {
    id: String,
    account_id: String,
    merchant: String,
    note: String,
    date: String,
    amount: i64,
    category: String,
}
#[allow(dead_code)]
#[derive(Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
struct BudgetData {
    id: String,
    category: String,
    limit: i64,
}
#[allow(dead_code)]
#[derive(Deserialize, Zeroize)]
#[serde(rename_all = "camelCase")]
struct DraftData {
    id: String,
    account_id: String,
    recipient: String,
    iban: String,
    amount: i64,
    reference: String,
}

fn safe_integer(value: i64) -> bool {
    value.unsigned_abs() <= 9_007_199_254_740_991
}

fn valid_category(value: &str) -> bool {
    [
        "Groceries",
        "Food & drink",
        "Shopping",
        "Transport",
        "Home",
        "Subscriptions",
        "Health",
        "Income",
        "Transfers",
        "Other",
    ]
    .contains(&value)
}

fn valid_date(value: &str) -> bool {
    let Some((year, rest)) = value.split_once('-') else {
        return false;
    };
    let Some((month, day)) = rest.split_once('-') else {
        return false;
    };
    if year.len() != 4 || month.len() != 2 || day.len() != 2 || value.len() != 10 {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    day >= 1 && day <= days
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
    let raw = fs::read_to_string(vault_path(directory)).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound && previous_path(directory).is_file() {
            "The current vault is missing. Choose explicit recovery from the previous encrypted snapshot.".to_string()
        } else {
            "Could not read the current vault. Existing files were left unchanged.".to_string()
        }
    })?;
    serde_json::from_str(&raw)
        .map_err(|_| "Current vault file is invalid. Existing files were left unchanged.".into())
}

fn write_exact_atomic_stage(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|_| "Could not stage previous vault recovery".to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Could not stage previous vault recovery".to_string())
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
        if previous.exists() {
            fs::remove_file(&previous)
                .map_err(|_| "Could not rotate encrypted vault snapshots".to_string())?;
        }
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

#[cfg(all(target_os = "macos", not(test)))]
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

#[cfg(all(target_os = "macos", not(test)))]
fn platform_biometry_available() -> bool {
    keychain::available()
}
#[cfg(any(not(target_os = "macos"), test))]
fn platform_biometry_available() -> bool {
    false
}
#[cfg(all(target_os = "macos", not(test)))]
fn platform_store_biometric_key(key: &[u8; 32]) -> Result<(), String> {
    keychain::store(key)
}
#[cfg(any(not(target_os = "macos"), test))]
fn platform_store_biometric_key(_: &[u8; 32]) -> Result<(), String> {
    Err("Biometric unlocking is supported only in the macOS app.".into())
}
#[cfg(all(target_os = "macos", not(test)))]
fn platform_load_biometric_key() -> Result<Zeroizing<[u8; 32]>, String> {
    keychain::load()
}
#[cfg(any(not(target_os = "macos"), test))]
fn platform_load_biometric_key() -> Result<Zeroizing<[u8; 32]>, String> {
    Err("Biometric unlocking is supported only in the macOS app.".into())
}
#[cfg(all(target_os = "macos", not(test)))]
fn platform_delete_biometric_key() -> Result<(), String> {
    keychain::delete()
}
#[cfg(any(not(target_os = "macos"), test))]
fn platform_delete_biometric_key() -> Result<(), String> {
    Err("Biometric unlocking is supported only in the macOS app.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "correct horse battery staple";
    const LEDGER: &str =
        r#"{"version":1,"accounts":[],"transactions":[],"budgets":[],"drafts":[]}"#;

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

    #[test]
    fn valid_json_with_wrong_schema_is_rejected_without_migration_or_cleanup() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("ledger.json"), "{}").unwrap();
        fs::write(directory.path().join("ledger.previous.json"), LEDGER).unwrap();
        let session = VaultSession::new();
        assert!(session
            .migrate(directory.path(), PASS, "ledger.json")
            .is_err());
        assert!(!vault_exists(directory.path()));
        assert_eq!(legacy_files(directory.path()).len(), 2);
    }

    #[test]
    fn ledger_references_and_dates_are_validated_before_encryption() {
        let invalid = r#"{"version":1,"accounts":[],"transactions":[{"id":"t1","accountId":"missing","merchant":"Coffee","note":"","date":"2026-02-30","amount":-450,"category":"Food & drink"}],"budgets":[],"drafts":[]}"#;
        assert!(validate_data(invalid).is_err());
        assert!(!valid_date("2026-02-30"));
        assert!(valid_date("2024-02-29"));
    }

    #[test]
    fn a_second_session_cannot_own_the_same_directory() {
        let directory = tempfile::tempdir().unwrap();
        let first = VaultSession::open(directory.path()).unwrap();
        assert!(VaultSession::open(directory.path()).is_err());
        drop(first);
        assert!(VaultSession::open(directory.path()).is_ok());
    }

    #[test]
    fn previous_snapshot_recovery_authenticates_and_preserves_rejected_copy() {
        let directory = tempfile::tempdir().unwrap();
        let session = VaultSession::new();
        session.setup(directory.path(), PASS, LEDGER).unwrap();
        let newer = r#"{"version":1,"accounts":[],"transactions":[],"budgets":[{"id":"b1","category":"Groceries","limit":100}],"drafts":[]}"#;
        session.save(directory.path(), newer).unwrap();
        session.lock().unwrap();
        fs::write(vault_path(directory.path()), b"damaged current file").unwrap();
        assert!(session
            .recover_previous(directory.path(), "wrong passphrase")
            .is_err());
        assert_eq!(
            fs::read(vault_path(directory.path())).unwrap(),
            b"damaged current file"
        );
        session
            .recover_previous_with_cleanup(directory.path(), PASS, || Ok(()))
            .unwrap();
        assert_eq!(session.load(directory.path()).unwrap().unwrap(), LEDGER);
        assert!(!session.status(directory.path()).unwrap().biometric_enabled);
        assert!(fs::read_dir(directory.path()).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("vault.rejected-")));
        assert!(previous_path(directory.path()).is_file());
    }

    #[test]
    fn recovery_keeps_biometric_cleanup_retry_visible_when_provider_fails() {
        let directory = tempfile::tempdir().unwrap();
        let session = VaultSession::new();
        session.setup(directory.path(), PASS, LEDGER).unwrap();
        let mut envelope: VaultFile =
            serde_json::from_slice(&fs::read(vault_path(directory.path())).unwrap()).unwrap();
        envelope.biometric_enabled = true;
        fs::write(
            vault_path(directory.path()),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
        session.save(directory.path(), LEDGER).unwrap();
        session.lock().unwrap();

        session
            .recover_previous_with_cleanup(directory.path(), PASS, || {
                Err("mock Keychain cleanup failure".into())
            })
            .unwrap();

        assert!(session.status(directory.path()).unwrap().biometric_enabled);
    }

    #[test]
    fn biometric_setup_reports_orphaned_key_and_independent_removal_clears_it() {
        let directory = tempfile::tempdir().unwrap();
        let session = VaultSession::new();
        session.setup(directory.path(), PASS, LEDGER).unwrap();

        let error = session
            .set_biometric_with(
                directory.path(),
                true,
                |_| Ok(()),
                || Err("mock delete failure".into()),
                |_, _| Err("mock vault write failure".into()),
            )
            .unwrap_err();

        assert!(error.contains("mock vault write failure"));
        assert!(error.contains("mock delete failure"));
        assert!(error.contains("Remove saved Touch ID key"));
        assert!(!session.status(directory.path()).unwrap().biometric_enabled);

        session
            .set_biometric_with(directory.path(), false, |_| Ok(()), || Ok(()), write_vault)
            .unwrap();
        assert!(!session.status(directory.path()).unwrap().biometric_enabled);
    }
}
