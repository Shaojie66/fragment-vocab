use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::db::models::{SrsCard, WordWithCard};

pub struct CardsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl CardsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(&self, word_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO srs_cards (word_id, status, stage) VALUES (?1, 'new', -1)",
            [word_id],
        ).context("Failed to insert srs_card")?;
        
        Ok(conn.last_insert_rowid())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<SrsCard>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word_id, status, stage, due_at, last_seen_at, last_result, correct_streak, lifetime_correct, lifetime_wrong, skip_cooldown_until, updated_at FROM srs_cards WHERE id = ?1"
        )?;

        let card = stmt.query_row([id], |row| {
            Ok(SrsCard {
                id: row.get(0)?,
                word_id: row.get(1)?,
                status: row.get(2)?,
                stage: row.get(3)?,
                due_at: row.get(4)?,
                last_seen_at: row.get(5)?,
                last_result: row.get(6)?,
                correct_streak: row.get(7)?,
                lifetime_correct: row.get(8)?,
                lifetime_wrong: row.get(9)?,
                skip_cooldown_until: row.get(10)?,
                updated_at: row.get(11)?,
            })
        }).optional()?;

        Ok(card)
    }

    pub fn get_by_word_id(&self, word_id: i64) -> Result<Option<SrsCard>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word_id, status, stage, due_at, last_seen_at, last_result, correct_streak, lifetime_correct, lifetime_wrong, skip_cooldown_until, updated_at FROM srs_cards WHERE word_id = ?1"
        )?;

        let card = stmt.query_row([word_id], |row| {
            Ok(SrsCard {
                id: row.get(0)?,
                word_id: row.get(1)?,
                status: row.get(2)?,
                stage: row.get(3)?,
                due_at: row.get(4)?,
                last_seen_at: row.get(5)?,
                last_result: row.get(6)?,
                correct_streak: row.get(7)?,
                lifetime_correct: row.get(8)?,
                lifetime_wrong: row.get(9)?,
                skip_cooldown_until: row.get(10)?,
                updated_at: row.get(11)?,
            })
        }).optional()?;

        Ok(card)
    }

    pub fn update(&self, card: &SrsCard) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE srs_cards SET status = ?1, stage = ?2, due_at = ?3, last_seen_at = ?4, last_result = ?5, correct_streak = ?6, lifetime_correct = ?7, lifetime_wrong = ?8, skip_cooldown_until = ?9, updated_at = datetime('now') WHERE id = ?10",
            (
                &card.status,
                card.stage,
                &card.due_at,
                &card.last_seen_at,
                &card.last_result,
                card.correct_streak,
                card.lifetime_correct,
                card.lifetime_wrong,
                &card.skip_cooldown_until,
                card.id,
            ),
        ).context("Failed to update srs_card")?;
        
        Ok(())
    }

    pub fn count_by_status(&self, status: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM srs_cards WHERE status = ?1",
            [status],
            |row| row.get(0)
        )?;
        Ok(count)
    }

    pub fn get_due_cards(&self, now: &str, limit: i64) -> Result<Vec<WordWithCard>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT w.id, w.word, w.phonetic, w.part_of_speech, w.meaning_zh, w.source, w.difficulty, w.created_at,
                    c.id, c.word_id, c.status, c.stage, c.due_at, c.last_seen_at, c.last_result, c.correct_streak, c.lifetime_correct, c.lifetime_wrong, c.skip_cooldown_until, c.updated_at
             FROM srs_cards c
             JOIN words w ON c.word_id = w.id
             WHERE c.status IN ('learning', 'review') 
               AND c.due_at <= ?1
               AND (c.skip_cooldown_until IS NULL OR c.skip_cooldown_until <= ?1)
             ORDER BY c.due_at ASC
             LIMIT ?2"
        )?;

        let cards = stmt.query_map([now, &limit.to_string()], |row| {
            Ok(WordWithCard {
                word: crate::db::models::Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    source: row.get(5)?,
                    difficulty: row.get(6)?,
                    created_at: row.get(7)?,
                },
                card: SrsCard {
                    id: row.get(8)?,
                    word_id: row.get(9)?,
                    status: row.get(10)?,
                    stage: row.get(11)?,
                    due_at: row.get(12)?,
                    last_seen_at: row.get(13)?,
                    last_result: row.get(14)?,
                    correct_streak: row.get(15)?,
                    lifetime_correct: row.get(16)?,
                    lifetime_wrong: row.get(17)?,
                    skip_cooldown_until: row.get(18)?,
                    updated_at: row.get(19)?,
                },
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(cards)
    }

    pub fn get_new_cards(&self, limit: i64) -> Result<Vec<WordWithCard>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT w.id, w.word, w.phonetic, w.part_of_speech, w.meaning_zh, w.source, w.difficulty, w.created_at,
                    c.id, c.word_id, c.status, c.stage, c.due_at, c.last_seen_at, c.last_result, c.correct_streak, c.lifetime_correct, c.lifetime_wrong, c.skip_cooldown_until, c.updated_at
             FROM srs_cards c
             JOIN words w ON c.word_id = w.id
             WHERE c.status = 'new'
               AND (c.skip_cooldown_until IS NULL OR c.skip_cooldown_until <= datetime('now'))
             ORDER BY w.id ASC
             LIMIT ?1"
        )?;

        let cards = stmt.query_map([limit], |row| {
            Ok(WordWithCard {
                word: crate::db::models::Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    source: row.get(5)?,
                    difficulty: row.get(6)?,
                    created_at: row.get(7)?,
                },
                card: SrsCard {
                    id: row.get(8)?,
                    word_id: row.get(9)?,
                    status: row.get(10)?,
                    stage: row.get(11)?,
                    due_at: row.get(12)?,
                    last_seen_at: row.get(13)?,
                    last_result: row.get(14)?,
                    correct_streak: row.get(15)?,
                    lifetime_correct: row.get(16)?,
                    lifetime_wrong: row.get(17)?,
                    skip_cooldown_until: row.get(18)?,
                    updated_at: row.get(19)?,
                },
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migration::Migrator, repositories::WordsRepository};
    use std::env;

    #[test]
    fn test_insert_and_get_card() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_cards_repo.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let words_repo = WordsRepository::new(db.get_connection());
        let cards_repo = CardsRepository::new(db.get_connection());
        
        let word_id = words_repo.insert("test", "测试", "test", None, None, 1).unwrap();
        let card_id = cards_repo.insert(word_id).unwrap();
        
        let card = cards_repo.get_by_id(card_id).unwrap().unwrap();
        assert_eq!(card.word_id, word_id);
        assert_eq!(card.status, "new");
        assert_eq!(card.stage, -1);

        drop(cards_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_update_card() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_cards_update.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let words_repo = WordsRepository::new(db.get_connection());
        let cards_repo = CardsRepository::new(db.get_connection());
        
        let word_id = words_repo.insert("test", "测试", "test", None, None, 1).unwrap();
        let card_id = cards_repo.insert(word_id).unwrap();
        
        let mut card = cards_repo.get_by_id(card_id).unwrap().unwrap();
        card.status = "learning".to_string();
        card.stage = 0;
        card.correct_streak = 1;
        
        cards_repo.update(&card).unwrap();
        
        let updated_card = cards_repo.get_by_id(card_id).unwrap().unwrap();
        assert_eq!(updated_card.status, "learning");
        assert_eq!(updated_card.stage, 0);
        assert_eq!(updated_card.correct_streak, 1);

        drop(cards_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
