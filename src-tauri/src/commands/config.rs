use std::collections::HashSet;

use chrono::{DateTime, Datelike, Duration, Local, Utc};
use rusqlite::{types::ValueRef, Connection, Row, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use tauri::State;

use crate::db::{
    models::{AppState as AppStateRow, ReviewLog, SrsCard, Word},
    pet_model::PetState,
    CardsRepository, Database, LogsRepository, StateRepository,
};

use super::types::*;
use super::utils::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LearningDataArchive {
    #[serde(default)]
    version: i32,
    #[serde(default)]
    exported_at: String,
    #[serde(default)]
    words: Vec<Word>,
    #[serde(default)]
    srs_cards: Vec<SrsCard>,
    #[serde(default)]
    review_logs: Vec<ReviewLog>,
    #[serde(default)]
    app_state: Vec<AppStateRow>,
    #[serde(default)]
    pets: Vec<PetState>,
    #[serde(default)]
    achievements: Vec<JsonMap<String, JsonValue>>,
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table_name],
        |row| row.get::<_, i64>(0),
    )
    .map(|exists| exists != 0)
    .map_err(|e| format!("Failed to inspect table {}: {}", table_name, e))
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn sql_value_ref_to_json(value: ValueRef<'_>) -> JsonValue {
    match value {
        ValueRef::Null => JsonValue::Null,
        ValueRef::Integer(value) => JsonValue::from(value),
        ValueRef::Real(value) => JsonValue::from(value),
        ValueRef::Text(value) => JsonValue::String(String::from_utf8_lossy(value).into_owned()),
        ValueRef::Blob(value) => JsonValue::Array(
            value
                .iter()
                .map(|byte| JsonValue::from(i64::from(*byte)))
                .collect(),
        ),
    }
}

fn json_value_to_sql(value: &JsonValue) -> rusqlite::types::Value {
    match value {
        JsonValue::Null => rusqlite::types::Value::Null,
        JsonValue::Bool(value) => rusqlite::types::Value::Integer(i64::from(*value)),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                rusqlite::types::Value::Integer(value)
            } else if let Some(value) = value.as_u64() {
                match i64::try_from(value) {
                    Ok(value) => rusqlite::types::Value::Integer(value),
                    Err(_) => rusqlite::types::Value::Text(value.to_string()),
                }
            } else if let Some(value) = value.as_f64() {
                rusqlite::types::Value::Real(value)
            } else {
                rusqlite::types::Value::Text(value.to_string())
            }
        }
        JsonValue::String(value) => rusqlite::types::Value::Text(value.clone()),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            rusqlite::types::Value::Text(value.to_string())
        }
    }
}

fn row_to_json_map(
    row: &Row<'_>,
    columns: &[String],
) -> rusqlite::Result<JsonMap<String, JsonValue>> {
    let mut object = JsonMap::new();

    for (index, column) in columns.iter().enumerate() {
        object.insert(column.clone(), sql_value_ref_to_json(row.get_ref(index)?));
    }

    Ok(object)
}

fn load_words_for_export(conn: &Connection) -> Result<Vec<Word>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at
             FROM words
             ORDER BY id ASC",
        )
        .map_err(|e| format!("Failed to prepare words export: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
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
        .map_err(|e| format!("Failed to export words: {}", e))?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect words: {}", e))?;

    Ok(items)
}

fn load_cards_for_export(conn: &Connection) -> Result<Vec<SrsCard>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, word_id, status, stage, due_at, last_seen_at, last_result, correct_streak,
                    lifetime_correct, lifetime_wrong, skip_cooldown_until, updated_at
             FROM srs_cards
             ORDER BY id ASC",
        )
        .map_err(|e| format!("Failed to prepare srs_cards export: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| format!("Failed to export srs_cards: {}", e))?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect srs_cards: {}", e))?;

    Ok(items)
}

fn load_logs_for_export(conn: &Connection) -> Result<Vec<ReviewLog>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, card_id, shown_at, result, trigger_type, response_ms, created_at
             FROM review_logs
             ORDER BY id ASC",
        )
        .map_err(|e| format!("Failed to prepare review_logs export: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ReviewLog {
                id: row.get(0)?,
                card_id: row.get(1)?,
                shown_at: row.get(2)?,
                result: row.get(3)?,
                trigger_type: row.get(4)?,
                response_ms: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to export review_logs: {}", e))?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect review_logs: {}", e))?;

    Ok(items)
}

fn load_app_state_for_export(conn: &Connection) -> Result<Vec<AppStateRow>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value, updated_at FROM app_state ORDER BY key ASC")
        .map_err(|e| format!("Failed to prepare app_state export: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AppStateRow {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| format!("Failed to export app_state: {}", e))?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect app_state: {}", e))?;

    Ok(items)
}

