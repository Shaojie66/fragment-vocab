use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub word_count: i64,
    pub created_at: String,
}

pub struct TagsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl TagsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, name: &str) -> Result<Tag> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tags (name) VALUES (?1)",
            [name],
        )
        .context("Failed to create tag")?;

        let id = conn.last_insert_rowid();
        let tag = conn
            .query_row(
                "SELECT id, name, created_at FROM tags WHERE id = ?1",
                [id],
                |row| {
                    Ok(Tag {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .context("Failed to read created tag")?;

        Ok(tag)
    }

    pub fn delete(&self, tag_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tags WHERE id = ?1", [tag_id])
            .context("Failed to delete tag")?;
        Ok(())
    }

    pub fn list_with_counts(&self) -> Result<Vec<TagWithCount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, COUNT(wt.word_id) AS word_count, t.created_at
             FROM tags t
             LEFT JOIN word_tags wt ON wt.tag_id = t.id
             GROUP BY t.id
             ORDER BY t.name COLLATE NOCASE ASC",
        )?;

        let tags = stmt
            .query_map([], |row| {
                Ok(TagWithCount {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    word_count: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    pub fn add_word_tag(&self, word_id: i64, tag_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO word_tags (word_id, tag_id) VALUES (?1, ?2)",
            (word_id, tag_id),
        )
        .context("Failed to add word tag")?;
        Ok(())
    }

    pub fn remove_word_tag(&self, word_id: i64, tag_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM word_tags WHERE word_id = ?1 AND tag_id = ?2",
            (word_id, tag_id),
        )
        .context("Failed to remove word tag")?;
        Ok(())
    }

    pub fn get_word_tags(&self, word_id: i64) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.created_at
             FROM tags t
             INNER JOIN word_tags wt ON wt.tag_id = t.id
             WHERE wt.word_id = ?1
             ORDER BY t.name COLLATE NOCASE ASC",
        )?;

        let tags = stmt
            .query_map([word_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    pub fn list_words_by_tag(&self, tag_id: i64) -> Result<Vec<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT word_id FROM word_tags WHERE tag_id = ?1",
        )?;

        let word_ids = stmt
            .query_map([tag_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(word_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migration::Migrator, Database};
    use crate::db::repositories::words::WordsRepository;
    use std::env;

    fn setup_test_db(name: &str) -> (Database, std::path::PathBuf) {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join(name);
        let _ = std::fs::remove_file(&db_path);
        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();
        (db, db_path)
    }

    #[test]
    fn test_create_and_list_tags() {
        let (db, db_path) = setup_test_db("test_tags_create.db");
        let repo = TagsRepository::new(db.get_connection());

        let tag = repo.create("vocabulary").unwrap();
        assert_eq!(tag.name, "vocabulary");
        assert!(tag.id > 0);

        let tags = repo.list_with_counts().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "vocabulary");
        assert_eq!(tags[0].word_count, 0);

        drop(repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_add_and_remove_word_tags() {
        let (db, db_path) = setup_test_db("test_tags_word.db");
        let tags_repo = TagsRepository::new(db.get_connection());
        let words_repo = WordsRepository::new(db.get_connection());

        let word_id = words_repo
            .insert("hello", "你好", "test", None, None, 1)
            .unwrap();
        let tag = tags_repo.create("greeting").unwrap();

        tags_repo.add_word_tag(word_id, tag.id).unwrap();

        let word_tags = tags_repo.get_word_tags(word_id).unwrap();
        assert_eq!(word_tags.len(), 1);
        assert_eq!(word_tags[0].name, "greeting");

        let tags_with_counts = tags_repo.list_with_counts().unwrap();
        assert_eq!(tags_with_counts[0].word_count, 1);

        tags_repo.remove_word_tag(word_id, tag.id).unwrap();
        let word_tags = tags_repo.get_word_tags(word_id).unwrap();
        assert!(word_tags.is_empty());

        drop(tags_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_delete_tag_cascades() {
        let (db, db_path) = setup_test_db("test_tags_delete.db");
        let tags_repo = TagsRepository::new(db.get_connection());
        let words_repo = WordsRepository::new(db.get_connection());

        let word_id = words_repo
            .insert("world", "世界", "test", None, None, 1)
            .unwrap();
        let tag = tags_repo.create("basic").unwrap();
        tags_repo.add_word_tag(word_id, tag.id).unwrap();

        tags_repo.delete(tag.id).unwrap();

        let word_tags = tags_repo.get_word_tags(word_id).unwrap();
        assert!(word_tags.is_empty());

        let tags = tags_repo.list_with_counts().unwrap();
        assert!(tags.is_empty());

        drop(tags_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_duplicate_word_tag_ignored() {
        let (db, db_path) = setup_test_db("test_tags_dup.db");
        let tags_repo = TagsRepository::new(db.get_connection());
        let words_repo = WordsRepository::new(db.get_connection());

        let word_id = words_repo
            .insert("test", "测试", "test", None, None, 1)
            .unwrap();
        let tag = tags_repo.create("misc").unwrap();

        tags_repo.add_word_tag(word_id, tag.id).unwrap();
        tags_repo.add_word_tag(word_id, tag.id).unwrap(); // duplicate - should not error

        let tags_with_counts = tags_repo.list_with_counts().unwrap();
        assert_eq!(tags_with_counts[0].word_count, 1);

        drop(tags_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
