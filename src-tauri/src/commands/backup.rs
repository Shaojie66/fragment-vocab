use tauri::{Emitter, Manager, State};

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
    db: State<Database>,
    file_name: String,
) -> Result<(), String> {
    let dir = get_app_data_dir(&app)?;

    // Close the connection before restoring to release the file lock.
    // Without this, the old connection keeps a lock on the database file,
    // making the restore ineffective until the app is restarted.
    db.close_connection();

    let result = backup::restore_backup(&dir, &file_name)
        .map_err(|e| format!("Failed to restore backup: {}", e));

    // Reopen the connection to the restored database file.
    // If this fails, the app may be in an inconsistent state (restored file
    // but in-memory connection). Emit an event so the frontend can prompt
    // the user to restart.
    if let Err(e) = db.reopen_connection() {
        // Emit event so frontend can show "please restart" message
        let _ = app.emit("backup-restored-needs-restart", format!("Reopen failed: {}", e));
        return Err(format!(
            "Backup restored but reconnect failed: {}. Please restart the app.",
            e
        ));
    }

    let _ = app.emit("backup-restored", ());
    result
}
