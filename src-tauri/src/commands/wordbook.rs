use base64::Engine;
use tauri::State;

use crate::db::{
    Database, StateRepository, WordbookImporter, WordbookImportSummary, WordsRepository,
};

use super::types::*;
use super::utils::*;

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
