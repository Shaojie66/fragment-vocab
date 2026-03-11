use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Option<String>,
    pub meaning_zh: String,
    pub source: String,
    pub difficulty: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrsCard {
    pub id: i64,
    pub word_id: i64,
    pub status: String,
    pub stage: i32,
    pub due_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub last_result: Option<String>,
    pub correct_streak: i32,
    pub lifetime_correct: i32,
    pub lifetime_wrong: i32,
    pub skip_cooldown_until: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewLog {
    pub id: i64,
    pub card_id: i64,
    pub shown_at: String,
    pub result: String,
    pub trigger_type: String,
    pub response_ms: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordWithCard {
    pub word: Word,
    pub card: SrsCard,
}
