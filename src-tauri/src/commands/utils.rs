use std::collections::HashSet;

use chrono::{DateTime, Local, Utc};

use crate::db::{models::WordWithCard, StateRepository, WordSourceSummary};

use super::types::WordbookListItem;

pub const APP_CONFIG_KEY: &str = "app_config";
pub const ONBOARDING_COMPLETED_KEY: &str = "onboarding_completed";
pub const FEEDBACK_HISTORY_KEY: &str = "feedback_history";
pub const WRONG_BOOK_KEY: &str = "wrong_book_card_ids";
pub const DISABLED_WORDBOOK_SOURCES_KEY: &str = "disabled_wordbook_sources";
pub const FEEDBACK_HISTORY_LIMIT: usize = 50;
pub const QUIZ_OPTION_COUNT: usize = 4;

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn local_day_start(now: DateTime<Local>) -> DateTime<Local> {
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap()
}

pub fn option_id_for_word(word_id: i64) -> String {
    format!("word-{}", word_id)
}

pub fn derive_custom_source(file_name: &str) -> String {
    let base_name = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(file_name)
        .trim();

    let normalized = base_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if normalized.is_empty() {
        "custom-upload".to_string()
    } else {
        format!("custom-{}", normalized)
    }
}

pub fn display_name_for_source(source: &str) -> String {
    if source == "ielts-core" {
        return "IELTS Core".to_string();
    }

    source
        .trim_start_matches("custom-")
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn load_disabled_wordbook_sources(
    state_repo: &StateRepository,
) -> Result<HashSet<String>, String> {
    let raw = state_repo
        .get(DISABLED_WORDBOOK_SOURCES_KEY)
        .map_err(|e| format!("Failed to get disabled wordbook sources: {}", e))?;

    match raw {
        Some(value) => serde_json::from_str::<Vec<String>>(&value)
            .map(|items| items.into_iter().collect())
            .map_err(|e| format!("Failed to parse disabled wordbook sources: {}", e)),
        None => Ok(HashSet::new()),
    }
}

pub fn persist_disabled_wordbook_sources(
    state_repo: &StateRepository,
    disabled_sources: &HashSet<String>,
) -> Result<(), String> {
    let mut items = disabled_sources.iter().cloned().collect::<Vec<_>>();
    items.sort();
    let raw = serde_json::to_string(&items)
        .map_err(|e| format!("Failed to serialize disabled wordbook sources: {}", e))?;

    state_repo
        .set(DISABLED_WORDBOOK_SOURCES_KEY, &raw, &now_rfc3339())
        .map_err(|e| format!("Failed to save disabled wordbook sources: {}", e))
}

pub fn is_source_enabled(disabled_sources: &HashSet<String>, source: &str) -> bool {
    !disabled_sources.contains(source)
}

pub fn filter_active_cards(
    cards: Vec<WordWithCard>,
    disabled_sources: &HashSet<String>,
) -> Vec<WordWithCard> {
    cards
        .into_iter()
        .filter(|item| is_source_enabled(disabled_sources, &item.word.source))
        .collect()
}

pub fn build_wordbook_list_items(
    sources: Vec<WordSourceSummary>,
    disabled_sources: &HashSet<String>,
) -> Vec<WordbookListItem> {
    sources
        .into_iter()
        .map(|source| WordbookListItem {
            display_name: display_name_for_source(&source.source),
            enabled: is_source_enabled(disabled_sources, &source.source),
            built_in: source.source == "ielts-core",
            source: source.source,
            total_words: source.total_words,
            first_created_at: source.first_created_at,
            last_created_at: source.last_created_at,
        })
        .collect()
}

pub fn clamp_wordbook_preview_limit(limit: i64) -> i64 {
    limit.clamp(1, 20_000)
}

pub fn clamp_wordbook_search_limit(limit: i64) -> i64 {
    limit.clamp(1, 50)
}

pub fn load_wrong_book_set(state_repo: &StateRepository) -> Result<HashSet<i64>, String> {
    let raw = state_repo
        .get(WRONG_BOOK_KEY)
        .map_err(|e| format!("Failed to get wrong book: {}", e))?;

    match raw {
        Some(value) => serde_json::from_str::<Vec<i64>>(&value)
            .map(|items| items.into_iter().collect())
            .map_err(|e| format!("Failed to parse wrong book: {}", e)),
        None => Ok(HashSet::new()),
    }
}

pub fn persist_wrong_book_set(
    state_repo: &StateRepository,
    wrong_book: &HashSet<i64>,
) -> Result<(), String> {
    let mut items = wrong_book.iter().copied().collect::<Vec<_>>();
    items.sort_unstable();
    let raw = serde_json::to_string(&items)
        .map_err(|e| format!("Failed to serialize wrong book: {}", e))?;

    state_repo
        .set(WRONG_BOOK_KEY, &raw, &now_rfc3339())
        .map_err(|e| format!("Failed to save wrong book: {}", e))
}

#[allow(dead_code)]
pub fn update_wrong_book(
    state_repo: &StateRepository,
    card_id: i64,
    result: &str,
) -> Result<(), String> {
    let mut wrong_book = load_wrong_book_set(state_repo)?;
    match result {
        "dont_know" => {
            wrong_book.insert(card_id);
        }
        "know" => {
            wrong_book.remove(&card_id);
        }
        _ => {}
    }
    persist_wrong_book_set(state_repo, &wrong_book)
}

pub fn format_date_for_export(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Local).format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}
