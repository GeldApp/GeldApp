use tauri::Manager;
use zeroize::Zeroize;

mod vault;
use vault::VaultSession;

fn data_directory(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|_| "Could not locate GeldApp storage".to_string())
}

#[tauri::command]
fn vault_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
) -> Result<vault::Status, String> {
    state.status(&data_directory(&app)?)
}

#[tauri::command]
fn setup_vault(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
    mut passphrase: String,
    data: String,
) -> Result<(), String> {
    let result =
        data_directory(&app).and_then(|directory| state.setup(&directory, &passphrase, &data));
    passphrase.zeroize();
    result
}

#[tauri::command]
fn migrate_legacy_vault(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
    mut passphrase: String,
    source: String,
) -> Result<(), String> {
    let result =
        data_directory(&app).and_then(|directory| state.migrate(&directory, &passphrase, &source));
    passphrase.zeroize();
    result
}

#[tauri::command]
fn unlock_vault(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
    mut passphrase: String,
) -> Result<(), String> {
    let result = data_directory(&app).and_then(|directory| state.unlock(&directory, &passphrase));
    passphrase.zeroize();
    result
}

#[tauri::command]
fn unlock_biometric(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
) -> Result<(), String> {
    state.unlock_biometric(&data_directory(&app)?)
}

#[tauri::command]
fn lock_vault(state: tauri::State<'_, VaultSession>) -> Result<(), String> {
    state.lock()
}

#[tauri::command]
fn load_ledger(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
) -> Result<Option<String>, String> {
    state.load(&data_directory(&app)?)
}

#[tauri::command]
fn save_ledger(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
    data: String,
) -> Result<(), String> {
    state.save(&data_directory(&app)?, &data)
}

#[tauri::command]
fn set_vault_biometric(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultSession>,
    enabled: bool,
) -> Result<(), String> {
    state.set_biometric(&data_directory(&app)?, enabled)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(VaultSession::new())
        .invoke_handler(tauri::generate_handler![
            vault_status,
            setup_vault,
            migrate_legacy_vault,
            unlock_vault,
            unlock_biometric,
            lock_vault,
            load_ledger,
            save_ledger,
            set_vault_biometric,
        ])
        .run(tauri::generate_context!())
        .expect("Could not start GeldApp");
}
