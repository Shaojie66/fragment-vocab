use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Reader};
use csv::ReaderBuilder;
use encoding_rs::GBK;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::db::{
    repositories::{CardsRepository, WordsRepository},
    Database,
};

fn default_difficulty() -> i32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
struct WordbookEntry {
    #[serde(
        alias = "english",
        alias = "term",
        alias = "vocab",
        alias = "单词",
        alias = "词汇",
        alias = "英文",
        alias = "eng",
        alias = "name"
    )]
    word: String,
    #[serde(
        alias = "pronunciation",
        alias = "ipa",
        alias = "音标",
        alias = "发音",
        alias = "phonetic_symbol"
    )]
    phonetic: Option<String>,
    #[serde(
        alias = "pos",
        alias = "词性",
        alias = "词类",
        alias = "type",
        alias = "class"
    )]
    part_of_speech: Option<String>,
    #[serde(
        alias = "meaning",
        alias = "translation",
        alias = "definition",
        alias = "chinese",
        alias = "中文",
        alias = "释义",
        alias = "词义",
        alias = "翻译",
        alias = "解释"
    )]
    meaning_zh: String,
    #[serde(
        default = "default_difficulty",
        alias = "level",
        alias = "rank",
        alias = "难度",
        alias = "等级"
    )]
    difficulty: i32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonWordbookPayload {
    Entries(Vec<WordbookEntry>),
    Wrapped {
        #[serde(alias = "entries", alias = "items", alias = "vocabulary", alias = "data", alias = "list")]
        words: Vec<WordbookEntry>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordbookImportSummary {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub total_count: usize,
    pub source: String,
    pub format: String,
}

// ============================================================================
// Encoding Detection
// ============================================================================

fn detect_encoding_and_convert(raw_bytes: &[u8]) -> Result<(String, String)> {
    // UTF-8 BOM
    if raw_bytes.len() >= 3 && raw_bytes[0] == 0xEF && raw_bytes[1] == 0xBB && raw_bytes[2] == 0xBF {
        let content = String::from_utf8_lossy(raw_bytes).trim().to_string();
        return Ok(("UTF-8".to_string(), content));
    }

    // Try UTF-8
    if let Ok(content) = std::str::from_utf8(raw_bytes) {
        let trimmed = content.trim();
        return Ok(("UTF-8".to_string(), trimmed.to_string()));
    }

    // Try GBK
    if raw_bytes.windows(2).any(|w| {
        let bytes = [w[0], w[1]];
        (bytes[0] >= 0xB0 && bytes[0] <= 0xF7 && bytes[1] >= 0xA1 && bytes[1] <= 0xFE)
            || (bytes[0] >= 0x81 && bytes[0] <= 0xFE && bytes[1] >= 0x40 && bytes[1] <= 0xFE)
    }) {
        let (decoded, _, had_errors) = GBK.decode(raw_bytes);
        if !had_errors {
            return Ok(("GBK".to_string(), decoded.trim().to_string()));
        }
    }

    // Fallback
    let content = String::from_utf8_lossy(raw_bytes).trim().to_string();
    Ok(("UTF-8-Lossy".to_string(), content))
}

// ============================================================================
// Format Detection
// ============================================================================

fn detect_format(file_name: Option<&str>, raw_content: Option<&str>) -> Result<String> {
    if let Some(name) = file_name {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".json") || lower.ends_with(".jsonl") {
            return Ok("json".to_string());
        }
        if lower.ends_with(".csv") {
            return Ok("csv".to_string());
        }
        if lower.ends_with(".txt") || lower.ends_with(".text") {
            return Ok("txt".to_string());
        }
        if lower.ends_with(".xlsx") || lower.ends_with(".xls") {
            return Ok("xlsx".to_string());
        }
        if lower.ends_with(".tsv") {
            return Ok("tsv".to_string());
        }
    }

    if let Some(content) = raw_content {
        let trimmed = content.trim_start();

        if trimmed.starts_with('[') || trimmed.starts_with('{') {
            return Ok("json".to_string());
        }

        let lines: Vec<&str> = trimmed.lines().take(3).collect();
        if lines.len() >= 2 {
            let commas1 = lines[0].matches(',').count();
            let commas2 = lines[1].matches(',').count();
            if commas1 >= 2 && commas1 == commas2 {
                return Ok("csv".to_string());
            }
        }

        if lines.iter().any(|l| l.contains('\t')) {
            return Ok("tsv".to_string());
        }

        if lines.iter().any(|l| l.contains(" - ") || l.contains(':') || l.contains('：')) {
            return Ok("txt".to_string());
        }
    }

    Err(anyhow::anyhow!(
        "Unsupported wordbook format. Use JSON, CSV, TXT, TSV or XLSX."
    ))
}

// ============================================================================
// Header Field Matching
// ============================================================================

fn normalize_header(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || ('\u{4e00}'..='\u{9fff}').contains(c))
        .collect()
}

