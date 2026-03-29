use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetState {
    pub id: i64,
    pub stage: u8,           // 0-4 (egg, hatchling, juvenile, adult, fully-evolved)
    pub health: f64,         // 0.0 - 1.0
    pub experience: u32,     // Cumulative experience
    pub current_streak: u32, // Consecutive study days
    pub vitality_multiplier: f64, // 1.0 - 3.0
    pub last_study_at: Option<String>, // ISO timestamp
    pub last_review_at: Option<String>, // ISO timestamp
    pub created_at: String,
    pub updated_at: String,
}

impl Default for PetState {
    fn default() -> Self {
        Self {
            id: 1,
            stage: 0,
            health: 1.0,
            experience: 0,
            current_streak: 0,
            vitality_multiplier: 1.0,
            last_study_at: None,
            last_review_at: None,
            created_at: chrono::Local::now().to_rfc3339(),
            updated_at: chrono::Local::now().to_rfc3339(),
        }
    }
}

/// Evolution thresholds (cumulative experience required to reach next stage)
pub const EVOLUTION_THRESHOLDS: [u32; 5] = [0, 100, 300, 1000, 3000];

/// Stage names for display
pub const STAGE_NAMES: [&str; 5] = ["蛋", "幼体", "青少年", "成体", "完全体"];

/// Vitality multiplier thresholds (consecutive days)
pub const VITALITY_THRESHOLDS: [(u32, f64); 4] = [
    (1, 1.0),  // 1 day = 1.0x
    (7, 1.5),  // 7 days = 1.5x
    (14, 2.0), // 14 days = 2.0x
    (30, 3.0), // 30 days = 3.0x
];
