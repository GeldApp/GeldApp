use std::{fs, io::Write, sync::Mutex};
use tauri::Manager;

struct LedgerLock(Mutex<()>);

#[tauri::command]
fn load_ledger(
    app: tauri::AppHandle,
    state: tauri::State<LedgerLock>,
) -> Result<Option<String>, String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("ledger.json");
    match fs::read_to_string(path) {
        Ok(data) => Ok(Some(data)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn save_ledger(
    app: tauri::AppHandle,
    state: tauri::State<LedgerLock>,
    data: String,
) -> Result<(), String> {
    let _guard = state.0.lock().map_err(|e| e.to_string())?;
    if data.len() > 30_000_000 {
        return Err("Ledger exceeds 30 MB".into());
    }
    serde_json::from_str::<serde_json::Value>(&data).map_err(|e| e.to_string())?;
    let directory = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join("ledger.json");
    let temporary = directory.join("ledger.next.json");
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary).map_err(|e| e.to_string())?;
    file.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    // Preserve the last successful snapshot if replacement is interrupted.
    if path.exists() {
        fs::copy(&path, directory.join("ledger.previous.json")).map_err(|e| e.to_string())?;
    }
    fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LedgerLock(Mutex::new(())))
        .invoke_handler(tauri::generate_handler![load_ledger, save_ledger])
        .run(tauri::generate_context!())
        .expect("Could not start GeldApp");
}
