use tauri::{Manager, State};

use crate::db::backup::{self, BackupEntry};
use crate::db::Database;

fn get_app_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))
}

#[tauri::command]
pub fn list_backups(app: tauri::AppHandle) -> Result<Vec<BackupEntry>, String> {
    let dir = get_app_data_dir(&app)?;
    backup::list_backups(&dir).map_err(|e| format!("Failed to list backups: {}", e))
}

#[tauri::command]
pub fn restore_backup(
    app: tauri::AppHandle,
    _db: State<Database>,
    file_name: String,
) -> Result<(), String> {
    let dir = get_app_data_dir(&app)?;
    backup::restore_backup(&dir, &file_name).map_err(|e| format!("Failed to restore backup: {}", e))
}
