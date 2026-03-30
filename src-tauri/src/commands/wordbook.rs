use base64::Engine;
use csv::Writer;
use rusqlite::OptionalExtension;
use serde::Serialize;
use tauri::State;

use crate::db::{
    Database, StateRepository, WordbookImportSummary, WordbookImporter, WordsRepository,
};

use super::types::*;
use super::utils::*;

#[derive(Serialize)]
struct ExportWordbookItem<'a> {
    word: &'a str,
    phonetic: &'a str,
    part_of_speech: &'a str,
    meaning_zh: &'a str,
    example_sentence: &'a str,
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
    list_wordbooks_for_db(db.inner())
}

pub fn list_wordbooks_for_db(db: &Database) -> Result<Vec<WordbookListItem>, String> {
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
pub fn export_wordbook(
    db: State<Database>,
    source: String,
    format: String,
) -> Result<String, String> {
    let conn = db.get_connection();
    let words_repo = WordsRepository::new(conn);
    let words = words_repo
        .list_all_by_source(&source)
        .map_err(|e| format!("Failed to load wordbook for export: {}", e))?;

    let normalized_format = format.trim().to_ascii_lowercase();
    match normalized_format.as_str() {
        "csv" => serialize_wordbook_csv(&words)
            .map_err(|e| format!("Failed to export wordbook as CSV: {}", e)),
        "json" => serialize_wordbook_json(&words)
            .map_err(|e| format!("Failed to export wordbook as JSON: {}", e)),
        _ => Err(format!(
            "Unsupported export format: {}. Expected csv or json.",
            format
        )),
    }
}

#[tauri::command]
pub fn search_words(
    db: State<Database>,
    query: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<SearchResult>, String> {
    let normalized_query = query.trim();

    if normalized_query.is_empty() {
        return Ok(Vec::new());
    }

    let conn = db.get_connection();
    let words_repo = WordsRepository::new(conn);
    let safe_limit = clamp_wordbook_search_limit(limit);
    let safe_offset = offset.max(0);

    words_repo
        .search_words(normalized_query, safe_limit, safe_offset)
        .map_err(|e| format!("Failed to search words: {}", e))
}

#[tauri::command]
pub fn get_word_detail(db: State<Database>, word_id: i64) -> Result<WordDetail, String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn.clone());
    let wrong_book = load_wrong_book_set(&state_repo)?;
    let conn = conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT
                w.word,
                w.phonetic,
                w.part_of_speech,
                w.meaning_zh,
                w.example_sentence,
                w.source,
                w.difficulty,
                COALESCE(c.status, 'new') AS srs_status,
                COALESCE(c.stage, -1) AS srs_stage,
                COALESCE(c.correct_streak, 0) AS correct_streak,
                COALESCE(c.lifetime_correct, 0) AS lifetime_correct,
                COALESCE(c.lifetime_wrong, 0) AS lifetime_wrong,
                c.due_at,
                c.id
             FROM words w
             LEFT JOIN srs_cards c ON c.word_id = w.id
             WHERE w.id = ?1",
        )
        .map_err(|e| format!("Failed to prepare word detail query: {}", e))?;

    let detail = stmt
        .query_row([word_id], |row| {
            let card_id: Option<i64> = row.get(13)?;

            Ok(WordDetail {
                word: row.get(0)?,
                phonetic: row.get(1)?,
                part_of_speech: row.get(2)?,
                meaning_zh: row.get(3)?,
                example_sentence: row.get(4)?,
                source: row.get(5)?,
                difficulty: row.get(6)?,
                srs_status: row.get(7)?,
                srs_stage: row.get(8)?,
                correct_streak: row.get(9)?,
                lifetime_correct: row.get(10)?,
                lifetime_wrong: row.get(11)?,
                due_at: row.get(12)?,
                in_wrong_book: card_id.is_some_and(|id| wrong_book.contains(&id)),
            })
        })
        .optional()
        .map_err(|e| format!("Failed to fetch word detail: {}", e))?;

    detail.ok_or_else(|| format!("Word {} not found", word_id))
}

#[tauri::command]
pub fn set_wordbook_enabled(
    db: State<Database>,
    source: String,
    enabled: bool,
) -> Result<Vec<WordbookListItem>, String> {
    set_wordbook_enabled_for_db(db.inner(), &source, enabled)
}

pub fn set_wordbook_enabled_for_db(
    db: &Database,
    source: &str,
    enabled: bool,
) -> Result<Vec<WordbookListItem>, String> {
    let conn = db.get_connection();
    let words_repo = WordsRepository::new(conn.clone());
    let state_repo = StateRepository::new(conn);
    let mut disabled_sources = load_disabled_wordbook_sources(&state_repo)?;

    if enabled {
        disabled_sources.remove(source);
    } else {
        disabled_sources.insert(source.to_string());
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
    delete_wordbook_for_db(db.inner(), &source)
}

pub fn delete_wordbook_for_db(
    db: &Database,
    source: &str,
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
    disabled_sources.remove(source);
    persist_disabled_wordbook_sources(&state_repo, &disabled_sources)?;

    let sources = words_repo
        .list_sources()
        .map_err(|e| format!("Failed to reload wordbooks: {}", e))?;

    Ok(build_wordbook_list_items(sources, &disabled_sources))
}

fn serialize_wordbook_csv(words: &[crate::db::models::Word]) -> Result<String, csv::Error> {
    let mut writer = Writer::from_writer(Vec::new());
    writer.write_record([
        "word",
        "phonetic",
        "part_of_speech",
        "meaning_zh",
        "example_sentence",
    ])?;

    for item in words {
        writer.serialize(build_export_wordbook_item(item))?;
    }

    let bytes = writer.into_inner().map_err(|e| e.into_error())?;
    Ok(String::from_utf8(bytes).expect("CSV writer should only produce valid UTF-8"))
}

fn serialize_wordbook_json(words: &[crate::db::models::Word]) -> Result<String, serde_json::Error> {
    let items = words
        .iter()
        .map(build_export_wordbook_item)
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&items)
}

fn build_export_wordbook_item(word: &crate::db::models::Word) -> ExportWordbookItem<'_> {
    ExportWordbookItem {
        word: &word.word,
        phonetic: word.phonetic.as_deref().unwrap_or(""),
        part_of_speech: word.part_of_speech.as_deref().unwrap_or(""),
        meaning_zh: &word.meaning_zh,
        example_sentence: word.example_sentence.as_deref().unwrap_or(""),
    }
}
