use anyhow::{Context, Result};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::commands::DayStats;
use crate::db::models::ReviewLog;

pub struct LogsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LogsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(
        &self,
        card_id: i64,
        shown_at: &str,
        result: &str,
        trigger_type: &str,
        response_ms: Option<i32>,
    ) -> Result<i64> {
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

        let logs = stmt
            .query_map([card_id, limit], |row| {
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
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM review_logs WHERE result = ?1",
                [result],
                |row| row.get(0),
            )?
        };

        Ok(count)
    }

    pub fn count_all(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM review_logs", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_recent_logs(&self, limit: i64) -> Result<Vec<ReviewLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, card_id, shown_at, result, trigger_type, response_ms, created_at FROM review_logs ORDER BY shown_at DESC LIMIT ?1"
        )?;

        let logs = stmt
            .query_map([limit], |row| {
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

    /// Get all logs since a given UTC timestamp (inclusive), ordered by shown_at DESC.
    pub fn get_logs_since(&self, since_utc: &str) -> Result<Vec<ReviewLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, card_id, shown_at, result, trigger_type, response_ms, created_at FROM review_logs WHERE shown_at >= ?1 ORDER BY shown_at DESC"
        )?;

        let logs = stmt
            .query_map([since_utc], |row| {
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

    pub fn get_history_stats(&self, since_utc: &str) -> Result<Vec<DayStats>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "WITH first_reviews AS (
                SELECT
                    card_id,
                    MIN(shown_at) AS first_shown_at
                FROM review_logs
                WHERE result IN ('know', 'dont_know')
                GROUP BY card_id
            )
            SELECT
                date(rl.shown_at, 'localtime') AS day,
                COUNT(*) AS total_reviews,
                SUM(CASE WHEN rl.result = 'know' THEN 1 ELSE 0 END) AS correct_count,
                COUNT(DISTINCT CASE
                    WHEN fr.first_shown_at IS NOT NULL
                     AND date(fr.first_shown_at, 'localtime') = date(rl.shown_at, 'localtime')
                    THEN rl.card_id
                END) AS new_words
            FROM review_logs rl
            LEFT JOIN first_reviews fr ON fr.card_id = rl.card_id
            WHERE rl.shown_at >= ?1
            GROUP BY day
            ORDER BY day ASC",
        )?;

        let rows = stmt
            .query_map([since_utc], |row| {
                Ok(DayStats {
                    date: row.get(0)?,
                    total_reviews: row.get(1)?,
                    correct_count: row.get(2)?,
                    new_words: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn get_review_dates(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT date(shown_at, 'localtime') AS day
             FROM review_logs
             GROUP BY day
             ORDER BY day ASC",
        )?;

        let dates = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(dates)
    }

    /// Count cards whose first-ever log entry falls on or after `day_start_utc`.
    /// This distinguishes genuinely new words from due-card reviews.
    pub fn count_new_cards_since(&self, day_start_utc: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT card_id) FROM review_logs \
             WHERE shown_at >= ?1 \
               AND result IN ('know', 'dont_know') \
               AND card_id NOT IN ( \
                   SELECT DISTINCT card_id FROM review_logs WHERE shown_at < ?1 \
               )",
            [day_start_utc],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        migration::Migrator,
        repositories::{CardsRepository, WordsRepository},
        Database,
    };
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

        let word_id = words_repo
            .insert("test", "测试", "test", None, None, 1)
            .unwrap();
        let card_id = cards_repo.insert(word_id).unwrap();

        let log_id = logs_repo
            .insert(card_id, "2026-03-12T00:00:00Z", "know", "idle", Some(1500))
            .unwrap();
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

    #[test]
    fn test_get_review_dates_groups_same_day_logs() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_logs_repo_review_dates.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let words_repo = WordsRepository::new(db.get_connection());
        let cards_repo = CardsRepository::new(db.get_connection());
        let logs_repo = LogsRepository::new(db.get_connection());

        let word_id = words_repo
            .insert("test", "测试", "test", None, None, 1)
            .unwrap();
        let card_id = cards_repo.insert(word_id).unwrap();

        logs_repo
            .insert(card_id, "2026-03-10T01:00:00Z", "know", "idle", Some(800))
            .unwrap();
        logs_repo
            .insert(
                card_id,
                "2026-03-10T08:00:00Z",
                "dont_know",
                "idle",
                Some(900),
            )
            .unwrap();
        logs_repo
            .insert(card_id, "2026-03-12T03:00:00Z", "know", "idle", Some(700))
            .unwrap();

        let dates = logs_repo.get_review_dates().unwrap();
        assert_eq!(
            dates,
            vec!["2026-03-10".to_string(), "2026-03-12".to_string()]
        );

        drop(logs_repo);
        drop(cards_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