fn match_field_type(header: &str) -> Option<&'static str> {
    let normalized = normalize_header(header);

    match normalized.as_str() {
        // Word
        "word" | "english" | "term" | "vocab" | "vocabulary" => Some("word"),
        "单词" | "词汇" | "英文" | "英语" | "eng" => Some("word"),

        // Meaning
        "meaning_zh" | "meaning" | "translation" | "definition" | "chinese" => Some("meaning_zh"),
        "中文" | "释义" | "词义" | "翻译" | "解释" => Some("meaning_zh"),

        // Phonetic
        "phonetic" | "pronunciation" | "ipa" | "音标" | "发音" => Some("phonetic"),

        // POS
        "part_of_speech" | "pos" | "词性" | "词类" => Some("part_of_speech"),

        // Difficulty
        "difficulty" | "level" | "rank" | "难度" | "等级" => Some("difficulty"),

        _ => None,
    }
}

// ============================================================================
// TXT Parsing
// ============================================================================

fn detect_txt_separator(content: &str) -> &'static str {
    let lines: Vec<&str> = content.lines().take(10).collect();

    let tab_count = lines.iter().map(|l| l.matches('\t').count()).sum::<usize>();
    let arrow_count = lines.iter().map(|l| l.matches(" - ").count()).sum::<usize>();
    let colon_count = lines.iter().map(|l| l.matches(':').count()).sum::<usize>();
    let cn_colon_count = lines.iter().map(|l| l.matches('：').count()).sum::<usize>();

    if tab_count >= 2 {
        return "\t";
    }
    if cn_colon_count >= arrow_count && cn_colon_count >= colon_count {
        return "：";
    }
    if arrow_count >= 2 {
        return " - ";
    }
    if colon_count >= 2 {
        return ":";
    }
    ","
}

fn parse_txt(content: &str) -> Vec<WordbookEntry> {
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let separator = detect_txt_separator(content);
    let sep_char = match separator {
        "\t" => '\t',
        "：" => '：',
        " - " => ' ',
        ":" => ':',
        _ => ',',
    };

    // Check first line for header
    let first_line_parts: Vec<&str> = if separator == " - " {
        lines[0].split(" - ").map(str::trim).collect()
    } else {
        lines[0].split(sep_char).map(str::trim).collect()
    };

    let has_header = first_line_parts.iter().any(|p| match_field_type(p).is_some());
    let start_idx = if has_header { 1 } else { 0 };

    // Determine column mapping
    let mut word_idx = 0usize;
    let mut meaning_idx = 1usize;
    let mut phonetic_idx = 2usize;
    let mut pos_idx = 3usize;
    let mut diff_idx = 4usize;

    if has_header {
        word_idx = 0;
        meaning_idx = 1;
        phonetic_idx = 2;
        pos_idx = 3;
        diff_idx = 4;

        for (idx, part) in first_line_parts.iter().enumerate() {
            match match_field_type(part) {
                Some("word") => word_idx = idx,
                Some("meaning_zh") => meaning_idx = idx,
                Some("phonetic") => phonetic_idx = idx,
                Some("part_of_speech") => pos_idx = idx,
                Some("difficulty") => diff_idx = idx,
                _ => {}
            }
        }
    }

    let mut entries = Vec::new();

    for line in lines.iter().skip(start_idx) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Try the detected separator first, then fall back to other separators
        let parts: Vec<&str> = if separator == " - " {
            if let Some(pos) = trimmed.find(" - ") {
                vec![&trimmed[..pos].trim(), &trimmed[pos + 3..].trim()]
            } else {
                // Fall back to tab or colon
                let tab_parts: Vec<&str> = trimmed.split('\t').map(str::trim).filter(|s| !s.is_empty()).collect();
                if tab_parts.len() >= 2 {
                    tab_parts
                } else if trimmed.contains(':') {
                    trimmed.split(':').map(str::trim).filter(|s| !s.is_empty()).collect()
                } else {
                    continue;
                }
            }
        } else {
            let primary_parts: Vec<&str> = line.split(sep_char).map(str::trim).filter(|s| !s.is_empty()).collect();
            if primary_parts.len() >= 2 {
                primary_parts
            } else if trimmed.find(" - ").is_some() {
                // Fall back to " - " separator
                if let Some(pos) = trimmed.find(" - ") {
                    vec![&trimmed[..pos].trim(), &trimmed[pos + 3..].trim()]
                } else {
                    continue;
                }
            } else if sep_char == '\t' && trimmed.contains(':') {
                // Fall back to colon
                trimmed.split(':').map(str::trim).filter(|s| !s.is_empty()).collect()
            } else {
                continue;
            }
        };

        if parts.len() < 2 {
            continue;
        }

        let word = parts.get(word_idx).unwrap_or(&"").trim();
        let meaning_zh = parts.get(meaning_idx).unwrap_or(&"").trim();

        if word.is_empty() || meaning_zh.is_empty() {
            continue;
        }

        let phonetic = parts.get(phonetic_idx).and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
        });
        let part_of_speech = parts.get(pos_idx).and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
        });
        let difficulty = parts
            .get(diff_idx)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);

        entries.push(WordbookEntry {
            word: word.to_string(),
            meaning_zh: meaning_zh.to_string(),
            phonetic,
            part_of_speech,
            difficulty,
        });
    }

    entries
}

