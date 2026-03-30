use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    pub db_path: PathBuf,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let conn = Connection::open(&db_path)
            .context(format!("Failed to open database at {:?}", db_path))?;

        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
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

    /// Acquire a connection guard for the duration of a closure.
    /// Returns the closure's result.
    pub fn with_conn<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }

    /// Close the connection, replacing it with a no-op that returns errors.
    /// The caller must call reopen_connection() to restore functionality.
    pub fn close_connection(&self) {
        // Take the connection and replace with an in-memory DB
        // This releases the file lock on the real database file
        let _ = std::mem::replace(
            &mut *self.conn.lock().unwrap(),
            Connection::open_in_memory()
                .expect("in-memory DB should always open"),
        );
    }

    /// Reopen the connection to the current db_path. Must be called after
    /// close_connection() and after the database file has been replaced.
    pub fn reopen_connection(&self) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        *conn = Connection::open(&self.db_path)
            .context(format!("Failed to reopen database at {:?}", self.db_path))?;
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys on reopen")?;
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
