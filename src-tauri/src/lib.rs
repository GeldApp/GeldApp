use std::{path::PathBuf, sync::Arc};
use tauri::Manager;
use zeroize::Zeroizing;

#[cfg(all(feature = "e2e", not(debug_assertions)))]
compile_error!("The e2e WebDriver server may only be built in debug mode.");

mod vault;
use vault::VaultSession;

struct VaultState(Arc<VaultSession>);

fn data_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    #[cfg(feature = "e2e")]
    {
        const TEST_IDENTIFIER: &str = "app.geldapp.desktop.e2e";
        if app.config().identifier != TEST_IDENTIFIER {
            return Err("The native e2e build requires its isolated Tauri configuration.".into());
        }
        let directory = std::env::var_os("GELDAPP_E2E_DATA_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| {
                "GELDAPP_E2E_DATA_DIR must point to a temporary test directory.".to_string()
            })?;
        let temp_root = std::env::var_os("RUNNER_TEMP")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let expected_parent = temp_root
            .canonicalize()
            .map_err(|_| "Could not locate the temporary test directory.".to_string())?;
        let actual_parent = directory.parent()
            .and_then(|parent| parent.canonicalize().ok())
            .ok_or_else(|| "The native e2e data directory must be a direct child of the temporary test directory.".to_string())?;
        if actual_parent != expected_parent
            || !directory
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("geldapp-e2e-"))
            || std::fs::symlink_metadata(&directory)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err("The native e2e data directory must be a new, isolated geldapp-e2e-* directory under the runner temporary directory.".into());
        }
        Ok(directory)
    }

    #[cfg(not(feature = "e2e"))]
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
    #[cfg(feature = "e2e")]
    eprintln!("GELDAPP_E2E phase=run-entry");

    let builder = tauri::Builder::default();
    #[cfg(feature = "e2e")]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());

    #[cfg(feature = "e2e")]
    eprintln!("GELDAPP_E2E phase=webdriver-plugins-registered");

    let app = builder
        .setup(|app| {
            #[cfg(feature = "e2e")]
            eprintln!("GELDAPP_E2E phase=app-setup-entry");

            let directory = data_directory(app.handle()).map_err(std::io::Error::other)?;
            #[cfg(feature = "e2e")]
            eprintln!("GELDAPP_E2E phase=fixture-directory-accepted");

            let session = VaultSession::open(&directory).map_err(std::io::Error::other)?;
            #[cfg(feature = "e2e")]
            eprintln!("GELDAPP_E2E phase=vault-session-open");

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
        ]);

    #[cfg(feature = "e2e")]
    eprintln!("GELDAPP_E2E phase=event-loop-start");

    app.run(tauri::generate_context!())
        .expect("Could not start GeldApp");

    #[cfg(feature = "e2e")]
    eprintln!("GELDAPP_E2E phase=app-exited");
}
