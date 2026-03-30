use crate::db::pet_model::{PetState, EVOLUTION_THRESHOLDS, VITALITY_THRESHOLDS};
use chrono::{DateTime, Local};

/// PetEngine handles all pet state calculations
pub struct PetEngine;

impl PetEngine {
    /// Calculate new health after a study action
    pub fn calculate_health_after_study(current_health: f64, _has_streak: bool) -> f64 {
        let mut health = current_health;
        // Each study adds 0.05 health, capped at 1.0
        health = (health + 0.05).min(1.0);
        health
    }

    /// Calculate health decay based on days since last study/review
    pub fn calculate_health_decay(
        current_health: f64,
        last_study_at: Option<&str>,
        last_review_at: Option<&str>,
    ) -> f64 {
        let now = Local::now();
        let mut health = current_health;

        // Check days since last new word study
        if let Some(last_study) = last_study_at {
            if let Ok(dt) = DateTime::parse_from_rfc3339(last_study) {
                let days = (now - dt.with_timezone(&Local)).num_days();
                if days >= 1 {
                    health -= 0.3; // 1 day no new words: -0.3
                }
            }
        }

        // Check days since last review
        if let Some(last_review) = last_review_at {
            if let Ok(dt) = DateTime::parse_from_rfc3339(last_review) {
                let days = (now - dt.with_timezone(&Local)).num_days();
                if days >= 7 {
                    health -= 0.2; // 7 days no review: -0.2
                }
            }
        }

        // Health can't go below 0
        health.max(0.0)
    }

    /// Calculate vitality multiplier based on streak
    pub fn calculate_vitality_multiplier(streak_days: u32) -> f64 {
        let mut multiplier = 1.0;
        for (threshold, mult) in VITALITY_THRESHOLDS.iter().rev() {
            if streak_days >= *threshold {
                multiplier = *mult;
                break;
            }
        }
        multiplier
    }

    /// Apply streak penalty (half multiplier on broken streak)
    pub fn apply_streak_penalty(current_multiplier: f64) -> f64 {
        (current_multiplier / 2.0).max(1.0)
    }

    /// Calculate experience gained from one study action
    pub fn calculate_experience(multiplier: f64) -> u32 {
        (1.0 * multiplier) as u32
    }

    /// Check if pet should evolve based on experience
    pub fn check_evolution(current_stage: u8, experience: u32) -> Option<u8> {
        if current_stage >= 4 {
            return None; // Already at max stage
        }

        let threshold = EVOLUTION_THRESHOLDS[(current_stage + 1) as usize];
        if experience >= threshold {
            Some(current_stage + 1)
        } else {
            None
        }
    }

    /// Get the saturation and opacity CSS values for current health
    #[allow(dead_code)]
    pub fn get_visual_state(health: f64) -> (u8, u8) {
        // Returns (saturation_percent, opacity_percent)
        match health {
            h if h >= 1.0 => (100, 100),
            h if h >= 0.75 => (80, 90),
            h if h >= 0.5 => (60, 70),
            h if h >= 0.25 => (40, 50),
            _ => (20, 30),
        }
    }

    /// Get slime size in pixels for current stage
    #[allow(dead_code)]
    pub fn get_slime_size(stage: u8) -> u32 {
        match stage {
            0 => 40,  // egg
            1 => 60,  // hatchling
            2 => 90,  // juvenile
            3 => 120, // adult
            _ => 150, // fully evolved
        }
    }

    /// Returns the new streak value based on last study date.
    /// - 0: same day (no change)
    /// - 1: first study or broken streak (start new)
    /// - 2: consecutive day (add 1)
    pub fn update_streak(last_study_at: Option<&str>) -> u32 {
        if let Some(last) = last_study_at {
            if let Ok(dt) = DateTime::parse_from_rfc3339(last) {
                let last_date = dt.date_naive();
                let today = Local::now().date_naive();

                if last_date == today {
                    return 0; // Already studied today
                } else {
                    let days_since = (today - last_date).num_days();
                    if days_since == 1 {
                        return 2; // Consecutive: current_streak + 1
                    }
                    // days_since > 1: broken streak, starts fresh
                }
            }
        }
        1 // No prior study or broken streak, start new
    }

