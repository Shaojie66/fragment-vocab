use anyhow::{Context, Result};

use crate::db::Database;

pub struct Migrator;

impl Migrator {
    pub fn run_migrations(db: &Database) -> Result<()> {
        // 读取 001_init.sql
        let migration_sql = include_str!("../../migrations/001_init.sql");

        db.execute_migration(migration_sql)
            .context("Failed to run 001_init.sql migration")?;

        println!("✅ Database migrations completed successfully");
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
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('words', 'srs_cards', 'review_logs', 'app_state')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 4);

        drop(conn);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
