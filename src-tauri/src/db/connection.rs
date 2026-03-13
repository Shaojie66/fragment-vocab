use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let conn = Connection::open(&db_path)
            .context(format!("Failed to open database at {:?}", db_path))?;

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    pub fn execute_migration(&self, sql: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql)
            .context("Failed to execute migration")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_database_creation() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_fragment_vocab.db");

        // 清理可能存在的旧文件
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        assert!(db_path.exists());

        // 清理
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_fk.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        let conn = db.conn.lock().unwrap();

        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();

        assert_eq!(fk_enabled, 1);

        drop(conn);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
