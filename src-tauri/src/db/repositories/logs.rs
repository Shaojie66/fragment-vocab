use anyhow::{Context, Result};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::db::models::ReviewLog;

pub struct LogsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LogsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(&self, card_id: i64, shown_at: &str, result: &str, trigger_type: &str, response_ms: Option<i32>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO review_logs (card_id, shown_at, result, trigger_type, response_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
            (card_id, shown_at, result, trigger_type, response_ms),
        ).context("Failed to insert review_log")?;
        
        Ok(conn.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub fn get_by_card_id(&self, card_id: i64, limit: i64) -> Result<Vec<ReviewLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, card_id, shown_at, result, trigger_type, response_ms, created_at FROM review_logs WHERE card_id = ?1 ORDER BY shown_at DESC LIMIT ?2"
        )?;

        let logs = stmt.query_map([card_id, limit], |row| {
            Ok(ReviewLog {
                id: row.get(0)?,
                card_id: row.get(1)?,
                shown_at: row.get(2)?,
                result: row.get(3)?,
                trigger_type: row.get(4)?,
                response_ms: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    #[allow(dead_code)]
    pub fn count_by_result(&self, result: &str, since: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        
        let count: i64 = if let Some(since_time) = since {
            conn.query_row(
                "SELECT COUNT(*) FROM review_logs WHERE result = ?1 AND shown_at >= ?2",
                [result, since_time],
                |row| row.get(0)
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM review_logs WHERE result = ?1",
                [result],
                |row| row.get(0)
            )?
        };
        
        Ok(count)
    }

    pub fn get_recent_logs(&self, limit: i64) -> Result<Vec<ReviewLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, card_id, shown_at, result, trigger_type, response_ms, created_at FROM review_logs ORDER BY shown_at DESC LIMIT ?1"
        )?;

        let logs = stmt.query_map([limit], |row| {
            Ok(ReviewLog {
                id: row.get(0)?,
                card_id: row.get(1)?,
                shown_at: row.get(2)?,
                result: row.get(3)?,
                trigger_type: row.get(4)?,
                response_ms: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migration::Migrator, repositories::{WordsRepository, CardsRepository}};
    use std::env;

    #[test]
    fn test_insert_and_get_logs() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_logs_repo.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let words_repo = WordsRepository::new(db.get_connection());
        let cards_repo = CardsRepository::new(db.get_connection());
        let logs_repo = LogsRepository::new(db.get_connection());

        let word_id = words_repo.insert("test", "测试", "test", None, None, 1).unwrap();
        let card_id = cards_repo.insert(word_id).unwrap();

        let log_id = logs_repo.insert(card_id, "2026-03-12T00:00:00Z", "know", "idle", Some(1500)).unwrap();
        assert!(log_id > 0);

        let logs = logs_repo.get_by_card_id(card_id, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].result, "know");

        let count = logs_repo.count_by_result("know", None).unwrap();
        assert_eq!(count, 1);

        drop(logs_repo);
        drop(cards_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