    /// Process a study completion event
    pub fn process_study_action(pet: &mut PetState) {
        let now = chrono::Local::now().to_rfc3339();

        // Update streak based on last study date
        let streak_delta = Self::update_streak(pet.last_study_at.as_deref());
        if streak_delta == 0 {
            // Same day, no streak change
        } else if streak_delta == 2 {
            // Consecutive day: add 1
            pet.current_streak = pet.current_streak.saturating_add(1);
        } else {
            // First study or broken streak: reset to 1
            pet.current_streak = 1;
        }

        // Recalculate vitality multiplier based on updated streak
        pet.vitality_multiplier = Self::calculate_vitality_multiplier(pet.current_streak);

        // Update last study time
        pet.last_study_at = Some(now);

        // Calculate experience gain
        let exp_gain = Self::calculate_experience(pet.vitality_multiplier);
        pet.experience += exp_gain;

        // Update health
        pet.health = Self::calculate_health_after_study(pet.health, pet.current_streak > 0);

        // Check for evolution
        if let Some(new_stage) = Self::check_evolution(pet.stage, pet.experience) {
            pet.stage = new_stage;
        }
    }

    /// Process a daily health check (called on app startup)
    pub fn process_daily_health_check(pet: &mut PetState) {
        let new_health = Self::calculate_health_decay(
            pet.health,
            pet.last_study_at.as_deref(),
            pet.last_review_at.as_deref(),
        );
        pet.health = new_health;

        // Check if streak is broken
        if let Some(last_study) = pet.last_study_at.as_deref() {
            if let Ok(dt) = DateTime::parse_from_rfc3339(last_study) {
                let last_date = dt.date_naive();
                let today = Local::now().date_naive();
                let days_since = (today - last_date).num_days();

                if days_since > 1 {
                    // Streak broken
                    pet.vitality_multiplier = Self::apply_streak_penalty(pet.vitality_multiplier);
                    pet.current_streak = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_calculation() {
        let health = PetEngine::calculate_health_after_study(0.5, false);
        assert_eq!(health, 0.55);

        let health = PetEngine::calculate_health_after_study(1.0, false);
        assert_eq!(health, 1.0); // Capped at 1.0
    }

    #[test]
    fn test_visual_state() {
        assert_eq!(PetEngine::get_visual_state(1.0), (100, 100));
        assert_eq!(PetEngine::get_visual_state(0.8), (80, 90));
        assert_eq!(PetEngine::get_visual_state(0.5), (60, 70));
        assert_eq!(PetEngine::get_visual_state(0.2), (20, 30));
    }

    #[test]
    fn test_slime_size() {
        assert_eq!(PetEngine::get_slime_size(0), 40);
        assert_eq!(PetEngine::get_slime_size(2), 90);
        assert_eq!(PetEngine::get_slime_size(4), 150);
    }

    #[test]
    fn test_evolution() {
        // Stage 0 with 99 XP should not evolve
        assert_eq!(PetEngine::check_evolution(0, 99), None);
        // Stage 0 with 100 XP should evolve to stage 1
        assert_eq!(PetEngine::check_evolution(0, 100), Some(1));
        // Already at max stage
        assert_eq!(PetEngine::check_evolution(4, 10000), None);
    }

    #[test]
    fn test_vitality_multiplier() {
        assert_eq!(PetEngine::calculate_vitality_multiplier(1), 1.0);
        assert_eq!(PetEngine::calculate_vitality_multiplier(7), 1.5);
        assert_eq!(PetEngine::calculate_vitality_multiplier(14), 2.0);
        assert_eq!(PetEngine::calculate_vitality_multiplier(30), 3.0);
    }
}
