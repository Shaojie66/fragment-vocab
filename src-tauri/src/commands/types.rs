use serde::{Deserialize, Serialize};

fn default_theme() -> String {
    "auto".to_string()
}

fn default_true() -> bool {
    true
}

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
    pub example_sentence: Option<String>,
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
pub struct DayStats {
    pub date: String,
    pub total_reviews: i64,
    pub correct_count: i64,
    pub new_words: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakStats {
    pub current_streak: i64,
    pub longest_streak: i64,
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
    #[serde(default = "default_true")]
    pub animations_enabled: bool,
    #[serde(default)]
    pub auto_pronounce: bool,
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            auto_hide_sec: 10,
            show_phonetic: true,
            reveal_order: "en-first".to_string(),
            allow_skip: true,
            shortcuts_enabled: true,
            animations_enabled: true,
            auto_pronounce: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub launch_at_login: bool,
    pub start_behavior: String,
    pub tray_enabled: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            start_behavior: "show-main".to_string(),
            tray_enabled: true,
            theme: default_theme(),
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
    pub current_streak: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub word: String,
    pub meaning_zh: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Option<String>,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrongBookWord {
    pub card_id: i64,
    pub word_id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Option<String>,
    pub meaning_zh: String,
    pub lifetime_wrong: i32,
    pub lifetime_correct: i32,
    pub last_result: Option<String>,
}
