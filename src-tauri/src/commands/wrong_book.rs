use tauri::State;

use crate::db::{CardsRepository, Database, StateRepository};

use super::types::WrongBookWord;
use super::utils::*;

#[tauri::command]
pub fn get_wrong_book_words(db: State<Database>) -> Result<Vec<WrongBookWord>, String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn.clone());
    let cards_repo = CardsRepository::new(conn);

    let wrong_book = load_wrong_book_set(&state_repo)?;
    if wrong_book.is_empty() {
        return Ok(Vec::new());
    }

    let card_ids: Vec<i64> = wrong_book.into_iter().collect();
    let words_with_cards = cards_repo
        .get_words_by_card_ids(&card_ids)
        .map_err(|e| format!("Failed to get wrong book words: {}", e))?;

    Ok(words_with_cards
        .into_iter()
        .map(|item| WrongBookWord {
            card_id: item.card.id,
            word_id: item.word.id,
            word: item.word.word,
            phonetic: item.word.phonetic,
            part_of_speech: item.word.part_of_speech,
            meaning_zh: item.word.meaning_zh,
            lifetime_wrong: item.card.lifetime_wrong,
            lifetime_correct: item.card.lifetime_correct,
            last_result: item.card.last_result,
        })
        .collect())
}

#[tauri::command]
pub fn remove_from_wrong_book(db: State<Database>, card_id: i64) -> Result<(), String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);

    let mut wrong_book = load_wrong_book_set(&state_repo)?;
    wrong_book.remove(&card_id);
    persist_wrong_book_set(&state_repo, &wrong_book)
}
