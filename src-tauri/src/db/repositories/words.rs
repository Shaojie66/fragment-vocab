use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::commands::SearchResult;
use crate::db::models::Word;

#[derive(Debug, Clone)]
pub struct WordSourceSummary {
    pub source: String,
    pub total_words: i64,
    pub first_created_at: Option<String>,
    pub last_created_at: Option<String>,
}

pub struct WordsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl WordsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(
        &self,
        word: &str,
        meaning_zh: &str,
        source: &str,
        phonetic: Option<&str>,
        part_of_speech: Option<&str>,
        difficulty: i32,
    ) -> Result<i64> {
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
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at FROM words WHERE id = ?1"
        )?;

        let word = stmt
            .query_row([id], |row| {
                Ok(Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    example_sentence: row.get(5)?,
                    source: row.get(6)?,
                    difficulty: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .optional()?;

        Ok(word)
    }

    pub fn get_by_word(&self, word: &str) -> Result<Option<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at FROM words WHERE word = ?1"
        )?;

        let word = stmt
            .query_row([word], |row| {
                Ok(Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    example_sentence: row.get(5)?,
                    source: row.get(6)?,
                    difficulty: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .optional()?;

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
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at FROM words ORDER BY id LIMIT ?1 OFFSET ?2"
        )?;

        let words = stmt
            .query_map([limit, offset], |row| {
                Ok(Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    example_sentence: row.get(5)?,
                    source: row.get(6)?,
                    difficulty: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(words)
    }

    pub fn get_distractors(
        &self,
        exclude_word_id: i64,
        difficulty: i32,
        limit: i64,
    ) -> Result<Vec<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at
             FROM words
             WHERE id != ?1
             ORDER BY ABS(difficulty - ?2) ASC, RANDOM()
             LIMIT ?3",
        )?;

        let words = stmt
            .query_map((exclude_word_id, difficulty, limit), |row| {
                Ok(Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    example_sentence: row.get(5)?,
                    source: row.get(6)?,
                    difficulty: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(words)
    }

    pub fn list_sources(&self) -> Result<Vec<WordSourceSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT source, COUNT(*) AS total_words, MIN(created_at) AS first_created_at, MAX(created_at) AS last_created_at
             FROM words
             GROUP BY source
             ORDER BY last_created_at DESC, source ASC"
        )?;

        let sources = stmt
            .query_map([], |row| {
                Ok(WordSourceSummary {
                    source: row.get(0)?,
                    total_words: row.get(1)?,
                    first_created_at: row.get(2)?,
                    last_created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sources)
    }

    pub fn list_by_source(&self, source: &str, limit: i64, offset: i64) -> Result<Vec<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at
             FROM words
             WHERE source = ?1
             ORDER BY word COLLATE NOCASE ASC, id ASC
             LIMIT ?2 OFFSET ?3",
        )?;

        let words = stmt
            .query_map((source, limit, offset), |row| {
                Ok(Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    example_sentence: row.get(5)?,
                    source: row.get(6)?,
                    difficulty: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(words)
    }

    pub fn list_all_by_source(&self, source: &str) -> Result<Vec<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at
             FROM words
             WHERE source = ?1
             ORDER BY word COLLATE NOCASE ASC, id ASC",
        )?;

        let words = stmt
            .query_map([source], |row| {
                Ok(Word {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    meaning_zh: row.get(4)?,
                    example_sentence: row.get(5)?,
                    source: row.get(6)?,
                    difficulty: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(words)
    }

    pub fn search_words(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<SearchResult>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query.trim());
        let safe_limit = limit.clamp(1, 50);
        let safe_offset = offset.max(0);
        let mut stmt = conn.prepare(
            "SELECT w.word, w.meaning_zh, w.phonetic, w.part_of_speech, COALESCE(c.status, 'new') AS status, w.source, w.example_sentence
             FROM words w
             LEFT JOIN srs_cards c ON c.word_id = w.id
             WHERE w.word LIKE ?1 COLLATE NOCASE OR w.meaning_zh LIKE ?1
             ORDER BY w.word COLLATE NOCASE ASC, w.id ASC
             LIMIT ?2 OFFSET ?3",
        )?;

        let results = stmt
            .query_map((pattern, safe_limit, safe_offset), |row| {
                Ok(SearchResult {
                    word: row.get(0)?,
                    meaning_zh: row.get(1)?,
                    phonetic: row.get(2)?,
                    part_of_speech: row.get(3)?,
                    status: row.get(4)?,
                    source: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    pub fn delete_by_source(&self, source: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn
            .execute("DELETE FROM words WHERE source = ?1", [source])
            .context("Failed to delete words by source")?;

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migration::Migrator, Database};
    use std::env;

    #[test]
    fn test_insert_and_get_word() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_words_repo.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = WordsRepository::new(db.get_connection());

        let word_id = repo
            .insert(
                "abandon",
                "放弃",
                "ielts-core",
                Some("/əˈbændən/"),
                Some("v."),
                1,
            )
            .unwrap();
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

        repo.insert("word1", "意思1", "test", None, None, 1)
            .unwrap();
        repo.insert("word2", "意思2", "test", None, None, 1)
            .unwrap();
        repo.insert("word3", "意思3", "test", None, None, 1)
            .unwrap();

        let count = repo.count().unwrap();
        assert_eq!(count, 3);

        let words = repo.list(2, 0).unwrap();
        assert_eq!(words.len(), 2);

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_get_distractors_excludes_target_word() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_words_distractors.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = WordsRepository::new(db.get_connection());

        let target_id = repo
            .insert("target", "目标", "test", None, None, 2)
            .unwrap();
        repo.insert("near-1", "近义1", "test", None, None, 2)
            .unwrap();
        repo.insert("near-2", "近义2", "test", None, None, 3)
            .unwrap();
        repo.insert("far-1", "远义1", "test", None, None, 5)
            .unwrap();

        let distractors = repo.get_distractors(target_id, 2, 3).unwrap();
        assert_eq!(distractors.len(), 3);
        assert!(distractors.iter().all(|word| word.id != target_id));

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_list_sources_and_delete_by_source() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_words_sources.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = WordsRepository::new(db.get_connection());
        repo.insert("alpha", "阿尔法", "custom-a", None, None, 1)
            .unwrap();
        repo.insert("beta", "贝塔", "custom-a", None, None, 1)
            .unwrap();
        repo.insert("gamma", "伽马", "custom-b", None, None, 1)
            .unwrap();

        let sources = repo.list_sources().unwrap();
        assert_eq!(sources.len(), 2);
        assert!(sources
            .iter()
            .any(|item| item.source == "custom-a" && item.total_words == 2));

        let deleted = repo.delete_by_source("custom-a").unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(repo.count().unwrap(), 1);

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_list_by_source() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_words_list_by_source.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = WordsRepository::new(db.get_connection());
        repo.insert("zebra", "斑马", "custom-alpha", None, None, 2)
            .unwrap();
        repo.insert(
            "apple",
            "苹果",
            "custom-alpha",
            Some("/ˈæpəl/"),
            Some("n."),
            1,
        )
        .unwrap();
        repo.insert("book", "书", "custom-beta", None, None, 1)
            .unwrap();

        let words = repo.list_by_source("custom-alpha", 10, 0).unwrap();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].word, "apple");
        assert_eq!(words[1].word, "zebra");
        assert!(words.iter().all(|word| word.source == "custom-alpha"));

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_search_words_offset_and_limit_cap() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_words_search_offset.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let repo = WordsRepository::new(db.get_connection());

        for index in 0..60 {
            repo.insert(
                &format!("word-{index:02}"),
                "测试",
                "search-source",
                None,
                None,
                1,
            )
            .unwrap();
        }

        let first_page = repo.search_words("word-", 50, 0).unwrap();
        let second_page = repo.search_words("word-", 50, 50).unwrap();
        let capped_page = repo.search_words("word-", 200, 0).unwrap();

        assert_eq!(first_page.len(), 50);
        assert_eq!(second_page.len(), 10);
        assert_eq!(capped_page.len(), 50);
        assert_eq!(second_page[0].word, "word-50");

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
