use std::collections::HashSet;

use chrono::{Duration, Utc};
use fsrs::{FSRS, MemoryState, DEFAULT_PARAMETERS};
use rusqlite::OptionalExtension;
use tauri::{Emitter, State};

use crate::db::{
    models::{SrsCard, Word, WordWithCard},
    CardsRepository, Database, LogsRepository, StateRepository, WordsRepository,
};

use super::config::load_app_config;
use super::types::*;
use super::utils::*;

/// Default desired retention rate for FSRS scheduling
const DEFAULT_DESIRED_RETENTION: f32 = 0.9;

/// Converts minutes to days for FSRS interval calculation
fn minutes_to_days(minutes: i64) -> f32 {
    minutes as f32 / 1440.0
}

/// Converts days to minutes for database storage
fn days_to_minutes(days: f32) -> i64 {
    (days * 1440.0).round() as i64
}

/// Maps user result to FSRS rating
/// "know" = good (3), "dont_know" = again (1)
fn result_to_rating(result: &str) -> Option<u32> {
    match result {
        "know" => Some(3),      // good
        "dont_know" => Some(1), // again
        "skip" => None,          // skip doesn't change FSRS state
        _ => None,
    }
}

/// Applies FSRS algorithm to calculate next card state
fn apply_fsrs(card: &mut SrsCard, result: &str, now: &str) {
    let rating = match result_to_rating(result) {
        Some(r) => r,
        None => return, // skip doesn't update FSRS state
    };

    let fsrs = match FSRS::new(Some(&DEFAULT_PARAMETERS)) {
        Ok(f) => f,
        Err(_) => return, // FSRS initialization failed
    };

    // Calculate actual days elapsed since last review (not the scheduled interval)
    let days_elapsed: u32 = if card.reviews_count == 0 || card.last_seen_at.is_none() {
        0 // new card or no last_seen_at
    } else {
        // Compute actual elapsed time from last_seen_at to now
        if let (Ok(last_seen), Ok(now_parsed)) = (
            chrono::DateTime::parse_from_rfc3339(card.last_seen_at.as_ref().unwrap()),
            chrono::DateTime::parse_from_rfc3339(now),
        ) {
            let elapsed = now_parsed - last_seen;
            (elapsed.num_minutes() as f32 / 1440.0).max(0.0) as u32
        } else {
            // Fallback to scheduled interval if parsing fails
            minutes_to_days(card.actual_interval.max(1)) as u32
        }
    };

    // Get current memory state from card's stability and difficulty
    let current_memory_state = if card.stability > 0.0 {
        Some(MemoryState {
            stability: card.stability as f32,
            difficulty: card.difficulty.max(1.0).min(10.0) as f32,
        })
    } else {
        None // new card
    };

    // Calculate pre-review retrievability (memory_strength before the update)
    let pre_review_retrievability = if card.stability > 0.0 {
        fsrs.current_retrievability(
            MemoryState {
                stability: card.stability as f32,
                difficulty: card.difficulty.max(1.0).min(10.0) as f32,
            },
            days_elapsed,
            DEFAULT_PARAMETERS[20],
        )
    } else {
        1.0 // new card starts at full strength
    };

    // Get next states from FSRS
    let next_states = match fsrs.next_states(current_memory_state, DEFAULT_DESIRED_RETENTION, days_elapsed) {
        Ok(states) => states,
        Err(_) => return, // FSRS calculation failed
    };

    // Select the appropriate state based on rating
    let (new_stability, new_difficulty, new_interval_days) = match rating {
        1 => (next_states.again.memory.stability, next_states.again.memory.difficulty, next_states.again.interval),
        2 => (next_states.hard.memory.stability, next_states.hard.memory.difficulty, next_states.hard.interval),
        3 => (next_states.good.memory.stability, next_states.good.memory.difficulty, next_states.good.interval),
        4 => (next_states.easy.memory.stability, next_states.easy.memory.difficulty, next_states.easy.interval),
        _ => return,
    };

    // Update card FSRS fields
    card.stability = new_stability as f64;
    card.difficulty = new_difficulty as f64;
    card.reviews_count += 1;
    card.actual_interval = days_to_minutes(new_interval_days);

    // Store pre-review retrievability as memory_strength (reflects state BEFORE this review)
    card.memory_strength = pre_review_retrievability as f64;
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
        example_sentence: word_with_card.word.example_sentence.clone(),
        quiz_mode,
        prompt,
        prompt_hint,
        options,
        correct_option_id: option_id_for_word(word_with_card.word.id),
        explanation_title: format_explanation_title(&word_with_card.word, show_phonetic),
        explanation_detail: format_explanation_detail(&word_with_card.word),
    }
}

