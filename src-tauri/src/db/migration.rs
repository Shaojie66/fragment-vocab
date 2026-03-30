use anyhow::{Context, Result};

use crate::db::Database;

pub struct Migrator;

impl Migrator {
    pub fn run_migrations(db: &Database) -> Result<()> {
        // 读取 001_init.sql
        let migration_sql = include_str!("../../migrations/001_init.sql");

        db.execute_migration(migration_sql)
            .context("Failed to run 001_init.sql migration")?;

        // 读取 002_pets.sql
        let pets_migration_sql = include_str!("../../migrations/002_pets.sql");

        db.execute_migration(pets_migration_sql)
            .context("Failed to run 002_pets.sql migration")?;

        Self::add_example_sentence_column(db)
            .context("Failed to run 003_example_sentence.sql migration")?;

        let performance_index_migration_sql =
            include_str!("../../migrations/005_performance_indexes.sql");

        db.execute_migration(performance_index_migration_sql)
            .context("Failed to run 005_performance_indexes.sql migration")?;

        let achievements_migration_sql = include_str!("../../migrations/004_achievements.sql");

        db.execute_migration(achievements_migration_sql)
            .context("Failed to run 004_achievements.sql migration")?;

        println!("✅ Database migrations completed successfully");
        Ok(())
    }

    fn add_example_sentence_column(db: &Database) -> Result<()> {
        let conn = db.get_connection();
        let conn = conn.lock().unwrap();

        let mut stmt = conn.prepare("PRAGMA table_info(words)")?;
        let has_column = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .any(|column| column == "example_sentence");

        drop(stmt);

        if !has_column {
            let migration_sql = include_str!("../../migrations/003_example_sentence.sql");
            conn.execute_batch(migration_sql)
                .context("Failed to add example_sentence column to words table")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_migration_execution() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_migration.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        // 验证表是否创建成功
        let conn = db.get_connection();
        let conn = conn.lock().unwrap();

        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('words', 'srs_cards', 'review_logs', 'app_state', 'achievements')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 5);

        let has_example_sentence: bool = conn
            .prepare("PRAGMA table_info(words)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .any(|column| column == "example_sentence");

        assert!(has_example_sentence);

        let index_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name IN (
                    'idx_words_source',
                    'idx_srs_cards_status_due_at',
                    'idx_srs_cards_word_id',
                    'idx_review_logs_card_id_shown_at'
                )",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(index_count, 4);

        drop(conn);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
