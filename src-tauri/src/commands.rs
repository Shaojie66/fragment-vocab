use tauri::State;
use crate::db::{Database, CardsRepository, WordsRepository, LogsRepository, StateRepository};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WordCardData {
    pub word_id: i64,
    pub card_id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Option<String>,
    pub meaning_zh: String,
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

/// 获取下一张要展示的卡片
#[tauri::command]
pub fn get_next_card(db: State<Database>) -> Result<Option<WordCardData>, String> {
    let conn = db.get_connection();
    let cards_repo = CardsRepository::new(conn.clone());
    let words_repo = WordsRepository::new(conn.clone());
    
    // 先尝试获取到期的复习卡片
    let due_cards = cards_repo.get_due_cards(1)
        .map_err(|e| format!("Failed to get due cards: {}", e))?;
    
    if let Some(card) = due_cards.first() {
        let word = words_repo.get_by_id(card.word_id)
            .map_err(|e| format!("Failed to get word: {}", e))?
            .ok_or_else(|| "Word not found".to_string())?;
        
        return Ok(Some(WordCardData {
            word_id: word.id,
            card_id: card.id,
            word: word.word,
            phonetic: word.phonetic,
            part_of_speech: word.part_of_speech,
            meaning_zh: word.meaning_zh,
        }));
    }
    
    // 如果没有到期的复习卡片，获取新词
    let new_cards = cards_repo.get_new_cards(1)
        .map_err(|e| format!("Failed to get new cards: {}", e))?;
    
    if let Some(card) = new_cards.first() {
        let word = words_repo.get_by_id(card.word_id)
            .map_err(|e| format!("Failed to get word: {}", e))?
            .ok_or_else(|| "Word not found".to_string())?;
        
        return Ok(Some(WordCardData {
            word_id: word.id,
            card_id: card.id,
            word: word.word,
            phonetic: word.phonetic,
            part_of_speech: word.part_of_speech,
            meaning_zh: word.meaning_zh,
        }));
    }
    
    Ok(None)
}

/// 提交复习结果
#[tauri::command]
pub fn submit_review(
    db: State<Database>,
    card_id: i64,
    result: String,
) -> Result<(), String> {
    let conn = db.get_connection();
    let cards_repo = CardsRepository::new(conn.clone());
    let logs_repo = LogsRepository::new(conn.clone());
    
    // 获取当前卡片
    let card = cards_repo.get_by_id(card_id)
        .map_err(|e| format!("Failed to get card: {}", e))?
        .ok_or_else(|| "Card not found".to_string())?;
    
    // 计算新的状态
    let (new_status, new_stage, new_due_at) = match result.as_str() {
        "know" => {
            let new_stage = card.stage + 1;
            let new_status = if new_stage >= 5 { "mastered" } else { "learning" };
            let interval_minutes = match new_stage {
                0 => 10,
                1 => 1440,      // 1 day
                2 => 4320,      // 3 days
                3 => 10080,     // 7 days
                4 => 20160,     // 14 days
                _ => 0,
            };
            let due_at = if new_status == "mastered" {
                None
            } else {
                Some(chrono::Utc::now() + chrono::Duration::minutes(interval_minutes))
            };
            (new_status.to_string(), new_stage, due_at)
        }
        "dont_know" => {
            let new_stage = std::cmp::max(0, card.stage - 1);
            let interval_minutes = match new_stage {
                0 => 10,
                1 => 1440,
                2 => 4320,
                3 => 10080,
                4 => 20160,
                _ => 10,
            };
            let due_at = Some(chrono::Utc::now() + chrono::Duration::minutes(interval_minutes));
            ("learning".to_string(), new_stage, due_at)
        }
        "skip" => {
            // 跳过不改变状态，只记录日志
            (card.status.clone(), card.stage, card.due_at)
        }
        _ => return Err(format!("Invalid result: {}", result)),
    };
    
    // 更新卡片
    cards_repo.update(
        card_id,
        &new_status,
        new_stage,
        new_due_at,
        Some(chrono::Utc::now()),
        Some(&result),
    ).map_err(|e| format!("Failed to update card: {}", e))?;
    
    // 记录日志
    logs_repo.insert(card_id, &result)
        .map_err(|e| format!("Failed to insert log: {}", e))?;
    
    Ok(())
}

/// 获取今日统计数据
#[tauri::command]
pub fn get_today_stats(db: State<Database>) -> Result<TodayStats, String> {
    let conn = db.get_connection();
    let logs_repo = LogsRepository::new(conn.clone());
    let cards_repo = CardsRepository::new(conn.clone());
    
    // 获取今日日志
    let today_logs = logs_repo.get_recent_logs(1000)
        .map_err(|e| format!("Failed to get logs: {}", e))?;
    
    // 过滤今日的日志
    let today_start = chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_logs: Vec<_> = today_logs.into_iter()
        .filter(|log| {
            if let Ok(created_at) = chrono::DateTime::parse_from_rfc3339(&log.created_at) {
                created_at.naive_utc() >= today_start
            } else {
                false
            }
        })
        .collect();
    
    let total_reviews = today_logs.len() as i64;
    let know_count = today_logs.iter().filter(|log| log.result == "know").count() as i64;
    let dont_know_count = today_logs.iter().filter(|log| log.result == "dont_know").count() as i64;
    let skip_count = today_logs.iter().filter(|log| log.result == "skip").count() as i64;
    
    let accuracy = if total_reviews > 0 {
        (know_count as f64 / (know_count + dont_know_count) as f64) * 100.0
    } else {
        0.0
    };
    
    // 统计今日新学词数（首次答对的词）
    let new_words_today = today_logs.iter()
        .filter(|log| log.result == "know")
        .map(|log| log.card_id)
        .collect::<std::collections::HashSet<_>>()
        .len() as i64;
    
    // 获取待复习词数
    let due_cards = cards_repo.get_due_cards(1000)
        .map_err(|e| format!("Failed to get due cards: {}", e))?;
    let due_cards_count = due_cards.len() as i64;
    
    // 获取已掌握词数
    let mastered_count = cards_repo.count_by_status("mastered")
        .map_err(|e| format!("Failed to count mastered cards: {}", e))?;
    
    Ok(TodayStats {
        total_reviews,
        know_count,
        dont_know_count,
        skip_count,
        new_words_today,
        due_cards_count,
        mastered_count,
        accuracy,
    })
}

/// 暂停调度器
#[tauri::command]
pub fn pause_scheduler(db: State<Database>, duration_minutes: i64) -> Result<(), String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);
    
    let pause_until = chrono::Utc::now() + chrono::Duration::minutes(duration_minutes);
    state_repo.set("pause_until", &pause_until.to_rfc3339())
        .map_err(|e| format!("Failed to set pause state: {}", e))?;
    
    Ok(())
}

/// 恢复调度器
#[tauri::command]
pub fn resume_scheduler(db: State<Database>) -> Result<(), String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);
    
    state_repo.delete("pause_until")
        .map_err(|e| format!("Failed to delete pause state: {}", e))?;
    
    Ok(())
}

/// 检查是否暂停中
#[tauri::command]
pub fn is_paused(db: State<Database>) -> Result<bool, String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);
    
    if let Some(pause_until_str) = state_repo.get("pause_until")
        .map_err(|e| format!("Failed to get pause state: {}", e))? {
        if let Ok(pause_until) = chrono::DateTime::parse_from_rfc3339(&pause_until_str) {
            return Ok(chrono::Utc::now() < pause_until);
        }
    }
    
    Ok(false)
}