// ============================================================================
// CSV Parsing
// ============================================================================

fn parse_csv(content: &str) -> Vec<WordbookEntry> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(content.as_bytes());

    let records: Vec<Vec<String>> = reader
        .records()
        .filter_map(|r| r.ok())
        .map(|r| r.iter().map(|s| s.trim().to_string()).collect())
        .collect();

    if records.is_empty() {
        return Vec::new();
    }

    // Check first row for header
    let first_row = &records[0];
    let mut word_idx = 0usize;
    let mut meaning_idx = 1usize;
    let mut phonetic_idx = 2usize;
    let mut pos_idx = 3usize;
    let mut diff_idx = 4usize;
    let mut has_header = false;

    for (idx, cell) in first_row.iter().enumerate() {
        if let Some(field) = match_field_type(cell) {
            has_header = true;
            match field {
                "word" => word_idx = idx,
                "meaning_zh" => meaning_idx = idx,
                "phonetic" => phonetic_idx = idx,
                "part_of_speech" => pos_idx = idx,
                "difficulty" => diff_idx = idx,
                _ => {}
            }
        }
    }

    let start_idx = if has_header { 1 } else { 0 };

    let mut entries = Vec::new();

    for record in records.iter().skip(start_idx) {
        let word = record.get(word_idx).map(|s| s.as_str()).unwrap_or("").trim();
        let meaning_zh = record.get(meaning_idx).map(|s| s.as_str()).unwrap_or("").trim();

        if word.is_empty() || meaning_zh.is_empty() {
            continue;
        }

        let phonetic = record.get(phonetic_idx).filter(|s| !s.is_empty()).cloned();
        let part_of_speech = record.get(pos_idx).filter(|s| !s.is_empty()).cloned();
        let difficulty = record
            .get(diff_idx)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);

        entries.push(WordbookEntry {
            word: word.to_string(),
            meaning_zh: meaning_zh.to_string(),
            phonetic,
            part_of_speech,
            difficulty,
        });
    }

    entries
}

// ============================================================================
// JSON Parsing
// ============================================================================

fn parse_json(content: &str) -> Result<Vec<WordbookEntry>> {
    let payload: JsonWordbookPayload =
        serde_json::from_str(content).context("Failed to parse wordbook JSON")?;

    let entries = match payload {
        JsonWordbookPayload::Entries(entries) => entries,
        JsonWordbookPayload::Wrapped { words } => words,
    };

    Ok(entries)
}

// ============================================================================
// XLSX Parsing
// ============================================================================

fn parse_xlsx(raw_bytes: &[u8], file_name: Option<&str>) -> Result<Vec<WordbookEntry>> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let extension = file_name
        .and_then(|name| name.rsplit_once('.').map(|(_, ext)| ext))
        .unwrap_or("xlsx");
    let unique_name = format!(
        "fragment-vocab-upload-{}.{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        extension
    );
    let temp_path = std::env::temp_dir().join(unique_name);

    fs::write(&temp_path, raw_bytes).context("Failed to stage XLSX wordbook")?;

    let result = (|| -> Result<Vec<WordbookEntry>> {
        let mut workbook =
            open_workbook_auto(&temp_path).context("Failed to open XLSX workbook")?;
        let sheet_name = workbook
            .sheet_names()
            .first()
            .cloned()
            .context("XLSX workbook has no sheets")?;
        let range = workbook
            .worksheet_range(&sheet_name)
            .context("Failed to read first sheet")?;

        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect())
            .collect();

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut word_idx = 0usize;
        let mut meaning_idx = 1usize;
        let mut phonetic_idx = 2usize;
        let mut pos_idx = 3usize;
        let mut diff_idx = 4usize;
        let mut has_header = false;

        for (idx, cell) in rows[0].iter().enumerate() {
            if let Some(field) = match_field_type(cell) {
                has_header = true;
                match field {
                    "word" => word_idx = idx,
                    "meaning_zh" => meaning_idx = idx,
                    "phonetic" => phonetic_idx = idx,
                    "part_of_speech" => pos_idx = idx,
                    "difficulty" => diff_idx = idx,
                    _ => {}
                }
            }
        }

        let start_idx = if has_header { 1 } else { 0 };

        let mut entries = Vec::new();

        for row in rows.iter().skip(start_idx) {
            let word = row.get(word_idx).map(|s| s.trim()).unwrap_or("").to_string();
            let meaning_zh = row.get(meaning_idx).map(|s| s.trim()).unwrap_or("").to_string();

            if word.is_empty() && meaning_zh.is_empty() {
                continue;
            }

            let phonetic = row.get(phonetic_idx).map(|s| s.trim()).filter(|s| !s.is_empty()).map(String::from);
            let part_of_speech = row.get(pos_idx).map(|s| s.trim()).filter(|s| !s.is_empty()).map(String::from);
            let difficulty = row
                .get(diff_idx)
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(1);

            entries.push(WordbookEntry {
                word,
                meaning_zh,
                phonetic,
                part_of_speech,
                difficulty,
            });
        }

        Ok(entries)
    })();

    let _ = fs::remove_file(&temp_path);
    result
}