fn update_pet_on_review(db: &Database, app: Option<&tauri::AppHandle>) {
    let _ = super::pet::update_pet_after_review(db, app);
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
pub fn get_next_card(db: State<Database>) -> Result<Option<WordCardData>, String> {
    get_next_card_for_db(db.inner())
}

pub fn get_next_card_for_db(db: &Database) -> Result<Option<WordCardData>, String> {
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
    let today_new_count = super::config::count_today_new_words(&logs_repo)?;
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

#[tauri::command]
pub fn submit_review(db: State<Database>, app: tauri::AppHandle, card_id: i64, result: String) -> Result<(), String> {
    submit_review_for_db(db.inner(), Some(&app), card_id, &result)
}

pub fn submit_review_for_db(db: &Database, app: Option<&tauri::AppHandle>, card_id: i64, result: &str) -> Result<(), String> {
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    {
        let conn_arc = db.get_connection();
        let mut conn = conn_arc
            .lock()
            .map_err(|_| "db lock poisoned".to_string())?;
        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut card: SrsCard = {
            let mut stmt = tx
                .prepare(
                    "SELECT id, word_id, status, stage, due_at, last_seen_at, last_result, \
                     correct_streak, lifetime_correct, lifetime_wrong, skip_cooldown_until, \
                     updated_at, stability, difficulty, memory_strength, reviews_count, actual_interval \
                     FROM srs_cards WHERE id = ?1",
                )
                .map_err(|e| format!("Failed to prepare card query: {}", e))?;
            stmt.query_row([card_id], |row| {
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
                    stability: row.get(12)?,
                    difficulty: row.get(13)?,
                    memory_strength: row.get(14)?,
                    reviews_count: row.get(15)?,
                    actual_interval: row.get(16)?,
                })
            })
            .optional()
            .map_err(|e| format!("Failed to get card: {}", e))?
            .ok_or_else(|| "Card not found".to_string())?
        };

        match result {
            "know" => {
                // Apply FSRS algorithm
                apply_fsrs(&mut card, result, &now_str);

                // Update basic tracking fields
                card.stage = (card.stage + 1).max(0); // Maintain stage for compatibility
                card.correct_streak += 1;
                card.lifetime_correct += 1;
                card.last_result = Some("know".to_string());
                card.last_seen_at = Some(now_str.clone());

                // Calculate due_at based on actual_interval (which FSRS has set in minutes)
                if card.actual_interval > 0 {
                    card.due_at = Some(
                        (now + Duration::minutes(card.actual_interval)).to_rfc3339()
                    );
                } else {
                    // Fallback for new cards
                    card.due_at = Some((now + Duration::minutes(10)).to_rfc3339());
                }
                card.skip_cooldown_until = None;

                // Keep status as "learning" so cards remain in the review queue
                // (FSRS handles interval growth naturally; no "mastered" terminal state)
                card.status = "learning".to_string();
            }
            "dont_know" => {
                // Apply FSRS algorithm
                apply_fsrs(&mut card, result, &now_str);

                // Update basic tracking fields
                card.stage = std::cmp::max(0, card.stage) - 1; // FSRS doesn't use stage, but we maintain it for compatibility
                card.correct_streak = 0;
                card.lifetime_wrong += 1;
                card.last_result = Some("dont_know".to_string());
                card.last_seen_at = Some(now_str.clone());

                // Calculate due_at based on actual_interval (FSRS sets this even for "dont_know")
                // But we ensure a minimum of 10 minutes
                let interval = card.actual_interval.max(10);
                card.due_at = Some((now + Duration::minutes(interval)).to_rfc3339());
                card.skip_cooldown_until = None;
                card.status = "learning".to_string();
            }
            "skip" => {
                card.last_result = Some("skip".to_string());
                card.last_seen_at = Some(now_str.clone());
                card.skip_cooldown_until = Some((now + Duration::minutes(30)).to_rfc3339());
            }
            _ => return Err(format!("Invalid result: {}", result)),
        }

        tx.execute(
            "UPDATE srs_cards SET status = ?1, stage = ?2, due_at = ?3, last_seen_at = ?4, \
             last_result = ?5, correct_streak = ?6, lifetime_correct = ?7, lifetime_wrong = ?8, \
             skip_cooldown_until = ?9, updated_at = ?10, \
             stability = ?11, difficulty = ?12, memory_strength = ?13, \
             reviews_count = ?14, actual_interval = ?15 \
             WHERE id = ?16",
            (
                &card.status,
                card.stage,
                &card.due_at,
                &card.last_seen_at,
                &card.last_result,
                card.correct_streak,
                card.lifetime_correct,
                card.lifetime_wrong,
                &card.skip_cooldown_until,
                &now_str,
                card.stability,
                card.difficulty,
                card.memory_strength,
                card.reviews_count,
                card.actual_interval,
                card.id,
            ),
        )
        .map_err(|e| format!("Failed to update card: {}", e))?;

        // Update wrong book
        let wrong_book_raw: Option<String> = tx
            .query_row(
                "SELECT value FROM app_state WHERE key = ?1",
                [WRONG_BOOK_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to read wrong book: {}", e))?;

        let mut wrong_book: HashSet<i64> = wrong_book_raw
            .as_deref()
            .and_then(|s| serde_json::from_str::<Vec<i64>>(s).ok())
            .map(|v| v.into_iter().collect())
            .unwrap_or_default();

        match result {
            "dont_know" => {
                wrong_book.insert(card_id);
            }
            "know" => {
                wrong_book.remove(&card_id);
            }
            _ => {}
        }

        let mut items = wrong_book.iter().copied().collect::<Vec<_>>();
        items.sort_unstable();
        let wrong_book_json = serde_json::to_string(&items)
            .map_err(|e| format!("Failed to serialize wrong book: {}", e))?;

        tx.execute(
            "INSERT OR REPLACE INTO app_state (key, value, updated_at) VALUES (?1, ?2, ?3)",
            [WRONG_BOOK_KEY, &wrong_book_json, &now_str],
        )
        .map_err(|e| format!("Failed to save wrong book: {}", e))?;

        tx.execute(
            "INSERT INTO review_logs (card_id, shown_at, result, trigger_type, response_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (card_id, &now_str, result, "manual", Option::<i32>::None),
        )
        .map_err(|e| format!("Failed to insert log: {}", e))?;

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
    }

    update_pet_on_review(&db, app);
    // Emit study-completed event for pet celebration animation
    if let Some(app_handle) = app {
        let _ = app_handle.emit("study-completed", ());
    }
    if let Err(error) = super::achievements::check_achievements_for_db(db) {
        eprintln!("Failed to check achievements after review: {}", error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migration::Migrator, Database};

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
                example_sentence: Some(format!("This is an example for {word}.")),
                source: "test".to_string(),
                difficulty: 2,
                created_at: now_rfc3339(),
            },
            card: SrsCard {
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
                stability: 0.0,
                difficulty: 5.0,
                memory_strength: 0.0,
                reviews_count: 0,
                actual_interval: 0,
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
                example_sentence: None,
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
                example_sentence: None,
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
                example_sentence: None,
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
        assert_eq!(
            card.example_sentence.as_deref(),
            Some("This is an example for target.")
        );
    }

    #[test]
    fn wrong_book_state_updates_on_review_result() {
        let temp_dir = std::env::temp_dir();
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
