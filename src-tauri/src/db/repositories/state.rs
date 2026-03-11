use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::db::models::AppState;

pub struct StateRepository {
    conn: Arc<Mutex<Connection>>,
}

impl StateRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn set(&self, key: &str, value: &str, now: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO app_state (key, value, updated_at) VALUES (?1, ?2, ?3)",
            (key, value, now),
        ).context("Failed to set app_state")?;
        
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM app_state WHERE key = ?1")?;

        let value = stmt.query_row([key], |row| row.get(0)).optional()?;
        Ok(value)
    }

    #[allow(dead_code)]
    pub fn get_all(&self) -> Result<Vec<AppState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value, updated_at FROM app_state ORDER BY key")?;

        let states = stmt.query_map([], |row| {
            Ok(AppState {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(states)
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM app_state WHERE key = ?1", [key])
            .context("Failed to delete app_state")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migration::Migrator};
    use std::env;

    #[test]
    fn test_set_and_get_state() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_state_repo.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = StateRepository::new(db.get_connection());
        let now = "2026-03-12T02:00:00Z";
        
        repo.set("paused_until", "2026-03-12T10:00:00Z", now).unwrap();
        
        let value = repo.get("paused_until").unwrap();
        assert_eq!(value, Some("2026-03-12T10:00:00Z".to_string()));

        let all_states = repo.get_all().unwrap();
        assert_eq!(all_states.len(), 1);

        repo.delete("paused_until").unwrap();
        let value = repo.get("paused_until").unwrap();
        assert_eq!(value, None);

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_replace_state() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_state_replace.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = StateRepository::new(db.get_connection());
        let now = "2026-03-12T02:00:00Z";
        
        repo.set("test_key", "value1", now).unwrap();
        repo.set("test_key", "value2", now).unwrap();
        
        let value = repo.get("test_key").unwrap();
        assert_eq!(value, Some("value2".to_string()));

        let all_states = repo.get_all().unwrap();
        assert_eq!(all_states.len(), 1);

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
