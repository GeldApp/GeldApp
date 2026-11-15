use std::{path::PathBuf, sync::Arc};
use tauri::Manager;
use zeroize::Zeroizing;

mod vault;
use vault::VaultSession;

struct VaultState(Arc<VaultSession>);

fn data_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|_| "Could not locate GeldApp storage".to_string())
}

async fn blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|_| "Vault operation failed unexpectedly".to_string())
}

#[tauri::command]
async fn vault_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
) -> Result<vault::Status, String> {
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.status(&directory)).await?
}

#[tauri::command]
async fn setup_vault(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
    passphrase: String,
    data: String,
) -> Result<(), String> {
    let passphrase = Zeroizing::new(passphrase);
    let data = Zeroizing::new(data);
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.setup(&directory, &passphrase, &data)).await?
}

#[tauri::command]
async fn migrate_legacy_vault(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
    passphrase: String,
    source: String,
) -> Result<(), String> {
    let passphrase = Zeroizing::new(passphrase);
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.migrate(&directory, &passphrase, &source)).await?
}

#[tauri::command]
async fn unlock_vault(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
    passphrase: String,
) -> Result<(), String> {
    let passphrase = Zeroizing::new(passphrase);
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.unlock(&directory, &passphrase)).await?
}

#[tauri::command]
async fn unlock_biometric(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.unlock_biometric(&directory)).await?
}

#[tauri::command]
async fn recover_previous_vault(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
    passphrase: String,
) -> Result<(), String> {
    let passphrase = Zeroizing::new(passphrase);
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.recover_previous(&directory, &passphrase)).await?
}

#[tauri::command]
async fn lock_vault(state: tauri::State<'_, VaultState>) -> Result<(), String> {
    let session = state.0.clone();
    blocking(move || session.lock()).await?
}

#[tauri::command]
async fn load_ledger(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
) -> Result<Option<String>, String> {
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.load(&directory)).await?
}

#[tauri::command]
async fn save_ledger(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
    data: String,
) -> Result<(), String> {
    let data = Zeroizing::new(data);
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.save(&directory, &data)).await?
}

#[tauri::command]
async fn set_vault_biometric(
    app: tauri::AppHandle,
    state: tauri::State<'_, VaultState>,
    enabled: bool,
) -> Result<(), String> {
    let directory = data_directory(&app)?;
    let session = state.0.clone();
    blocking(move || session.set_biometric(&directory, enabled)).await?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            let session = VaultSession::open(&directory).map_err(std::io::Error::other)?;
            app.manage(VaultState(Arc::new(session)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            vault_status,
            setup_vault,
            migrate_legacy_vault,
            unlock_vault,
            unlock_biometric,
            recover_previous_vault,
            lock_vault,
            load_ledger,
            save_ledger,
            set_vault_biometric,
        ])
        .run(tauri::generate_context!())
        .expect("Could not start GeldApp");
}
