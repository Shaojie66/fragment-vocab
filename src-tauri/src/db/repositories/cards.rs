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
        )
        .context("Failed to insert srs_card")?;

        Ok(conn.last_insert_rowid())
    }

    /// Maps a row (26 columns: 9 Word + 17 SrsCard) to WordWithCard.
    fn map_row_to_word_with_card(row: &rusqlite::Row) -> rusqlite::Result<WordWithCard> {
        Ok(WordWithCard {
            word: crate::db::models::Word {
                id: row.get(0)?,
                word: row.get(1)?,
                phonetic: row.get(2)?,
                part_of_speech: row.get(3)?,
                meaning_zh: row.get(4)?,
                example_sentence: row.get(5)?,
                source: row.get(6)?,
                difficulty: row.get(7)?,
                created_at: row.get(8)?,
            },
            card: SrsCard {
                id: row.get(9)?,
                word_id: row.get(10)?,
                status: row.get(11)?,
                stage: row.get(12)?,
                due_at: row.get(13)?,
                last_seen_at: row.get(14)?,
                last_result: row.get(15)?,
                correct_streak: row.get(16)?,
                lifetime_correct: row.get(17)?,
                lifetime_wrong: row.get(18)?,
                skip_cooldown_until: row.get(19)?,
                updated_at: row.get(20)?,
                // FSRS fields
                stability: row.get(21)?,
                difficulty: row.get(22)?,
                memory_strength: row.get(23)?,
                reviews_count: row.get(24)?,
                actual_interval: row.get(25)?,
            },
        })
    }

    /// Maps a row (17 columns: all SrsCard fields) to SrsCard.
    fn map_row_to_card(row: &rusqlite::Row) -> rusqlite::Result<SrsCard> {
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
            // FSRS fields
            stability: row.get(12)?,
            difficulty: row.get(13)?,
            memory_strength: row.get(14)?,
            reviews_count: row.get(15)?,
            actual_interval: row.get(16)?,
        })
    }

    const CARD_SELECT: &'static str =
        "SELECT id, word_id, status, stage, due_at, last_seen_at, last_result, \
         correct_streak, lifetime_correct, lifetime_wrong, skip_cooldown_until, updated_at, \
         stability, difficulty, memory_strength, reviews_count, actual_interval \
         FROM srs_cards WHERE id = ?1";

    pub fn get_by_id(&self, id: i64) -> Result<Option<SrsCard>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(Self::CARD_SELECT)?;
        let card = stmt
            .query_row([id], Self::map_row_to_card)
            .optional()?;
        Ok(card)
    }

    #[allow(dead_code)]
    pub fn get_by_word_id(&self, word_id: i64) -> Result<Option<SrsCard>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word_id, status, stage, due_at, last_seen_at, last_result, \
             correct_streak, lifetime_correct, lifetime_wrong, skip_cooldown_until, updated_at, \
             stability, difficulty, memory_strength, reviews_count, actual_interval \
             FROM srs_cards WHERE word_id = ?1",
        )?;
        let card = stmt
            .query_row([word_id], Self::map_row_to_card)
            .optional()?;
        Ok(card)
    }

    pub fn update(&self, card: &SrsCard, now: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE srs_cards SET status = ?1, stage = ?2, due_at = ?3, last_seen_at = ?4, \
             last_result = ?5, correct_streak = ?6, lifetime_correct = ?7, lifetime_wrong = ?8, \
             skip_cooldown_until = ?9, updated_at = ?10, \
             stability = ?11, difficulty = ?12, memory_strength = ?13, \
             reviews_count = ?14, actual_interval = ?15 \
             WHERE id = ?16",
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
                now,
                card.stability,
                card.difficulty,
                card.memory_strength,
                card.reviews_count,
                card.actual_interval,
                card.id,
            ),
        )
        .context("Failed to update srs_card")?;
        Ok(())
    }

    pub fn count_by_status(&self, status: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM srs_cards WHERE status = ?1",
            [status],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    const WORD_CARD_SELECT: &'static str =
        "SELECT w.id, w.word, w.phonetic, w.part_of_speech, w.meaning_zh, w.example_sentence, \
         w.source, w.difficulty, w.created_at, \
         c.id, c.word_id, c.status, c.stage, c.due_at, c.last_seen_at, c.last_result, \
         c.correct_streak, c.lifetime_correct, c.lifetime_wrong, c.skip_cooldown_until, c.updated_at, \
         c.stability, c.difficulty, c.memory_strength, c.reviews_count, c.actual_interval \
         FROM srs_cards c \
         JOIN words w ON c.word_id = w.id";

    pub fn get_due_cards(&self, now: &str, limit: i64) -> Result<Vec<WordWithCard>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "{} WHERE c.status = 'learning' AND c.due_at <= ?1 \
             AND (c.skip_cooldown_until IS NULL OR c.skip_cooldown_until <= ?1) \
             ORDER BY c.due_at ASC LIMIT ?2",
            Self::WORD_CARD_SELECT
        );
        let mut stmt = conn.prepare(&sql)?;

        let cards = stmt
            .query_map([now, &limit.to_string()], Self::map_row_to_word_with_card)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cards)
    }

    pub fn get_new_cards(&self, now: &str, limit: i64) -> Result<Vec<WordWithCard>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "{} WHERE c.status = 'new' \
             AND (c.skip_cooldown_until IS NULL OR c.skip_cooldown_until <= ?1) \
             ORDER BY w.difficulty ASC, w.id ASC LIMIT ?2",
            Self::WORD_CARD_SELECT
        );
        let mut stmt = conn.prepare(&sql)?;

        let cards = stmt
            .query_map([now, &limit.to_string()], Self::map_row_to_word_with_card)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cards)
    }

    /// Get words with cards by a set of card IDs.
    /// Used for the wrong book feature.
    pub fn get_words_by_card_ids(&self, card_ids: &[i64]) -> Result<Vec<WordWithCard>> {
        if card_ids.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().unwrap();
        let placeholders = card_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "{} WHERE c.id IN ({}) ORDER BY c.lifetime_wrong DESC, c.updated_at DESC",
            Self::WORD_CARD_SELECT,
            placeholders
        );
        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<Box<dyn rusqlite::ToSql>> = card_ids
            .iter()
            .map(|id| Box::new(*id) as Box<dyn rusqlite::ToSql>)
            .collect();

        let cards = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), Self::map_row_to_word_with_card)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migration::Migrator, repositories::WordsRepository, Database};
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

        let word_id = words_repo
            .insert("test", "测试", "test", None, None, 1)
            .unwrap();
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

        let word_id = words_repo
            .insert("test", "测试", "test", None, None, 1)
            .unwrap();
        let card_id = cards_repo.insert(word_id).unwrap();

        let mut card = cards_repo.get_by_id(card_id).unwrap().unwrap();
        card.status = "learning".to_string();
        card.stage = 0;
        card.correct_streak = 1;
        // FSRS fields
        card.stability = 1.0;
        card.difficulty = 5.0;
        card.memory_strength = 0.5;
        card.reviews_count = 1;
        card.actual_interval = 10;

        let now = "2026-03-12T02:00:00Z";
        cards_repo.update(&card, now).unwrap();

        let updated_card = cards_repo.get_by_id(card_id).unwrap().unwrap();
        assert_eq!(updated_card.status, "learning");
        assert_eq!(updated_card.stage, 0);
        assert_eq!(updated_card.correct_streak, 1);
        assert_eq!(updated_card.stability, 1.0);
        assert_eq!(updated_card.difficulty, 5.0);

        drop(cards_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
