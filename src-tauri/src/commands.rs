use std::collections::HashSet;

use base64::Engine;
use chrono::{DateTime, Datelike, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{
    models::{Word, WordWithCard},
    CardsRepository, Database, LogsRepository, StateRepository, WordSourceSummary,
    WordbookImportSummary, WordbookImporter, WordsRepository,
};

const APP_CONFIG_KEY: &str = "app_config";
const ONBOARDING_COMPLETED_KEY: &str = "onboarding_completed";
const FEEDBACK_HISTORY_KEY: &str = "feedback_history";
const WRONG_BOOK_KEY: &str = "wrong_book_card_ids";
const DISABLED_WORDBOOK_SOURCES_KEY: &str = "disabled_wordbook_sources";
const FEEDBACK_HISTORY_LIMIT: usize = 50;
const QUIZ_OPTION_COUNT: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WordQuizMode {
    ZhToEnChoice,
    EnToZhChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordQuizOption {
    pub id: String,
    pub label: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordCardData {
    pub word_id: i64,
    pub card_id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Option<String>,
    pub meaning_zh: String,
    pub quiz_mode: WordQuizMode,
    pub prompt: String,
    pub prompt_hint: Option<String>,
    pub options: Vec<WordQuizOption>,
    pub correct_option_id: String,
    pub explanation_title: String,
    pub explanation_detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodayStats {
    pub total_reviews: i64,
    pub know_count: i64,
    pub dont_know_count: i64,
    pub skip_count: i64,
    pub new_words_today: i64,
    pub due_cards_count: i64,
    pub mastered_count: i64,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    pub mode: String,
    pub using_recommended: bool,
    pub idle_threshold_sec: i64,
    pub fallback_enabled: bool,
    pub fallback_interval_min: i64,
}

impl Default for ReminderConfig {
    fn default() -> Self {
        Self {
            mode: "gentle".to_string(),
            using_recommended: true,
            idle_threshold_sec: 180,
            fallback_enabled: true,
            fallback_interval_min: 45,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub quiet_hours_start: String,
    pub quiet_hours_end: String,
    pub weekday_profile: Option<String>,
    pub weekend_profile: Option<String>,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            quiet_hours_start: "23:00".to_string(),
            quiet_hours_end: "07:00".to_string(),
            weekday_profile: Some("gentle".to_string()),
            weekend_profile: Some("balanced".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub daily_new_limit: i64,
    pub review_first: bool,
    pub allow_new_when_no_due: bool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            daily_new_limit: 10,
            review_first: true,
            allow_new_when_no_due: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardConfig {
    pub auto_hide_sec: i64,
    pub show_phonetic: bool,
    pub reveal_order: String,
    pub allow_skip: bool,
    pub shortcuts_enabled: bool,
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            auto_hide_sec: 10,
            show_phonetic: true,
            reveal_order: "en-first".to_string(),
            allow_skip: true,
            shortcuts_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub launch_at_login: bool,
    pub start_behavior: String,
    pub tray_enabled: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            start_behavior: "show-main".to_string(),
            tray_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub reminder: ReminderConfig,
    pub schedule: ScheduleConfig,
    pub learning: LearningConfig,
    pub card: CardConfig,
    pub system: SystemConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardState {
    pub app_config: AppConfig,
    pub today_stats: TodayStats,
    pub pause_until: Option<String>,
    pub needs_onboarding: bool,
    pub recommendation: RecommendationSummary,
    pub recent_feedback: Vec<FeedbackRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub feedback_type: String,
    pub source: String,
    pub created_at: String,
    pub card_id: Option<i64>,
    pub word: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSummary {
    pub base_mode: String,
    pub suggested_mode: String,
    pub explanation: String,
    pub reasons: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub summary: String,
    pub config: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBundle {
    pub file_name_hint: String,
    pub summary_text: String,
    pub config_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordbookListItem {
    pub source: String,
    pub display_name: String,
    pub total_words: i64,
    pub enabled: bool,
    pub built_in: bool,
    pub first_created_at: Option<String>,
    pub last_created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordbookWordItem {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Option<String>,
    pub meaning_zh: String,
    pub difficulty: i32,
    pub created_at: String,
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn local_day_start(now: DateTime<Local>) -> DateTime<Local> {
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap()
}

fn option_id_for_word(word_id: i64) -> String {
    format!("word-{}", word_id)
}

fn derive_custom_source(file_name: &str) -> String {
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

fn display_name_for_source(source: &str) -> String {
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

fn load_disabled_wordbook_sources(state_repo: &StateRepository) -> Result<HashSet<String>, String> {
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

fn persist_disabled_wordbook_sources(
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

fn is_source_enabled(disabled_sources: &HashSet<String>, source: &str) -> bool {
    !disabled_sources.contains(source)
}

fn filter_active_cards(
    cards: Vec<WordWithCard>,
    disabled_sources: &HashSet<String>,
) -> Vec<WordWithCard> {
    cards
        .into_iter()
        .filter(|item| is_source_enabled(disabled_sources, &item.word.source))
        .collect()
}

fn build_wordbook_list_items(
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

fn clamp_wordbook_preview_limit(limit: i64) -> i64 {
    limit.clamp(1, 50)
}

fn load_wrong_book_set(state_repo: &StateRepository) -> Result<HashSet<i64>, String> {
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

fn persist_wrong_book_set(
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

fn update_wrong_book(
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

fn select_card_candidate(
    due_cards: &[WordWithCard],
    new_cards: &[WordWithCard],
    wrong_book: &HashSet<i64>,
    review_first: bool,
    new_cards_allowed: bool,
) -> Option<WordWithCard> {
    if let Some(card) = due_cards
        .iter()
        .find(|item| wrong_book.contains(&item.card.id))
    {
        return Some(card.clone());
    }

    if new_cards_allowed {
        if let Some(card) = new_cards
            .iter()
            .find(|item| wrong_book.contains(&item.card.id))
        {
            return Some(card.clone());
        }
    }

    if review_first {
        due_cards.first().cloned().or_else(|| {
            if new_cards_allowed {
                new_cards.first().cloned()
            } else {
                None
            }
        })
    } else if new_cards_allowed {
        new_cards
            .first()
            .cloned()
            .or_else(|| due_cards.first().cloned())
    } else {
        due_cards.first().cloned()
    }
}

fn determine_quiz_mode(word_with_card: &WordWithCard) -> WordQuizMode {
    let seed = word_with_card.card.id
        + i64::from(word_with_card.word.difficulty)
        + i64::from(word_with_card.card.lifetime_correct)
        + i64::from(word_with_card.card.lifetime_wrong);

    if seed.rem_euclid(2) == 0 {
        WordQuizMode::ZhToEnChoice
    } else {
        WordQuizMode::EnToZhChoice
    }
}

fn format_explanation_title(word: &Word, show_phonetic: bool) -> String {
    match (show_phonetic, word.phonetic.as_deref()) {
        (true, Some(phonetic)) if !phonetic.is_empty() => format!("{} {}", word.word, phonetic),
        _ => word.word.clone(),
    }
}

fn format_explanation_detail(word: &Word) -> String {
    match word.part_of_speech.as_deref() {
        Some(pos) if !pos.is_empty() => format!("{} · {}", pos, word.meaning_zh),
        _ => word.meaning_zh.clone(),
    }
}

fn build_quiz_options(
    target: &Word,
    distractors: &[Word],
    quiz_mode: &WordQuizMode,
    show_phonetic: bool,
) -> Vec<WordQuizOption> {
    let mut options = Vec::with_capacity(QUIZ_OPTION_COUNT);
    let mut seen_labels = HashSet::new();

    let build_option = |word: &Word| -> WordQuizOption {
        match quiz_mode {
            WordQuizMode::ZhToEnChoice => WordQuizOption {
                id: option_id_for_word(word.id),
                label: word.word.clone(),
                detail: if show_phonetic {
                    word.phonetic.clone().filter(|value| !value.is_empty())
                } else {
                    None
                },
            },
            WordQuizMode::EnToZhChoice => WordQuizOption {
                id: option_id_for_word(word.id),
                label: word.meaning_zh.clone(),
                detail: word
                    .part_of_speech
                    .clone()
                    .filter(|value| !value.is_empty()),
            },
        }
    };

    let correct = build_option(target);
    seen_labels.insert(correct.label.clone());
    options.push(correct);

    for distractor in distractors {
        let option = build_option(distractor);
        if seen_labels.insert(option.label.clone()) {
            options.push(option);
        }

        if options.len() >= QUIZ_OPTION_COUNT {
            break;
        }
    }

    let rotation = (target.id.rem_euclid(options.len() as i64)) as usize;
    options.rotate_left(rotation);
    options
}

fn build_word_card_data(
    word_with_card: WordWithCard,
    distractors: Vec<Word>,
    show_phonetic: bool,
) -> WordCardData {
    let quiz_mode = determine_quiz_mode(&word_with_card);
    let options = build_quiz_options(
        &word_with_card.word,
        &distractors,
        &quiz_mode,
        show_phonetic,
    );

    let (prompt, prompt_hint) = match quiz_mode {
        WordQuizMode::ZhToEnChoice => (
            word_with_card.word.meaning_zh.clone(),
            word_with_card
                .word
                .part_of_speech
                .clone()
                .filter(|value| !value.is_empty()),
        ),
        WordQuizMode::EnToZhChoice => (
            word_with_card.word.word.clone(),
            if show_phonetic {
                word_with_card
                    .word
                    .phonetic
                    .clone()
                    .filter(|value| !value.is_empty())
            } else {
                None
            },
        ),
    };

    WordCardData {
        word_id: word_with_card.word.id,
        card_id: word_with_card.card.id,
        word: word_with_card.word.word.clone(),
        phonetic: if show_phonetic {
            word_with_card.word.phonetic.clone()
        } else {
            None
        },
        part_of_speech: word_with_card.word.part_of_speech.clone(),
        meaning_zh: word_with_card.word.meaning_zh.clone(),
        quiz_mode,
        prompt,
        prompt_hint,
        options,
        correct_option_id: option_id_for_word(word_with_card.word.id),
        explanation_title: format_explanation_title(&word_with_card.word, show_phonetic),
        explanation_detail: format_explanation_detail(&word_with_card.word),
    }
}

fn normalize_app_config(config: AppConfig) -> AppConfig {
    let mut normalized = config;

    normalized.reminder.mode = match normalized.reminder.mode.as_str() {
        "gentle" | "balanced" | "intensive" | "custom" => normalized.reminder.mode,
        _ => "gentle".to_string(),
    };
    normalized.reminder.idle_threshold_sec = normalized.reminder.idle_threshold_sec.clamp(30, 3600);
    normalized.reminder.fallback_interval_min =
        normalized.reminder.fallback_interval_min.clamp(5, 240);

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

fn persist_app_config(
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

fn needs_onboarding(state_repo: &StateRepository) -> Result<bool, String> {
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

fn recommended_mode_label(mode: &str) -> &'static str {
    match mode {
        "balanced" => "平衡",
        "intensive" => "强化",
        _ => "克制",
    }
}

fn feedback_type_label(feedback_type: &str) -> &'static str {
    match feedback_type {
        "too_many_reminders" => "提醒太多",
        "too_few_reminders" => "提醒太少",
        "not_interested_word" => "这张词先别再推",
        _ => "其他反馈",
    }
}

fn current_schedule_mode(config: &AppConfig, now: DateTime<Local>) -> String {
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

fn load_feedback_records(state_repo: &StateRepository) -> Result<Vec<FeedbackRecord>, String> {
    match state_repo
        .get(FEEDBACK_HISTORY_KEY)
        .map_err(|e| format!("Failed to get feedback history: {}", e))?
    {
        Some(raw) => serde_json::from_str::<Vec<FeedbackRecord>>(&raw)
            .map_err(|e| format!("Failed to parse feedback history: {}", e)),
        None => Ok(Vec::new()),
    }
}

fn persist_feedback_records(
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

fn is_feedback_within_days(record: &FeedbackRecord, days: i64, now: DateTime<Local>) -> bool {
    DateTime::parse_from_rfc3339(&record.created_at)
        .map(|created_at| created_at.with_timezone(&Local) >= now - Duration::days(days))
        .unwrap_or(false)
}

fn compute_recommendation(
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
        reasons.push("你最近反馈“提醒太多”，推荐先调回更克制的频率。".to_string());
    } else if too_few_count > too_many_count {
        delta += 1;
        reasons.push("你最近反馈“提醒太少”，推荐适度提高提醒频率。".to_string());
    }

    if uninterested_count >= 3 {
        delta -= 1;
        reasons.push("近期多次出现“这张词先别再推”，说明当前节奏可能偏紧。".to_string());
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

fn build_team_templates() -> Vec<TeamTemplate> {
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

fn build_export_bundle(
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

fn format_date_for_export(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Local).format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}

fn count_today_new_words(logs_repo: &LogsRepository) -> Result<i64, String> {
    let now = Local::now();
    let day_start = local_day_start(now);
    let today_logs = logs_repo
        .get_recent_logs(1000)
        .map_err(|e| format!("Failed to get logs: {}", e))?;

    let card_ids = today_logs
        .into_iter()
        .filter_map(|log| {
            let shown_at = DateTime::parse_from_rfc3339(&log.shown_at).ok()?;
            let shown_local = shown_at.with_timezone(&Local);
            if shown_local < day_start {
                return None;
            }
            if log.result == "know" || log.result == "dont_know" {
                return Some(log.card_id);
            }
            None
        })
        .collect::<HashSet<_>>();

    Ok(card_ids.len() as i64)
}

fn load_today_stats(
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

    let new_words_today = today_logs
        .iter()
        .filter(|log| log.result == "know" || log.result == "dont_know")
        .map(|log| log.card_id)
        .collect::<HashSet<_>>()
        .len() as i64;

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
pub fn import_custom_wordbook(
    db: State<Database>,
    file_name: String,
    content_base64: String,
) -> Result<WordbookImportSummary, String> {
    let source = derive_custom_source(&file_name);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&content_base64)
        .map_err(|e| format!("Failed to decode uploaded wordbook: {}", e))?;
    WordbookImporter::import_from_bytes(&db, &bytes, &source, Some(&file_name))
        .map_err(|e| format!("Failed to import custom wordbook: {}", e))
}

#[tauri::command]
pub fn list_wordbooks(db: State<Database>) -> Result<Vec<WordbookListItem>, String> {
    let conn = db.get_connection();
    let words_repo = WordsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let disabled_sources = load_disabled_wordbook_sources(&state_repo)?;
    let sources = words_repo
        .list_sources()
        .map_err(|e| format!("Failed to list wordbooks: {}", e))?;

    Ok(build_wordbook_list_items(sources, &disabled_sources))
}

#[tauri::command]
pub fn list_wordbook_words(
    db: State<Database>,
    source: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<WordbookWordItem>, String> {
    let conn = db.get_connection();
    let words_repo = WordsRepository::new(conn);
    let safe_limit = clamp_wordbook_preview_limit(limit);
    let safe_offset = offset.max(0);

    words_repo
        .list_by_source(&source, safe_limit, safe_offset)
        .map(|words| {
            words
                .into_iter()
                .map(|word| WordbookWordItem {
                    id: word.id,
                    word: word.word,
                    phonetic: word.phonetic,
                    part_of_speech: word.part_of_speech,
                    meaning_zh: word.meaning_zh,
                    difficulty: word.difficulty,
                    created_at: word.created_at,
                })
                .collect()
        })
        .map_err(|e| format!("Failed to list wordbook words: {}", e))
}

#[tauri::command]
pub fn set_wordbook_enabled(
    db: State<Database>,
    source: String,
    enabled: bool,
) -> Result<Vec<WordbookListItem>, String> {
    let conn = db.get_connection();
    let words_repo = WordsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let mut disabled_sources = load_disabled_wordbook_sources(&state_repo)?;

    if enabled {
        disabled_sources.remove(&source);
    } else {
        disabled_sources.insert(source.clone());
    }

    persist_disabled_wordbook_sources(&state_repo, &disabled_sources)?;
    let sources = words_repo
        .list_sources()
        .map_err(|e| format!("Failed to reload wordbooks: {}", e))?;

    Ok(build_wordbook_list_items(sources, &disabled_sources))
}

#[tauri::command]
pub fn delete_wordbook(
    db: State<Database>,
    source: String,
) -> Result<Vec<WordbookListItem>, String> {
    if source == "ielts-core" {
        return Err("内置词库不能删除，只能停用。".to_string());
    }

    let conn = db.get_connection();
    let words_repo = WordsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let mut disabled_sources = load_disabled_wordbook_sources(&state_repo)?;

    words_repo
        .delete_by_source(&source)
        .map_err(|e| format!("Failed to delete wordbook: {}", e))?;
    disabled_sources.remove(&source);
    persist_disabled_wordbook_sources(&state_repo, &disabled_sources)?;

    let sources = words_repo
        .list_sources()
        .map_err(|e| format!("Failed to reload wordbooks: {}", e))?;

    Ok(build_wordbook_list_items(sources, &disabled_sources))
}

/// 获取下一张要展示的卡片
#[tauri::command]
pub fn get_next_card(db: State<Database>) -> Result<Option<WordCardData>, String> {
    let conn = db.get_connection();
    let cards_repo = CardsRepository::new(conn.clone());
    let logs_repo = LogsRepository::new(conn.clone());
    let words_repo = WordsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let config = load_app_config(&state_repo)?;
    let now = now_rfc3339();
    let wrong_book = load_wrong_book_set(&state_repo)?;
    let disabled_sources = load_disabled_wordbook_sources(&state_repo)?;

    let due_cards = filter_active_cards(
        cards_repo
            .get_due_cards(&now, 24)
            .map_err(|e| format!("Failed to get due cards: {}", e))?,
        &disabled_sources,
    );
    let today_new_count = count_today_new_words(&logs_repo)?;
    let new_cards_allowed =
        config.learning.allow_new_when_no_due && today_new_count < config.learning.daily_new_limit;
    let new_cards = if new_cards_allowed {
        filter_active_cards(
            cards_repo
                .get_new_cards(&now, 24)
                .map_err(|e| format!("Failed to get new cards: {}", e))?,
            &disabled_sources,
        )
    } else {
        Vec::new()
    };

    let selected = select_card_candidate(
        &due_cards,
        &new_cards,
        &wrong_book,
        config.learning.review_first,
        new_cards_allowed,
    );

    match selected {
        Some(word_with_card) => {
            let distractors = words_repo
                .get_distractors(
                    word_with_card.word.id,
                    word_with_card.word.difficulty,
                    (QUIZ_OPTION_COUNT as i64) * 3,
                )
                .map_err(|e| format!("Failed to get distractors: {}", e))?;

            Ok(Some(build_word_card_data(
                word_with_card,
                distractors,
                config.card.show_phonetic,
            )))
        }
        None => Ok(None),
    }
}

/// 提交复习结果
#[tauri::command]
pub fn submit_review(db: State<Database>, card_id: i64, result: String) -> Result<(), String> {
    let conn = db.get_connection();
    let cards_repo = CardsRepository::new(conn.clone());
    let logs_repo = LogsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);

    let now = Utc::now();
    let now_str = now.to_rfc3339();

    // 获取当前卡片
    let mut card = cards_repo
        .get_by_id(card_id)
        .map_err(|e| format!("Failed to get card: {}", e))?
        .ok_or_else(|| "Card not found".to_string())?;

    // 计算新的状态
    match result.as_str() {
        "know" => {
            card.stage += 1;
            card.status = if card.stage >= 5 {
                "mastered".to_string()
            } else {
                "learning".to_string()
            };
            card.correct_streak += 1;
            card.lifetime_correct += 1;
            card.last_result = Some("know".to_string());
            card.last_seen_at = Some(now_str.clone());

            let interval_minutes = match card.stage {
                0 => 10,
                1 => 1440,  // 1 day
                2 => 4320,  // 3 days
                3 => 10080, // 7 days
                4 => 20160, // 14 days
                _ => 0,
            };

            card.due_at = if card.status == "mastered" {
                None
            } else {
                Some((now + Duration::minutes(interval_minutes)).to_rfc3339())
            };
            card.skip_cooldown_until = None;
        }
        "dont_know" => {
            card.stage = std::cmp::max(0, card.stage - 1);
            card.status = "learning".to_string();
            card.correct_streak = 0;
            card.lifetime_wrong += 1;
            card.last_result = Some("dont_know".to_string());
            card.last_seen_at = Some(now_str.clone());

            let interval_minutes = match card.stage {
                0 => 10,
                1 => 1440,
                2 => 4320,
                3 => 10080,
                4 => 20160,
                _ => 10,
            };

            card.due_at = Some((now + Duration::minutes(interval_minutes)).to_rfc3339());
            card.skip_cooldown_until = None;
        }
        "skip" => {
            // 跳过：设置冷却期，不改变其他状态
            card.last_result = Some("skip".to_string());
            card.last_seen_at = Some(now_str.clone());
            card.skip_cooldown_until = Some((now + Duration::minutes(30)).to_rfc3339());
        }
        _ => return Err(format!("Invalid result: {}", result)),
    };

    // 更新卡片
    cards_repo
        .update(&card, &now_str)
        .map_err(|e| format!("Failed to update card: {}", e))?;

    update_wrong_book(&state_repo, card_id, &result)?;

    // 记录日志
    logs_repo
        .insert(card_id, &now_str, &result, "manual", None)
        .map_err(|e| format!("Failed to insert log: {}", e))?;

    Ok(())
}

/// 获取今日统计数据
#[tauri::command]
pub fn get_today_stats(db: State<Database>) -> Result<TodayStats, String> {
    let conn = db.get_connection();
    let logs_repo = LogsRepository::new(conn.clone());
    let cards_repo = CardsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let disabled_sources = load_disabled_wordbook_sources(&state_repo)?;

    load_today_stats(&logs_repo, &cards_repo, &disabled_sources)
}

/// 暂停调度器
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

/// 恢复调度器
#[tauri::command]
pub fn resume_scheduler(db: State<Database>) -> Result<(), String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);

    state_repo
        .delete("pause_until")
        .map_err(|e| format!("Failed to delete pause state: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migration::Migrator, Database};
    use std::env;

    fn sample_word_with_card(
        card_id: i64,
        word_id: i64,
        word: &str,
        meaning_zh: &str,
    ) -> WordWithCard {
        WordWithCard {
            word: Word {
                id: word_id,
                word: word.to_string(),
                phonetic: Some("/test/".to_string()),
                part_of_speech: Some("n.".to_string()),
                meaning_zh: meaning_zh.to_string(),
                source: "test".to_string(),
                difficulty: 2,
                created_at: now_rfc3339(),
            },
            card: crate::db::models::SrsCard {
                id: card_id,
                word_id,
                status: "learning".to_string(),
                stage: 0,
                due_at: Some(now_rfc3339()),
                last_seen_at: None,
                last_result: None,
                correct_streak: 0,
                lifetime_correct: 0,
                lifetime_wrong: 0,
                skip_cooldown_until: None,
                updated_at: now_rfc3339(),
            },
        }
    }

    #[test]
    fn wrong_book_cards_are_prioritized_before_regular_due_cards() {
        let due_cards = vec![
            sample_word_with_card(1, 1, "alpha", "阿尔法"),
            sample_word_with_card(2, 2, "beta", "贝塔"),
        ];
        let new_cards = vec![sample_word_with_card(3, 3, "gamma", "伽马")];
        let wrong_book = HashSet::from([2_i64]);

        let selected = select_card_candidate(&due_cards, &new_cards, &wrong_book, true, true)
            .expect("expected a selected card");

        assert_eq!(selected.card.id, 2);
    }

    #[test]
    fn build_word_card_data_generates_multiple_choice_payload() {
        let target = sample_word_with_card(9, 9, "target", "目标");
        let distractors = vec![
            Word {
                id: 10,
                word: "d1".to_string(),
                phonetic: Some("/d1/".to_string()),
                part_of_speech: Some("n.".to_string()),
                meaning_zh: "干扰1".to_string(),
                source: "test".to_string(),
                difficulty: 2,
                created_at: now_rfc3339(),
            },
            Word {
                id: 11,
                word: "d2".to_string(),
                phonetic: Some("/d2/".to_string()),
                part_of_speech: Some("n.".to_string()),
                meaning_zh: "干扰2".to_string(),
                source: "test".to_string(),
                difficulty: 2,
                created_at: now_rfc3339(),
            },
            Word {
                id: 12,
                word: "d3".to_string(),
                phonetic: Some("/d3/".to_string()),
                part_of_speech: Some("n.".to_string()),
                meaning_zh: "干扰3".to_string(),
                source: "test".to_string(),
                difficulty: 2,
                created_at: now_rfc3339(),
            },
        ];

        let card = build_word_card_data(target, distractors, true);

        assert_eq!(card.options.len(), QUIZ_OPTION_COUNT);
        assert!(card
            .options
            .iter()
            .any(|option| option.id == card.correct_option_id));
        assert!(!card.prompt.is_empty());
        assert!(!card.explanation_title.is_empty());
        assert!(!card.explanation_detail.is_empty());
    }

    #[test]
    fn wrong_book_state_updates_on_review_result() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_wrong_book_state.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let state_repo = StateRepository::new(db.get_connection());
        update_wrong_book(&state_repo, 42, "dont_know").unwrap();
        assert!(load_wrong_book_set(&state_repo).unwrap().contains(&42));

        update_wrong_book(&state_repo, 42, "know").unwrap();
        assert!(!load_wrong_book_set(&state_repo).unwrap().contains(&42));

        drop(state_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