// ============================================================================
// Main Import
// ============================================================================

pub struct WordbookImporter;

impl WordbookImporter {
    pub fn import_from_embedded(db: &Database, json_content: &str, source: &str) -> Result<usize> {
        let entries = parse_json(json_content)?;
        let summary = Self::import_entries(db, entries, source, "json")?;
        Ok(summary.imported_count)
    }

    pub fn import_from_bytes(
        db: &Database,
        raw_bytes: &[u8],
        source: &str,
        file_name: Option<&str>,
    ) -> Result<WordbookImportSummary> {
        let (_, content) = detect_encoding_and_convert(raw_bytes)?;
        let format = detect_format(file_name, Some(&content))?;

        let entries = match format.as_str() {
            "json" => parse_json(&content)?,
            "csv" | "tsv" => parse_csv(&content),
            "txt" => parse_txt(&content),
            "xlsx" => parse_xlsx(raw_bytes, file_name)?,
            _ => unreachable!(),
        };

        Self::import_entries(db, entries, source, &format)
    }

    fn import_entries(
        db: &Database,
        entries: Vec<WordbookEntry>,
        source: &str,
        format: &str,
    ) -> Result<WordbookImportSummary> {
        let words_repo = WordsRepository::new(db.get_connection());
        let cards_repo = CardsRepository::new(db.get_connection());

        let mut imported_count = 0;
        let mut skipped_count = 0;

        for entry in entries.iter() {
            let word = entry.word.trim();
            let meaning_zh = entry.meaning_zh.trim();

            if word.is_empty() || meaning_zh.is_empty() {
                skipped_count += 1;
                continue;
            }

            if words_repo.get_by_word(word)?.is_some() {
                skipped_count += 1;
                continue;
            }

            let word_id = words_repo.insert(
                word,
                meaning_zh,
                source,
                entry.phonetic.as_deref().map(str::trim).filter(|v| !v.is_empty()),
                entry.part_of_speech.as_deref().map(str::trim).filter(|v| !v.is_empty()),
                entry.difficulty.max(1),
            )?;

            cards_repo.insert(word_id)?;
            imported_count += 1;
        }

        Ok(WordbookImportSummary {
            imported_count,
            skipped_count,
            total_count: entries.len(),
            source: source.to_string(),
            format: format.to_string(),
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::Migrator;
    use std::env;

    #[test]
    fn test_import_json() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_import_json.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let test_json = r#"[
            {"word": "test1", "meaning_zh": "测试1"},
            {"word": "test2", "meaning_zh": "测试2"}
        ]"#;

        let count = WordbookImporter::import_from_embedded(&db, test_json, "test").unwrap();
        assert_eq!(count, 2);

        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_parse_txt_tab_separated() {
        let txt = "abandon\t放弃\t/əˈbændən/\tv.\t2\nability - 能力\n";
        let entries = parse_txt(txt);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].word, "abandon");
        assert_eq!(entries[0].meaning_zh, "放弃");
    }

    #[test]
    fn test_parse_csv_with_header() {
        let csv = "word,meaning_zh\nabandon,放弃\nability,能力\n";
        let entries = parse_csv(csv);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].word, "abandon");
        assert_eq!(entries[0].meaning_zh, "放弃");
    }

    #[test]
    fn test_match_field_type() {
        assert_eq!(match_field_type("word"), Some("word"));
        assert_eq!(match_field_type("英文"), Some("word"));
        assert_eq!(match_field_type("meaning_zh"), Some("meaning_zh"));
        assert_eq!(match_field_type("中文"), Some("meaning_zh"));
        assert_eq!(match_field_type("phonetic"), Some("phonetic"));
        assert_eq!(match_field_type("音标"), Some("phonetic"));
    }
}
