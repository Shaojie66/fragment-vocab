use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

use crate::db::{Database, repositories::{WordsRepository, CardsRepository}};

#[derive(Debug, Deserialize)]
struct WordbookEntry {
    word: String,
    phonetic: Option<String>,
    part_of_speech: Option<String>,
    meaning_zh: String,
    difficulty: i32,
}

pub struct WordbookImporter;

impl WordbookImporter {
    pub fn import_from_json(db: &Database, json_path: PathBuf, source: &str) -> Result<usize> {
        let json_content = std::fs::read_to_string(&json_path)
            .context(format!("Failed to read wordbook file: {:?}", json_path))?;

        let entries: Vec<WordbookEntry> = serde_json::from_str(&json_content)
            .context("Failed to parse wordbook JSON")?;

        let words_repo = WordsRepository::new(db.get_connection());
        let cards_repo = CardsRepository::new(db.get_connection());

        let mut imported_count = 0;

        for entry in entries {
            // 检查是否已存在
            if words_repo.get_by_word(&entry.word)?.is_some() {
                println!("⚠️  Word '{}' already exists, skipping", entry.word);
                continue;
            }

            // 插入单词
            let word_id = words_repo.insert(
                &entry.word,
                &entry.meaning_zh,
                source,
                entry.phonetic.as_deref(),
                entry.part_of_speech.as_deref(),
                entry.difficulty,
            )?;

            // 创建 SRS 卡片
            cards_repo.insert(word_id)?;

            imported_count += 1;
        }

        println!("✅ Imported {} words from {:?}", imported_count, json_path);
        Ok(imported_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::Migrator;
    use std::env;

    #[test]
    fn test_import_wordbook() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_import.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        // 创建测试词库文件
        let test_json = r#"[
            {
                "word": "test1",
                "phonetic": "/test/",
                "part_of_speech": "n.",
                "meaning_zh": "测试1",
                "difficulty": 1
            },
            {
                "word": "test2",
                "meaning_zh": "测试2",
                "difficulty": 2
            }
        ]"#;

        let json_path = temp_dir.join("test_wordbook.json");
        std::fs::write(&json_path, test_json).unwrap();

        let count = WordbookImporter::import_from_json(&db, json_path.clone(), "test").unwrap();
        assert_eq!(count, 2);

        let words_repo = WordsRepository::new(db.get_connection());
        let total = words_repo.count().unwrap();
        assert_eq!(total, 2);

        let cards_repo = CardsRepository::new(db.get_connection());
        let new_cards = cards_repo.count_by_status("new").unwrap();
        assert_eq!(new_cards, 2);

        // 测试重复导入
        let count2 = WordbookImporter::import_from_json(&db, json_path.clone(), "test").unwrap();
        assert_eq!(count2, 0);

        drop(cards_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(&json_path);
    }
}
