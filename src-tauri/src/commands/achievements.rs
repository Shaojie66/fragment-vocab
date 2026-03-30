use std::collections::HashMap;

use chrono::Utc;
use rusqlite::params;
use serde::Serialize;
use tauri::State;

use crate::db::{CardsRepository, Database, LogsRepository};

use super::config::load_streak_stats;

#[derive(Debug, Clone, Serialize)]
pub struct AchievementStatus {
    pub achievement_key: String,
    pub title: String,
    pub description: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum AchievementRequirement {
    TotalReviews(i64),
    CurrentStreak(i64),
    MasteredWords(i64),
}

#[derive(Debug, Clone, Copy)]
struct AchievementDefinition {
    key: &'static str,
    title: &'static str,
    description: &'static str,
    requirement: AchievementRequirement,
}

impl AchievementDefinition {
    fn is_unlocked(&self, progress: &AchievementProgress) -> bool {
        match self.requirement {
            AchievementRequirement::TotalReviews(min_reviews) => {
                progress.total_reviews >= min_reviews
            }
            AchievementRequirement::CurrentStreak(min_streak) => {
                progress.current_streak >= min_streak
            }
            AchievementRequirement::MasteredWords(min_words) => {
                progress.mastered_words >= min_words
            }
        }
    }

    fn to_status(&self, unlocked_at: Option<String>) -> AchievementStatus {
        AchievementStatus {
            achievement_key: self.key.to_string(),
            title: self.title.to_string(),
            description: self.description.to_string(),
            unlocked: unlocked_at.is_some(),
            unlocked_at,
        }
    }
}

#[derive(Debug)]
struct AchievementProgress {
    total_reviews: i64,
    current_streak: i64,
    mastered_words: i64,
}

const ACHIEVEMENTS: [AchievementDefinition; 9] = [
    AchievementDefinition {
        key: "first_review",
        title: "First Review",
        description: "Complete your first review",
        requirement: AchievementRequirement::TotalReviews(1),
    },
    AchievementDefinition {
        key: "streak_7",
        title: "7-Day Streak",
        description: "7-day learning streak",
        requirement: AchievementRequirement::CurrentStreak(7),
    },
    AchievementDefinition {
        key: "streak_30",
        title: "30-Day Streak",
        description: "30-day learning streak",
        requirement: AchievementRequirement::CurrentStreak(30),
    },
    AchievementDefinition {
        key: "words_50",
        title: "50 Words Mastered",
        description: "Master 50 words",
        requirement: AchievementRequirement::MasteredWords(50),
    },
    AchievementDefinition {
        key: "words_100",
        title: "100 Words Mastered",
        description: "Master 100 words",
        requirement: AchievementRequirement::MasteredWords(100),
    },
    AchievementDefinition {
        key: "words_500",
        title: "500 Words Mastered",
        description: "Master 500 words",
        requirement: AchievementRequirement::MasteredWords(500),
    },
    AchievementDefinition {
        key: "reviews_100",
        title: "100 Reviews",
        description: "Complete 100 reviews",
        requirement: AchievementRequirement::TotalReviews(100),
    },
    AchievementDefinition {
        key: "reviews_500",
        title: "500 Reviews",
        description: "Complete 500 reviews",
        requirement: AchievementRequirement::TotalReviews(500),
    },
    AchievementDefinition {
        key: "reviews_1000",
        title: "1000 Reviews",
        description: "Complete 1000 reviews",
        requirement: AchievementRequirement::TotalReviews(1000),
    },
];

fn load_progress(db: &Database) -> Result<AchievementProgress, String> {
    let conn = db.get_connection();
    let logs_repo = LogsRepository::new(conn.clone());
    let cards_repo = CardsRepository::new(conn);
    let streak_stats = load_streak_stats(&logs_repo)?;

    Ok(AchievementProgress {
        total_reviews: logs_repo
            .count_all()
            .map_err(|e| format!("Failed to count reviews: {}", e))?,
        current_streak: streak_stats.current_streak,
        mastered_words: cards_repo
            .count_by_status("mastered")
            .map_err(|e| format!("Failed to count mastered words: {}", e))?,
    })
}

fn load_unlocked_map(db: &Database) -> Result<HashMap<String, String>, String> {
    let conn_arc = db.get_connection();
    let conn = conn_arc
        .lock()
        .map_err(|_| "db lock poisoned".to_string())?;
    let mut stmt = conn
        .prepare("SELECT achievement_key, unlocked_at FROM achievements")
        .map_err(|e| format!("Failed to prepare achievements query: {}", e))?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to load achievements: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to parse achievements: {}", e))?;

    Ok(rows.into_iter().collect())
}

pub fn get_achievements_for_db(db: &Database) -> Result<Vec<AchievementStatus>, String> {
    let unlocked_map = load_unlocked_map(db)?;

    Ok(ACHIEVEMENTS
        .iter()
        .map(|achievement| achievement.to_status(unlocked_map.get(achievement.key).cloned()))
        .collect())
}

pub fn check_achievements_for_db(db: &Database) -> Result<Vec<AchievementStatus>, String> {
    let progress = load_progress(db)?;
    let unlocked_at = Utc::now().to_rfc3339();
    let conn = db.get_connection();
    let conn = conn.lock().map_err(|_| "db lock poisoned".to_string())?;
    let mut unlocked = Vec::new();

    for achievement in ACHIEVEMENTS
        .iter()
        .filter(|item| item.is_unlocked(&progress))
    {
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO achievements (achievement_key, unlocked_at) VALUES (?1, ?2)",
                params![achievement.key, &unlocked_at],
            )
            .map_err(|e| format!("Failed to unlock achievement {}: {}", achievement.key, e))?;

        if inserted > 0 {
            unlocked.push(achievement.to_status(Some(unlocked_at.clone())));
        }
    }

    Ok(unlocked)
}

#[tauri::command]
pub fn get_achievements(db: State<Database>) -> Result<Vec<AchievementStatus>, String> {
    get_achievements_for_db(db.inner())
}

#[tauri::command]
pub fn check_and_unlock_achievements(
    db: State<Database>,
) -> Result<Vec<AchievementStatus>, String> {
    check_achievements_for_db(db.inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::review::submit_review_for_db,
        db::{migration::Migrator, CardsRepository, Database, WordsRepository},
    };
    use std::env;

    #[test]
    fn get_achievements_includes_locked_entries() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_achievements_list.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let achievements = get_achievements_for_db(&db).unwrap();
        assert_eq!(achievements.len(), ACHIEVEMENTS.len());
        assert!(achievements.iter().all(|achievement| !achievement.unlocked));

        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn submit_review_unlocks_first_review_achievement() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_achievements_submit_review.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let words_repo = WordsRepository::new(db.get_connection());
        let cards_repo = CardsRepository::new(db.get_connection());
        let word_id = words_repo
            .insert("achievement", "成就", "test", None, None, 1)
            .unwrap();
        let card_id = cards_repo.insert(word_id).unwrap();

        submit_review_for_db(&db, card_id, "know").unwrap();

        let achievements = get_achievements_for_db(&db).unwrap();
        let first_review = achievements
            .iter()
            .find(|achievement| achievement.achievement_key == "first_review")
            .unwrap();

        assert!(first_review.unlocked);
        assert!(first_review.unlocked_at.is_some());

        drop(cards_repo);
        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
