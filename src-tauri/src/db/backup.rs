use anyhow::{Context, Result};
use chrono::Local;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_BACKUPS: usize = 5;
const BACKUP_DIR_NAME: &str = "backups";
const DB_FILE_NAME: &str = "fragment-vocab.db";

fn backup_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(BACKUP_DIR_NAME)
}

fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(DB_FILE_NAME)
}

/// Create a timestamped backup of the database.
/// Called automatically at app startup.
pub fn create_backup(app_data_dir: &Path) -> Result<Option<PathBuf>> {
    let source = db_path(app_data_dir);
    if !source.exists() {
        debug!("No database file to backup");
        return Ok(None);
    }

    let dir = backup_dir(app_data_dir);
    fs::create_dir_all(&dir).context("Failed to create backup directory")?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("fragment-vocab-{}.db", timestamp);
    let dest = dir.join(&backup_name);

    fs::copy(&source, &dest).context("Failed to copy database for backup")?;
    info!("Database backed up to {}", backup_name);

    cleanup_old_backups(&dir)?;

    Ok(Some(dest))
}

/// Remove oldest backups, keeping at most MAX_BACKUPS.
fn cleanup_old_backups(dir: &Path) -> Result<()> {
    let mut backups = list_backup_files(dir)?;

    // Sort by name descending (newest first, since names contain timestamps)
    backups.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    if backups.len() > MAX_BACKUPS {
        for old in &backups[MAX_BACKUPS..] {
            debug!("Removing old backup: {:?}", old.file_name());
            fs::remove_file(old).ok();
        }
    }

    Ok(())
}

/// List all backup files in the backup directory.
fn list_backup_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir).context("Failed to read backup directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "db") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("fragment-vocab-") {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

/// Backup entry for frontend display.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupEntry {
    pub file_name: String,
    pub created_at: String,
    pub size_bytes: u64,
}

/// List available backups for the UI.
pub fn list_backups(app_data_dir: &Path) -> Result<Vec<BackupEntry>> {
    let dir = backup_dir(app_data_dir);
    let mut files = list_backup_files(&dir)?;
    files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    let entries = files
        .into_iter()
        .filter_map(|path| {
            let file_name = path.file_name()?.to_str()?.to_string();
            let metadata = fs::metadata(&path).ok()?;

            // Extract timestamp from filename: fragment-vocab-YYYYMMDD-HHMMSS.db
            let created_at = file_name
                .strip_prefix("fragment-vocab-")
                .and_then(|s| s.strip_suffix(".db"))
                .map(|ts| {
                    // Convert YYYYMMDD-HHMMSS to YYYY-MM-DD HH:MM:SS
                    if ts.len() == 15 {
                        format!(
                            "{}-{}-{} {}:{}:{}",
                            &ts[0..4],
                            &ts[4..6],
                            &ts[6..8],
                            &ts[9..11],
                            &ts[11..13],
                            &ts[13..15]
                        )
                    } else {
                        ts.to_string()
                    }
                })
                .unwrap_or_default();

            Some(BackupEntry {
                file_name,
                created_at,
                size_bytes: metadata.len(),
            })
        })
        .collect();

    Ok(entries)
}

/// Restore a backup by replacing the current database.
/// Caller (restore_backup command) is responsible for closing/reopening connections.
pub fn restore_backup(app_data_dir: &Path, backup_file_name: &str) -> Result<()> {
    let dir = backup_dir(app_data_dir);
    let source = dir.join(backup_file_name);

    if !source.exists() {
        anyhow::bail!("Backup file not found: {}", backup_file_name);
    }

    // Validate the backup filename to prevent path traversal
    if backup_file_name.contains("..") || backup_file_name.contains('/') {
        anyhow::bail!("Invalid backup filename");
    }

    let dest = db_path(app_data_dir);

    // Create a safety backup of current DB before restoring
    let safety_name = format!(
        "fragment-vocab-pre-restore-{}.db",
        Local::now().format("%Y%m%d-%H%M%S")
    );
    let safety_path = dir.join(&safety_name);
    if dest.exists() {
        fs::copy(&dest, &safety_path).context("Failed to create safety backup before restore")?;
        info!("Safety backup created: {}", safety_name);
    }

    fs::copy(&source, &dest).context("Failed to restore backup")?;
    info!("Database restored from {}", backup_file_name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("fragment-vocab-backup-test-{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Create a fake database file
        fs::write(dir.join(DB_FILE_NAME), b"test database content").unwrap();
        dir
    }

    fn teardown(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_create_backup() {
        let dir = setup_test_dir("create");
        let result = create_backup(&dir).unwrap();
        assert!(result.is_some());

        let backup_path = result.unwrap();
        assert!(backup_path.exists());

        let content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(content, "test database content");

        teardown(&dir);
    }

    #[test]
    fn test_list_backups() {
        let dir = setup_test_dir("list");
        create_backup(&dir).unwrap();

        let backups = list_backups(&dir).unwrap();
        assert_eq!(backups.len(), 1);
        assert!(backups[0].file_name.starts_with("fragment-vocab-"));
        assert!(backups[0].size_bytes > 0);

        teardown(&dir);
    }

    #[test]
    fn test_cleanup_old_backups() {
        let dir = setup_test_dir("cleanup");
        let backup_dir = dir.join(BACKUP_DIR_NAME);
        fs::create_dir_all(&backup_dir).unwrap();

        // Create 7 fake backup files
        for i in 1..=7 {
            let name = format!("fragment-vocab-20260301-00000{}.db", i);
            fs::write(backup_dir.join(&name), b"data").unwrap();
        }

        cleanup_old_backups(&backup_dir).unwrap();

        let remaining = list_backup_files(&backup_dir).unwrap();
        assert_eq!(remaining.len(), MAX_BACKUPS);

        teardown(&dir);
    }

    #[test]
    fn test_restore_backup() {
        let dir = setup_test_dir("restore");
        create_backup(&dir).unwrap();

        // Modify the "database"
        fs::write(dir.join(DB_FILE_NAME), b"modified content").unwrap();

        let backups = list_backups(&dir).unwrap();
        restore_backup(&dir, &backups[0].file_name).unwrap();

        let content = fs::read_to_string(dir.join(DB_FILE_NAME)).unwrap();
        assert_eq!(content, "test database content");

        teardown(&dir);
    }
}
