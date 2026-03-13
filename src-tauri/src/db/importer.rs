use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Reader};
use csv::{ReaderBuilder, StringRecord, Trim};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

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
        alias = "英文"
    )]
    word: String,
    #[serde(alias = "pronunciation", alias = "ipa", alias = "音标")]
    phonetic: Option<String>,
    #[serde(alias = "pos", alias = "词性")]
    part_of_speech: Option<String>,
    #[serde(
        alias = "meaning",
        alias = "translation",
        alias = "definition",
        alias = "chinese",
        alias = "中文",
        alias = "释义",
        alias = "词义"
    )]
    meaning_zh: String,
    #[serde(
        default = "default_difficulty",
        alias = "level",
        alias = "rank",
        alias = "难度"
    )]
    difficulty: i32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonWordbookPayload {
    Entries(Vec<WordbookEntry>),
    Wrapped {
        #[serde(alias = "entries", alias = "items", alias = "vocabulary")]
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

pub struct WordbookImporter;

impl WordbookImporter {
    pub fn import_from_embedded(db: &Database, json_content: &str, source: &str) -> Result<usize> {
        let entries = parse_json_wordbook(json_content)?;
        let summary = Self::import_entries(db, entries, source, "json")?;
        Ok(summary.imported_count)
    }

    pub fn import_from_bytes(
        db: &Database,
        raw_bytes: &[u8],
        source: &str,
        file_name: Option<&str>,
    ) -> Result<WordbookImportSummary> {
        let utf8_content = std::str::from_utf8(raw_bytes).ok();
        let format = detect_format(file_name, utf8_content)?;

        let entries = match format.as_str() {
            "json" => {
                parse_json_wordbook(utf8_content.context("JSON wordbook must be valid UTF-8")?)?
            }
            "csv" => parse_csv_wordbook(utf8_content.context("CSV wordbook must be valid UTF-8")?)?,
            "txt" => parse_txt_wordbook(utf8_content.context("TXT wordbook must be valid UTF-8")?)?,
            "xlsx" => parse_xlsx_wordbook(raw_bytes, file_name)?,
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
                entry
                    .phonetic
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
                entry
                    .part_of_speech
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
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

fn detect_format(file_name: Option<&str>, raw_content: Option<&str>) -> Result<String> {
    if let Some(name) = file_name {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".json") {
            return Ok("json".to_string());
        }
        if lower.ends_with(".csv") {
            return Ok("csv".to_string());
        }
        if lower.ends_with(".txt") {
            return Ok("txt".to_string());
        }
        if lower.ends_with(".xlsx") {
            return Ok("xlsx".to_string());
        }
    }

    if let Some(raw_content) = raw_content {
        let trimmed = raw_content.trim_start();
        if trimmed.starts_with('[') || trimmed.starts_with('{') {
            return Ok("json".to_string());
        }

        if trimmed.contains(',') || trimmed.contains('\n') {
            return Ok("csv".to_string());
        }

        if trimmed.contains('\t')
            || trimmed.contains(" - ")
            || trimmed.contains('：')
            || trimmed.contains(':')
        {
            return Ok("txt".to_string());
        }
    }

    Err(anyhow::anyhow!(
        "Unsupported wordbook format. Use JSON, CSV, TXT or XLSX."
    ))
}

fn parse_json_wordbook(json_content: &str) -> Result<Vec<WordbookEntry>> {
    let payload: JsonWordbookPayload =
        serde_json::from_str(json_content).context("Failed to parse wordbook JSON")?;

    let entries = match payload {
        JsonWordbookPayload::Entries(entries) => entries,
        JsonWordbookPayload::Wrapped { words } => words,
    };

    Ok(entries)
}

fn normalize_header(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&ch) {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn build_header_index_from_values(
    values: &[String],
) -> Option<(usize, usize, Option<usize>, Option<usize>, Option<usize>)> {
    let normalized = values
        .iter()
        .map(|value| normalize_header(value))
        .collect::<Vec<_>>();

    let find_index = |aliases: &[&str]| {
        normalized
            .iter()
            .position(|header| aliases.iter().any(|alias| header == alias))
    };

    let word_index = find_index(&[
        "word",
        "english",
        "term",
        "vocab",
        "vocabulary",
        "单词",
        "词汇",
        "英文",
    ])?;
    let meaning_index = find_index(&[
        "meaning_zh",
        "meaning",
        "translation",
        "definition",
        "chinese",
        "chinese_meaning",
        "中文",
        "释义",
        "词义",
    ])?;
    let phonetic_index = find_index(&["phonetic", "pronunciation", "ipa", "音标", "发音"]);
    let pos_index = find_index(&["part_of_speech", "pos", "词性"]);
    let difficulty_index = find_index(&["difficulty", "level", "rank", "难度"]);

    Some((
        word_index,
        meaning_index,
        phonetic_index,
        pos_index,
        difficulty_index,
    ))
}

fn build_header_index(
    record: &StringRecord,
) -> Option<(usize, usize, Option<usize>, Option<usize>, Option<usize>)> {
    build_header_index_from_values(&record.iter().map(str::to_string).collect::<Vec<_>>())
}

fn record_value(record: &StringRecord, index: Option<usize>) -> Option<String> {
    index
        .and_then(|position| record.get(position))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn parse_csv_wordbook(csv_content: &str) -> Result<Vec<WordbookEntry>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(csv_content.as_bytes());

    let records = reader
        .records()
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to parse wordbook CSV")?;

    if records.is_empty() {
        return Ok(Vec::new());
    }

    let header_index = build_header_index(&records[0]);
    let records_start = usize::from(header_index.is_some());
    let default_index = (0, 1, Some(2), Some(3), Some(4));
    let (word_index, meaning_index, phonetic_index, pos_index, difficulty_index) =
        header_index.unwrap_or(default_index);

    let mut entries = Vec::new();

    for record in records.iter().skip(records_start) {
        let word = record.get(word_index).unwrap_or("").trim().to_string();
        let meaning_zh = record.get(meaning_index).unwrap_or("").trim().to_string();

        if word.is_empty() && meaning_zh.is_empty() {
            continue;
        }

        let difficulty = record_value(record, difficulty_index)
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or_else(default_difficulty);

        entries.push(WordbookEntry {
            word,
            meaning_zh,
            phonetic: record_value(record, phonetic_index),
            part_of_speech: record_value(record, pos_index),
            difficulty,
        });
    }

    Ok(entries)
}

fn parse_txt_wordbook(txt_content: &str) -> Result<Vec<WordbookEntry>> {
    let mut entries = Vec::new();

    for line in txt_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let segments = if trimmed.contains('\t') {
            trimmed.split('\t').map(str::trim).collect::<Vec<_>>()
        } else if trimmed.contains(',') {
            trimmed.split(',').map(str::trim).collect::<Vec<_>>()
        } else if trimmed.contains(" - ") {
            trimmed.splitn(2, " - ").map(str::trim).collect::<Vec<_>>()
        } else if trimmed.contains('：') {
            trimmed.splitn(2, '：').map(str::trim).collect::<Vec<_>>()
        } else if trimmed.contains(':') {
            trimmed.splitn(2, ':').map(str::trim).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if segments.len() < 2 {
            continue;
        }

        let difficulty = segments
            .get(4)
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or_else(default_difficulty);

        entries.push(WordbookEntry {
            word: segments[0].to_string(),
            meaning_zh: segments[1].to_string(),
            phonetic: segments
                .get(2)
                .filter(|value| !value.is_empty())
                .map(|value| (*value).to_string()),
            part_of_speech: segments
                .get(3)
                .filter(|value| !value.is_empty())
                .map(|value| (*value).to_string()),
            difficulty,
        });
    }

    Ok(entries)
}

fn parse_xlsx_wordbook(raw_bytes: &[u8], file_name: Option<&str>) -> Result<Vec<WordbookEntry>> {
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

        let rows = range
            .rows()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let header_index = build_header_index_from_values(&rows[0]);
        let rows_start = usize::from(header_index.is_some());
        let default_index = (0, 1, Some(2), Some(3), Some(4));
        let (word_index, meaning_index, phonetic_index, pos_index, difficulty_index) =
            header_index.unwrap_or(default_index);

        let mut entries = Vec::new();

        for row in rows.iter().skip(rows_start) {
            let word = row
                .get(word_index)
                .map(String::as_str)
                .unwrap_or("")
                .trim()
                .to_string();
            let meaning_zh = row
                .get(meaning_index)
                .map(String::as_str)
                .unwrap_or("")
                .trim()
                .to_string();

            if word.is_empty() && meaning_zh.is_empty() {
                continue;
            }

            let difficulty = difficulty_index
                .and_then(|index| row.get(index))
                .and_then(|value| value.trim().parse::<i32>().ok())
                .unwrap_or_else(default_difficulty);

            entries.push(WordbookEntry {
                word,
                meaning_zh,
                phonetic: phonetic_index
                    .and_then(|index| row.get(index))
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                part_of_speech: pos_index
                    .and_then(|index| row.get(index))
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                difficulty,
            });
        }

        Ok(entries)
    })();

    let _ = fs::remove_file(&temp_path);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::Migrator;
    use std::env;

    #[test]
    fn test_import_wordbook_json() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_import_json.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let test_json = r#"[
            {
                "word": "test1",
                "phonetic": "/test/",
                "part_of_speech": "n.",
                "meaning_zh": "测试1",
                "difficulty": 1
            },
            {
                "word": "test2",
                "meaning_zh": "测试2"
            }
        ]"#;

        let count = WordbookImporter::import_from_embedded(&db, test_json, "test").unwrap();
        assert_eq!(count, 2);

        let words_repo = WordsRepository::new(db.get_connection());
        assert_eq!(words_repo.count().unwrap(), 2);

        let count2 = WordbookImporter::import_from_embedded(&db, test_json, "test").unwrap();
        assert_eq!(count2, 0);

        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_import_wordbook_csv_with_header() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_import_csv.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let csv = "word,meaning_zh,phonetic,part_of_speech,difficulty\nabandon,放弃,/əˈbændən/,v.,2\nability,能力,,,1\n";
        let summary = WordbookImporter::import_from_bytes(
            &db,
            csv.as_bytes(),
            "custom-upload",
            Some("my.csv"),
        )
        .unwrap();

        assert_eq!(summary.format, "csv");
        assert_eq!(summary.imported_count, 2);
        assert_eq!(summary.skipped_count, 0);

        let words_repo = WordsRepository::new(db.get_connection());
        assert_eq!(words_repo.count().unwrap(), 2);

        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_import_wordbook_csv_with_alias_headers() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_import_csv_alias.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let csv =
            "English,Translation,IPA,POS,Level\nabandon,放弃,/əˈbændən/,v.,2\nability,能力,,,1\n";
        let summary = WordbookImporter::import_from_bytes(
            &db,
            csv.as_bytes(),
            "custom-upload",
            Some("alias.csv"),
        )
        .unwrap();

        assert_eq!(summary.imported_count, 2);
        assert_eq!(summary.skipped_count, 0);

        let words_repo = WordsRepository::new(db.get_connection());
        let abandon = words_repo.get_by_word("abandon").unwrap().unwrap();
        assert_eq!(abandon.meaning_zh, "放弃");
        assert_eq!(abandon.phonetic.as_deref(), Some("/əˈbændən/"));

        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_import_wordbook_json_with_alias_fields() {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_import_json_alias.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        Migrator::run_migrations(&db).unwrap();

        let test_json = r#"{
            "entries": [
                {
                    "english": "curate",
                    "translation": "策展；整理",
                    "ipa": "/kjʊˈreɪt/",
                    "pos": "v.",
                    "level": 3
                }
            ]
        }"#;

        let summary = WordbookImporter::import_from_bytes(
            &db,
            test_json.as_bytes(),
            "custom-json-alias",
            Some("alias.json"),
        )
        .unwrap();

        assert_eq!(summary.imported_count, 1);

        let words_repo = WordsRepository::new(db.get_connection());
        let word = words_repo.get_by_word("curate").unwrap().unwrap();
        assert_eq!(word.meaning_zh, "策展；整理");
        assert_eq!(word.part_of_speech.as_deref(), Some("v."));
        assert_eq!(word.difficulty, 3);

        drop(words_repo);
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_parse_txt_wordbook() {
        let txt = "abandon\t放弃\t/əˈbændən/\tv.\t2\nability - 能力\n";
        let entries = parse_txt_wordbook(txt).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].word, "abandon");
        assert_eq!(entries[0].meaning_zh, "放弃");
        assert_eq!(entries[1].word, "ability");
        assert_eq!(entries[1].meaning_zh, "能力");
    }
}
