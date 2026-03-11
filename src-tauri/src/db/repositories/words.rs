use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::db::models::Word;

pub struct WordsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl WordsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(&self, word: &str, meaning_zh: &str, source: &str, phonetic: Option<&str>, part_of_speech: Option<&str>, difficulty: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO words (word, phonetic, part_of_speech, meaning_zh, source, difficulty) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (word, phonetic, part_of_speech, meaning_zh, source, difficulty),
        ).context("Failed to insert word")?;
        
        Ok(conn.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: i64) -> Result<Option<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, source, difficulty, created_at FROM words WHERE id = ?1"
        )?;

        let word = stmt.query_row([id], |row| {
            Ok(Word {
                id: row.get(0)?,
                word: row.get(1)?,
                phonetic: row.get(2)?,
                part_of_speech: row.get(3)?,
                meaning_zh: row.get(4)?,
                source: row.get(5)?,
                difficulty: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).optional()?;

        Ok(word)
    }

    pub fn get_by_word(&self, word: &str) -> Result<Option<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, source, difficulty, created_at FROM words WHERE word = ?1"
        )?;

        let word = stmt.query_row([word], |row| {
            Ok(Word {
                id: row.get(0)?,
                word: row.get(1)?,
                phonetic: row.get(2)?,
                part_of_speech: row.get(3)?,
                meaning_zh: row.get(4)?,
                source: row.get(5)?,
                difficulty: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).optional()?;

        Ok(word)
    }

    pub fn count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))?;
        Ok(count)
    }

    #[allow(dead_code)]
    pub fn list(&self, limit: i64, offset: i64) -> Result<Vec<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, source, difficulty, created_at FROM words ORDER BY id LIMIT ?1 OFFSET ?2"
        )?;

        let words = stmt.query_map([limit, offset], |row| {
            Ok(Word {
                id: row.get(0)?,
                word: row.get(1)?,
                phonetic: row.get(2)?,
                part_of_speech: row.get(3)?,
                meaning_zh: row.get(4)?,
                source: row.get(5)?,
                difficulty: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migration::Migrator};
    use std::env;

    #[test]
    fn test_insert_and_get_word() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_words_repo.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = WordsRepository::new(db.get_connection());
        
        let word_id = repo.insert("abandon", "放弃", "ielts-core", Some("/əˈbændən/"), Some("v."), 1).unwrap();
        assert!(word_id > 0);

        let word = repo.get_by_id(word_id).unwrap().unwrap();
        assert_eq!(word.word, "abandon");
        assert_eq!(word.meaning_zh, "放弃");

        let word_by_name = repo.get_by_word("abandon").unwrap().unwrap();
        assert_eq!(word_by_name.id, word_id);

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_count_and_list() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_words_list.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = WordsRepository::new(db.get_connection());
        
        repo.insert("word1", "意思1", "test", None, None, 1).unwrap();
        repo.insert("word2", "意思2", "test", None, None, 1).unwrap();
        repo.insert("word3", "意思3", "test", None, None, 1).unwrap();

        let count = repo.count().unwrap();
        assert_eq!(count, 3);

        let words = repo.list(2, 0).unwrap();
        assert_eq!(words.len(), 2);

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