fn load_pets_for_export(conn: &Connection) -> Result<Vec<PetState>, String> {
    if !table_exists(conn, "pets")? {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, stage, health, experience, current_streak, vitality_multiplier,
                    last_study_at, last_review_at, created_at, updated_at
             FROM pets
             ORDER BY id ASC",
        )
        .map_err(|e| format!("Failed to prepare pets export: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(PetState {
                id: row.get(0)?,
                stage: row.get::<_, i64>(1)? as u8,
                health: row.get(2)?,
                experience: row.get::<_, i64>(3)? as u32,
                current_streak: row.get::<_, i64>(4)? as u32,
                vitality_multiplier: row.get(5)?,
                last_study_at: row.get(6)?,
                last_review_at: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Failed to export pets: {}", e))?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect pets: {}", e))?;

    Ok(items)
}

fn load_generic_table_for_export(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<JsonMap<String, JsonValue>>, String> {
    if !table_exists(conn, table_name)? {
        return Ok(Vec::new());
    }

    let mut pragma_stmt = conn
        .prepare(&format!(
            "PRAGMA table_info({})",
            quote_identifier(table_name)
        ))
        .map_err(|e| format!("Failed to inspect {} columns: {}", table_name, e))?;
    let columns = pragma_stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("Failed to query {} columns: {}", table_name, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect {} columns: {}", table_name, e))?;

    if columns.is_empty() {
        return Ok(Vec::new());
    }

    let select_columns = columns
        .iter()
        .map(|column| quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM {}",
            select_columns,
            quote_identifier(table_name)
        ))
        .map_err(|e| format!("Failed to prepare {} export: {}", table_name, e))?;

    let rows = stmt
        .query_map([], |row| row_to_json_map(row, &columns))
        .map_err(|e| format!("Failed to export {}: {}", table_name, e))?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect {} rows: {}", table_name, e))?;

    Ok(items)
}

fn import_words(tx: &Transaction<'_>, rows: &[Word]) -> Result<i64, String> {
    let mut stmt = tx
        .prepare(
            "INSERT OR REPLACE INTO words
             (id, word, phonetic, part_of_speech, meaning_zh, example_sentence, source, difficulty, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .map_err(|e| format!("Failed to prepare words import: {}", e))?;

    for row in rows {
        stmt.execute((
            row.id,
            &row.word,
            &row.phonetic,
            &row.part_of_speech,
            &row.meaning_zh,
            &row.example_sentence,
            &row.source,
            row.difficulty,
            &row.created_at,
        ))
        .map_err(|e| format!("Failed to import word {}: {}", row.word, e))?;
    }

    Ok(rows.len() as i64)
}

fn import_srs_cards(tx: &Transaction<'_>, rows: &[SrsCard]) -> Result<i64, String> {
    let mut stmt = tx
        .prepare(
            "INSERT OR REPLACE INTO srs_cards
             (id, word_id, status, stage, due_at, last_seen_at, last_result, correct_streak, lifetime_correct,
              lifetime_wrong, skip_cooldown_until, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .map_err(|e| format!("Failed to prepare srs_cards import: {}", e))?;

    for row in rows {
        stmt.execute((
            row.id,
            row.word_id,
            &row.status,
            row.stage,
            &row.due_at,
            &row.last_seen_at,
            &row.last_result,
            row.correct_streak,
            row.lifetime_correct,
            row.lifetime_wrong,
            &row.skip_cooldown_until,
            &row.updated_at,
        ))
        .map_err(|e| format!("Failed to import srs_card {}: {}", row.id, e))?;
    }

    Ok(rows.len() as i64)
}

fn import_review_logs(tx: &Transaction<'_>, rows: &[ReviewLog]) -> Result<i64, String> {
    let mut stmt = tx
        .prepare(
            "INSERT OR REPLACE INTO review_logs
             (id, card_id, shown_at, result, trigger_type, response_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(|e| format!("Failed to prepare review_logs import: {}", e))?;

    for row in rows {
        stmt.execute((
            row.id,
            row.card_id,
            &row.shown_at,
            &row.result,
            &row.trigger_type,
            row.response_ms,
            &row.created_at,
        ))
        .map_err(|e| format!("Failed to import review_log {}: {}", row.id, e))?;
    }

    Ok(rows.len() as i64)
}

fn import_app_state(tx: &Transaction<'_>, rows: &[AppStateRow]) -> Result<i64, String> {
    let mut stmt = tx
        .prepare("INSERT OR REPLACE INTO app_state (key, value, updated_at) VALUES (?1, ?2, ?3)")
        .map_err(|e| format!("Failed to prepare app_state import: {}", e))?;

    for row in rows {
        stmt.execute((&row.key, &row.value, &row.updated_at))
            .map_err(|e| format!("Failed to import app_state {}: {}", row.key, e))?;
    }

    Ok(rows.len() as i64)
}

fn import_pets(tx: &Transaction<'_>, rows: &[PetState]) -> Result<i64, String> {
    if rows.is_empty() || !table_exists(tx, "pets")? {
        return Ok(0);
    }

    let mut stmt = tx
        .prepare(
            "INSERT OR REPLACE INTO pets
             (id, stage, health, experience, current_streak, vitality_multiplier, last_study_at, last_review_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .map_err(|e| format!("Failed to prepare pets import: {}", e))?;

    for row in rows {
        stmt.execute((
            row.id,
            i64::from(row.stage),
            row.health,
            i64::from(row.experience),
            i64::from(row.current_streak),
            row.vitality_multiplier,
            &row.last_study_at,
            &row.last_review_at,
            &row.created_at,
            &row.updated_at,
        ))
        .map_err(|e| format!("Failed to import pet {}: {}", row.id, e))?;
    }

    Ok(rows.len() as i64)
}

fn import_generic_rows(
    tx: &Transaction<'_>,
    table_name: &str,
    rows: &[JsonMap<String, JsonValue>],
) -> Result<i64, String> {
    if rows.is_empty() || !table_exists(tx, table_name)? {
        return Ok(0);
    }

    let mut imported = 0_i64;

    for row in rows {
        if row.is_empty() {
            continue;
        }

        let columns = row.keys().cloned().collect::<Vec<_>>();
        let quoted_columns = columns
            .iter()
            .map(|column| quote_identifier(column))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = (1..=columns.len())
            .map(|index| format!("?{}", index))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT OR REPLACE INTO {} ({}) VALUES ({})",
            quote_identifier(table_name),
            quoted_columns,
            placeholders
        );
        let values = columns
            .iter()
            .filter_map(|column| row.get(column))
            .map(json_value_to_sql)
            .collect::<Vec<_>>();

        tx.execute(&sql, rusqlite::params_from_iter(values))
            .map_err(|e| format!("Failed to import {} row: {}", table_name, e))?;
        imported += 1;
    }

    Ok(imported)
}

pub fn normalize_app_config(config: AppConfig) -> AppConfig {
    let mut normalized = config;

    normalized.reminder.mode = match normalized.reminder.mode.as_str() {
        "gentle" | "balanced" | "intensive" | "custom" => normalized.reminder.mode,
        _ => "gentle".to_string(),
    };
    normalized.reminder.idle_threshold_sec = normalized.reminder.idle_threshold_sec.clamp(5, 3600);
    normalized.reminder.fallback_interval_min =
        normalized.reminder.fallback_interval_min.clamp(1, 240);

    if normalized.schedule.quiet_hours_start.len() != 5 {
        normalized.schedule.quiet_hours_start = "23:00".to_string();
    }
    if normalized.schedule.quiet_hours_end.len() != 5 {
        normalized.schedule.quiet_hours_end = "07:00".to_string();
    }
    normalized.schedule.weekday_profile = match normalized.schedule.weekday_profile.as_deref() {
        Some("gentle" | "balanced" | "intensive") => normalized.schedule.weekday_profile,
        _ => Some("gentle".to_string()),
    };
    normalized.schedule.weekend_profile = match normalized.schedule.weekend_profile.as_deref() {
        Some("gentle" | "balanced" | "intensive") => normalized.schedule.weekend_profile,
        _ => Some("balanced".to_string()),
    };

    normalized.learning.daily_new_limit = normalized.learning.daily_new_limit.clamp(0, 100);
    normalized.card.auto_hide_sec = normalized.card.auto_hide_sec.clamp(5, 60);
    normalized.card.reveal_order = match normalized.card.reveal_order.as_str() {
        "en-first" | "zh-first" => normalized.card.reveal_order,
        _ => "en-first".to_string(),
    };
    normalized.system.start_behavior = match normalized.system.start_behavior.as_str() {
        "show-main" | "minimize-to-tray" => normalized.system.start_behavior,
        _ => "show-main".to_string(),
    };
    normalized.system.theme = match normalized.system.theme.as_str() {
        "auto" | "light" | "dark" => normalized.system.theme,
        _ => "auto".to_string(),
    };

    if !normalized.system.tray_enabled && normalized.system.start_behavior == "minimize-to-tray" {
        normalized.system.start_behavior = "show-main".to_string();
    }

    normalized
}

pub(crate) fn load_app_config(state_repo: &StateRepository) -> Result<AppConfig, String> {
    let now = now_rfc3339();

    match state_repo
        .get(APP_CONFIG_KEY)
        .map_err(|e| format!("Failed to get app config: {}", e))?
    {
        Some(raw) => match serde_json::from_str::<AppConfig>(&raw) {
            Ok(config) => Ok(normalize_app_config(config)),
            Err(_) => {
                let default_config = AppConfig::default();
                let raw_default = serde_json::to_string(&default_config)
                    .map_err(|e| format!("Failed to serialize default app config: {}", e))?;
                state_repo
                    .set(APP_CONFIG_KEY, &raw_default, &now)
                    .map_err(|e| format!("Failed to reset app config: {}", e))?;
                Ok(default_config)
            }
        },
        None => {
            let default_config = AppConfig::default();
            let raw_default = serde_json::to_string(&default_config)
                .map_err(|e| format!("Failed to serialize default app config: {}", e))?;
            state_repo
                .set(APP_CONFIG_KEY, &raw_default, &now)
                .map_err(|e| format!("Failed to persist default app config: {}", e))?;
            Ok(default_config)
        }
    }
}

pub fn persist_app_config(
    state_repo: &StateRepository,
    config: AppConfig,
) -> Result<AppConfig, String> {
    let config = normalize_app_config(config);
    let raw = serde_json::to_string(&config)
        .map_err(|e| format!("Failed to serialize app config: {}", e))?;

    state_repo
        .set(APP_CONFIG_KEY, &raw, &now_rfc3339())
        .map_err(|e| format!("Failed to save app config: {}", e))?;

    Ok(config)
}

pub fn needs_onboarding(state_repo: &StateRepository) -> Result<bool, String> {
    let value = state_repo
        .get(ONBOARDING_COMPLETED_KEY)
        .map_err(|e| format!("Failed to get onboarding state: {}", e))?;

    Ok(value.as_deref() != Some("true"))
}

fn reminder_preset(mode: &str) -> (i64, bool, i64) {
    match mode {
        "balanced" => (120, true, 30),
        "intensive" => (90, true, 20),
        _ => (180, true, 45),
    }
}

fn apply_recommended_mode(mut config: AppConfig, mode: &str) -> AppConfig {
    let (idle_threshold_sec, fallback_enabled, fallback_interval_min) = reminder_preset(mode);
    config.reminder.mode = mode.to_string();
    config.reminder.using_recommended = true;
    config.reminder.idle_threshold_sec = idle_threshold_sec;
    config.reminder.fallback_enabled = fallback_enabled;
    config.reminder.fallback_interval_min = fallback_interval_min;
    config
}

fn recommended_mode_rank(mode: &str) -> i32 {
    match mode {
        "balanced" => 1,
        "intensive" => 2,
        _ => 0,
    }
}

fn rank_to_recommended_mode(rank: i32) -> String {
    match rank.clamp(0, 2) {
        1 => "balanced".to_string(),
        2 => "intensive".to_string(),
        _ => "gentle".to_string(),
    }
}

pub fn recommended_mode_label(mode: &str) -> &'static str {
    match mode {
        "balanced" => "平衡",
        "intensive" => "强化",
        _ => "克制",
    }
}

pub fn feedback_type_label(feedback_type: &str) -> &'static str {
    match feedback_type {
        "too_many_reminders" => "提醒太多",
        "too_few_reminders" => "提醒太少",
        "not_interested_word" => "这张词先别再推",
        _ => "其他反馈",
    }
}

pub fn current_schedule_mode(config: &AppConfig, now: DateTime<Local>) -> String {
    let is_weekend = matches!(now.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun);
    let fallback_mode = match config.reminder.mode.as_str() {
        "gentle" | "balanced" | "intensive" => config.reminder.mode.clone(),
        _ => "gentle".to_string(),
    };
    let candidate = if is_weekend {
        config.schedule.weekend_profile.as_deref()
    } else {
        config.schedule.weekday_profile.as_deref()
    };

    match candidate {
        Some("gentle" | "balanced" | "intensive") => candidate.unwrap().to_string(),
        _ => fallback_mode,
    }
}

fn is_feedback_within_days(record: &FeedbackRecord, days: i64, now: DateTime<Local>) -> bool {
    DateTime::parse_from_rfc3339(&record.created_at)
        .map(|created_at| created_at.with_timezone(&Local) >= now - Duration::days(days))
        .unwrap_or(false)
}

pub fn compute_recommendation(
    app_config: &AppConfig,
    today_stats: &TodayStats,
    pause_until: &Option<String>,
    feedback_records: &[FeedbackRecord],
) -> RecommendationSummary {
    let now = Local::now();
    let base_mode = current_schedule_mode(app_config, now);
    let mut delta = 0;
    let mut reasons = Vec::new();
    let recent_feedback: Vec<_> = feedback_records
        .iter()
        .filter(|record| is_feedback_within_days(record, 7, now))
        .collect();

    let too_many_count = recent_feedback
        .iter()
        .filter(|record| record.feedback_type == "too_many_reminders")
        .count();
    let too_few_count = recent_feedback
        .iter()
        .filter(|record| record.feedback_type == "too_few_reminders")
        .count();
    let uninterested_count = recent_feedback
        .iter()
        .filter(|record| record.feedback_type == "not_interested_word")
        .count();

    if today_stats.total_reviews >= 6 {
        let skip_ratio = today_stats.skip_count as f64 / today_stats.total_reviews as f64;
        if skip_ratio >= 0.35 {
            delta -= 1;
            reasons.push("最近跳过率偏高，系统建议先减少打断感。".to_string());
        }
    }

    if too_many_count > too_few_count {
        delta -= 1;
        reasons.push("你最近反馈\"提醒太多\"，推荐先调回更克制的频率。".to_string());
    } else if too_few_count > too_many_count {
        delta += 1;
        reasons.push("你最近反馈\"提醒太少\"，推荐适度提高提醒频率。".to_string());
    }

    if uninterested_count >= 3 {
        delta -= 1;
        reasons.push("近期多次出现\"这张词先别再推\"，说明当前节奏可能偏紧。".to_string());
    }

    let is_currently_paused = pause_until
        .as_ref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc) > Utc::now())
        .unwrap_or(false);
    if is_currently_paused {
        delta -= 1;
        reasons.push("当前仍处于暂停状态，系统会继续偏向克制策略。".to_string());
    }

    if today_stats.total_reviews >= 8
        && today_stats.accuracy >= 80.0
        && today_stats.skip_count <= 1
        && today_stats.due_cards_count >= 8
    {
        delta += 1;
        reasons.push("最近识别稳定且仍有待复习词，可以适度加快提醒节奏。".to_string());
    }

    let suggested_mode =
        rank_to_recommended_mode(recommended_mode_rank(&base_mode) + delta.clamp(-1, 1));
    let source = if reasons.is_empty() {
        "static"
    } else {
        "adaptive"
    }
    .to_string();
    if reasons.is_empty() {
        reasons.push("先沿用当前时段默认推荐，继续观察你的作答、跳过和暂停反馈。".to_string());
    }

    let explanation = if suggested_mode == base_mode {
        format!(
            "系统建议今天继续使用{}模式。{}",
            recommended_mode_label(&suggested_mode),
            reasons[0]
        )
    } else {
        format!(
            "系统建议今天从{}调整到{}模式。{}",
            recommended_mode_label(&base_mode),
            recommended_mode_label(&suggested_mode),
            reasons[0]
        )
    };

    RecommendationSummary {
        base_mode,
        suggested_mode,
        explanation,
        reasons,
        source,
    }
}

pub fn build_team_templates() -> Vec<TeamTemplate> {
    let mut engineering = apply_recommended_mode(AppConfig::default(), "gentle");
    engineering.schedule.weekday_profile = Some("gentle".to_string());
    engineering.schedule.weekend_profile = Some("gentle".to_string());
    engineering.learning.daily_new_limit = 6;
    engineering.card.auto_hide_sec = 12;

    let mut product = apply_recommended_mode(AppConfig::default(), "balanced");
    product.schedule.weekday_profile = Some("balanced".to_string());
    product.schedule.weekend_profile = Some("balanced".to_string());
    product.learning.daily_new_limit = 10;
    product.card.auto_hide_sec = 10;

    let mut operations = apply_recommended_mode(AppConfig::default(), "balanced");
    operations.schedule.weekday_profile = Some("balanced".to_string());
    operations.schedule.weekend_profile = Some("intensive".to_string());
    operations.learning.daily_new_limit = 12;
    operations.card.auto_hide_sec = 8;

    vec![
        TeamTemplate {
            id: "engineering-focus".to_string(),
            name: "开发团队".to_string(),
            description: "优先减少工作流打断，适合需要长时间专注的同事。".to_string(),
            summary: "工作日和周末都保持克制频率，每日新词较少，自动隐藏略长。".to_string(),
            config: engineering,
        },
        TeamTemplate {
            id: "product-rhythm".to_string(),
            name: "产品/设计".to_string(),
            description: "兼顾会议与碎片学习，频率比默认更积极一点。".to_string(),
            summary: "工作日与周末都使用平衡频率，适合需要在讨论间隙持续接触词汇。".to_string(),
            config: product,
        },
        TeamTemplate {
            id: "ops-coverage".to_string(),
            name: "运营/客服".to_string(),
            description: "面向响应节奏更碎片化的岗位，周末允许更积极的提醒。".to_string(),
            summary: "工作日平衡、周末强化，每日新词更多，自动隐藏更短。".to_string(),
            config: operations,
        },
    ]
}

pub fn build_export_bundle(
    app_config: &AppConfig,
    today_stats: &TodayStats,
    pause_until: &Option<String>,
    recommendation: &RecommendationSummary,
    feedback_records: &[FeedbackRecord],
) -> Result<ExportBundle, String> {
    let feedback_summary = feedback_records
        .iter()
        .take(5)
        .map(|record| {
            format!(
                "{} {}",
                feedback_type_label(&record.feedback_type),
                format_date_for_export(&record.created_at)
            )
        })
        .collect::<Vec<_>>()
        .join(" / ");
    let pause_summary = pause_until
        .as_ref()
        .map(|value| format!("已暂停至 {}", value))
        .unwrap_or_else(|| "未暂停".to_string());
    let summary_text = format!(
        "Fragment Vocab 配置摘要\n\
日期：{}\n\
当前提醒模式：{}\n\
系统建议：{}\n\
静默时间：{} - {}\n\
工作日 / 周末策略：{} / {}\n\
每日新词上限：{}\n\
自动隐藏：{} 秒\n\
托盘：{} | 启动行为：{}\n\
暂停状态：{}\n\
今日学习：{} 次，正确率 {:.0}%，新词 {}，待复习 {}\n\
最近反馈：{}\n\
推荐说明：{}",
        Local::now().format("%Y-%m-%d %H:%M"),
        recommended_mode_label(&current_schedule_mode(app_config, Local::now())),
        recommended_mode_label(&recommendation.suggested_mode),
        app_config.schedule.quiet_hours_start,
        app_config.schedule.quiet_hours_end,
        recommended_mode_label(
            app_config
                .schedule
                .weekday_profile
                .as_deref()
                .unwrap_or("gentle")
        ),
        recommended_mode_label(
            app_config
                .schedule
                .weekend_profile
                .as_deref()
                .unwrap_or("balanced")
        ),
        app_config.learning.daily_new_limit,
        app_config.card.auto_hide_sec,
        if app_config.system.tray_enabled {
            "开启"
        } else {
            "关闭"
        },
        if app_config.system.start_behavior == "show-main" {
            "显示主页面"
        } else {
            "最小化到托盘"
        },
        pause_summary,
        today_stats.total_reviews,
        today_stats.accuracy,
        today_stats.new_words_today,
        today_stats.due_cards_count,
        if feedback_summary.is_empty() {
            "暂无".to_string()
        } else {
            feedback_summary
        },
        recommendation.explanation,
    );
    let config_json = serde_json::to_string_pretty(app_config)
        .map_err(|e| format!("Failed to serialize export config: {}", e))?;

    Ok(ExportBundle {
        file_name_hint: format!("fragment-vocab-config-{}", Local::now().format("%Y-%m-%d")),
        summary_text,
        config_json,
    })
}

pub fn load_feedback_records(state_repo: &StateRepository) -> Result<Vec<FeedbackRecord>, String> {
    match state_repo
        .get(FEEDBACK_HISTORY_KEY)
        .map_err(|e| format!("Failed to get feedback history: {}", e))?
    {
        Some(raw) => serde_json::from_str::<Vec<FeedbackRecord>>(&raw)
            .map_err(|e| format!("Failed to parse feedback history: {}", e)),
        None => Ok(Vec::new()),
    }
}

pub fn persist_feedback_records(
    state_repo: &StateRepository,
    feedback_records: Vec<FeedbackRecord>,
) -> Result<Vec<FeedbackRecord>, String> {
    let trimmed: Vec<_> = feedback_records
        .into_iter()
        .take(FEEDBACK_HISTORY_LIMIT)
        .collect();
    let raw = serde_json::to_string(&trimmed)
        .map_err(|e| format!("Failed to serialize feedback history: {}", e))?;

    state_repo
        .set(FEEDBACK_HISTORY_KEY, &raw, &now_rfc3339())
        .map_err(|e| format!("Failed to save feedback history: {}", e))?;

    Ok(trimmed)
}

pub fn count_today_new_words(logs_repo: &LogsRepository) -> Result<i64, String> {
    let day_start_utc = local_day_start(Local::now())
        .with_timezone(&Utc)
        .to_rfc3339();
    logs_repo
        .count_new_cards_since(&day_start_utc)
        .map_err(|e| format!("Failed to count new words: {}", e))
}

pub fn load_today_stats(
    logs_repo: &LogsRepository,
    cards_repo: &CardsRepository,
    disabled_sources: &HashSet<String>,
) -> Result<TodayStats, String> {
    let today_logs = logs_repo
        .get_recent_logs(1000)
        .map_err(|e| format!("Failed to get logs: {}", e))?;

    let day_start = local_day_start(Local::now());
    let today_logs: Vec<_> = today_logs
        .into_iter()
        .filter(|log| {
            DateTime::parse_from_rfc3339(&log.shown_at)
                .map(|shown_at| shown_at.with_timezone(&Local) >= day_start)
                .unwrap_or(false)
        })
        .collect();

    let total_reviews = today_logs.len() as i64;
    let know_count = today_logs.iter().filter(|log| log.result == "know").count() as i64;
    let dont_know_count = today_logs
        .iter()
        .filter(|log| log.result == "dont_know")
        .count() as i64;
    let skip_count = today_logs.iter().filter(|log| log.result == "skip").count() as i64;

    let accuracy = if know_count + dont_know_count > 0 {
        (know_count as f64 / (know_count + dont_know_count) as f64) * 100.0
    } else {
        0.0
    };

    let day_start_utc = day_start.with_timezone(&Utc).to_rfc3339();
    let new_words_today = logs_repo
        .count_new_cards_since(&day_start_utc)
        .map_err(|e| format!("Failed to count new words today: {}", e))?;

    let due_cards = cards_repo
        .get_due_cards(&now_rfc3339(), 1000)
        .map_err(|e| format!("Failed to get due cards: {}", e))?;
    let due_cards_count = filter_active_cards(due_cards, disabled_sources).len() as i64;

    let mastered_count = cards_repo
        .count_by_status("mastered")
        .map_err(|e| format!("Failed to count mastered cards: {}", e))?;

    Ok(TodayStats {
        total_reviews,
        know_count,
        dont_know_count,
        skip_count,
        new_words_today,
        due_cards_count,
        mastered_count,
        accuracy,
    })
}

pub fn load_streak_stats(logs_repo: &LogsRepository) -> Result<StreakStats, String> {
    let review_dates = logs_repo
        .get_review_dates()
        .map_err(|e| format!("Failed to load review dates: {}", e))?;

    if review_dates.is_empty() {
        return Ok(StreakStats {
            current_streak: 0,
            longest_streak: 0,
        });
    }

    let parsed_dates = review_dates
        .iter()
        .map(|value| {
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|e| format!("Failed to parse review date {}: {}", value, e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut longest_streak = 1_i64;
    let mut running_streak = 1_i64;

    for window in parsed_dates.windows(2) {
        let previous = window[0];
        let current = window[1];

        if current == previous + Duration::days(1) {
            running_streak += 1;
        } else {
            running_streak = 1;
        }

        longest_streak = longest_streak.max(running_streak);
    }

    let today = Local::now().date_naive();
    let mut current_streak = 0_i64;

    if parsed_dates.last().copied() == Some(today) {
        current_streak = 1;

        for window in parsed_dates.windows(2).rev() {
            let previous = window[0];
            let current = window[1];

            if current == previous + Duration::days(1) {
                current_streak += 1;
            } else {
                break;
            }
        }
    }

    Ok(StreakStats {
        current_streak,
        longest_streak,
    })
}

#[tauri::command]
pub fn get_history_stats(db: State<Database>, days: i64) -> Result<Vec<DayStats>, String> {
    if days <= 0 {
        return Err("days must be greater than 0".to_string());
    }

    let logs_repo = LogsRepository::new(db.get_connection());
    let start_day = local_day_start(Local::now() - Duration::days(days - 1));
    let since_utc = start_day.with_timezone(&Utc).to_rfc3339();

    logs_repo
        .get_history_stats(&since_utc)
        .map_err(|e| format!("Failed to load history stats: {}", e))
}

#[tauri::command]
pub fn get_streak_stats(db: State<Database>) -> Result<StreakStats, String> {
    let logs_repo = LogsRepository::new(db.get_connection());
    load_streak_stats(&logs_repo)
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
pub fn get_app_config(db: State<Database>) -> Result<AppConfig, String> {
    let state_repo = StateRepository::new(db.get_connection());
    load_app_config(&state_repo)
}

#[tauri::command]
pub fn update_app_config(db: State<Database>, config: AppConfig) -> Result<AppConfig, String> {
    let state_repo = StateRepository::new(db.get_connection());
    persist_app_config(&state_repo, config)
}

#[tauri::command]
pub fn complete_onboarding(db: State<Database>, config: AppConfig) -> Result<AppConfig, String> {
    let state_repo = StateRepository::new(db.get_connection());
    let config = persist_app_config(&state_repo, config)?;

    state_repo
        .set(ONBOARDING_COMPLETED_KEY, "true", &now_rfc3339())
        .map_err(|e| format!("Failed to save onboarding state: {}", e))?;

    Ok(config)
}

#[tauri::command]
pub fn get_dashboard_state(db: State<Database>) -> Result<DashboardState, String> {
    let conn = db.get_connection();
    let logs_repo = LogsRepository::new(conn.clone());
    let cards_repo = CardsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);

    let app_config = load_app_config(&state_repo)?;
    let disabled_sources = load_disabled_wordbook_sources(&state_repo)?;
    let today_stats = load_today_stats(&logs_repo, &cards_repo, &disabled_sources)?;
    let streak_stats = load_streak_stats(&logs_repo)?;
    let pause_until = state_repo
        .get("pause_until")
        .map_err(|e| format!("Failed to get pause state: {}", e))?;
    let needs_onboarding = needs_onboarding(&state_repo)?;
    let feedback_records = load_feedback_records(&state_repo)?;
    let recommendation =
        compute_recommendation(&app_config, &today_stats, &pause_until, &feedback_records);

    Ok(DashboardState {
        app_config,
        today_stats,
        current_streak: streak_stats.current_streak,
        pause_until,
        needs_onboarding,
        recommendation,
        recent_feedback: feedback_records.into_iter().take(5).collect(),
    })
}

#[tauri::command]
pub fn list_team_templates() -> Vec<TeamTemplate> {
    build_team_templates()
        .into_iter()
        .map(|mut template| {
            template.config = normalize_app_config(template.config);
            template
        })
        .collect()
}

#[tauri::command]
pub fn record_feedback(
    db: State<Database>,
    feedback_type: String,
    source: String,
    card_id: Option<i64>,
    word: Option<String>,
) -> Result<Vec<FeedbackRecord>, String> {
    let feedback_type = match feedback_type.as_str() {
        "too_many_reminders" | "too_few_reminders" | "not_interested_word" => feedback_type,
        _ => return Err(format!("Invalid feedback type: {}", feedback_type)),
    };
    let source = match source.as_str() {
        "console" | "card" => source,
        _ => return Err(format!("Invalid feedback source: {}", source)),
    };

    let state_repo = StateRepository::new(db.get_connection());
    let mut feedback_records = load_feedback_records(&state_repo)?;
    feedback_records.insert(
        0,
        FeedbackRecord {
            feedback_type,
            source,
            created_at: now_rfc3339(),
            card_id,
            word,
        },
    );

    persist_feedback_records(&state_repo, feedback_records)
}

#[tauri::command]
pub fn get_export_bundle(db: State<Database>) -> Result<ExportBundle, String> {
    let conn = db.get_connection();
    let logs_repo = LogsRepository::new(conn.clone());
    let cards_repo = CardsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let app_config = load_app_config(&state_repo)?;
    let disabled_sources = load_disabled_wordbook_sources(&state_repo)?;
    let today_stats = load_today_stats(&logs_repo, &cards_repo, &disabled_sources)?;
    let pause_until = state_repo
        .get("pause_until")
        .map_err(|e| format!("Failed to get pause state: {}", e))?;
    let feedback_records = load_feedback_records(&state_repo)?;
    let recommendation =
        compute_recommendation(&app_config, &today_stats, &pause_until, &feedback_records);

    build_export_bundle(
        &app_config,
        &today_stats,
        &pause_until,
        &recommendation,
        &feedback_records,
    )
}

#[tauri::command]
pub fn export_all_data(db: State<Database>) -> Result<String, String> {
    let conn = db.get_connection();
    let conn = conn.lock().unwrap();
    let archive = LearningDataArchive {
        version: 1,
        exported_at: now_rfc3339(),
        words: load_words_for_export(&conn)?,
        srs_cards: load_cards_for_export(&conn)?,
        review_logs: load_logs_for_export(&conn)?,
        app_state: load_app_state_for_export(&conn)?,
        pets: load_pets_for_export(&conn)?,
        achievements: load_generic_table_for_export(&conn, "achievements")?,
    };

    serde_json::to_string_pretty(&archive)
        .map_err(|e| format!("Failed to serialize learning data export: {}", e))
}

#[tauri::command]
pub fn import_all_data(db: State<Database>, json_data: String) -> Result<ImportSummary, String> {
    let archive: LearningDataArchive = serde_json::from_str(&json_data)
        .map_err(|e| format!("Failed to parse learning data JSON: {}", e))?;

    let conn = db.get_connection();
    let mut conn = conn.lock().unwrap();
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start import transaction: {}", e))?;

    let words = import_words(&tx, &archive.words)?;
    let srs_cards = import_srs_cards(&tx, &archive.srs_cards)?;
    let review_logs = import_review_logs(&tx, &archive.review_logs)?;
    let app_state = import_app_state(&tx, &archive.app_state)?;
    let pets = import_pets(&tx, &archive.pets)?;
    let achievements = import_generic_rows(&tx, "achievements", &archive.achievements)?;

    tx.commit()
        .map_err(|e| format!("Failed to commit imported learning data: {}", e))?;

    Ok(ImportSummary {
        words,
        srs_cards,
        review_logs,
        app_state,
        pets,
        achievements,
        total_imported: words + srs_cards + review_logs + app_state + pets + achievements,
    })
}

#[tauri::command]
pub fn pause_scheduler(db: State<Database>, minutes: i64) -> Result<(), String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);

    let pause_until = Utc::now() + Duration::minutes(minutes);
    let now = now_rfc3339();

    state_repo
        .set("pause_until", &pause_until.to_rfc3339(), &now)
        .map_err(|e| format!("Failed to set pause state: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn resume_scheduler(db: State<Database>) -> Result<(), String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);

    state_repo
        .delete("pause_until")
        .map_err(|e| format!("Failed to delete pause state: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_today_stats(db: State<Database>) -> Result<TodayStats, String> {
    let conn = db.get_connection();
    let logs_repo = LogsRepository::new(conn.clone());
    let cards_repo = CardsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let disabled_sources = load_disabled_wordbook_sources(&state_repo)?;

    load_today_stats(&logs_repo, &cards_repo, &disabled_sources)
}
