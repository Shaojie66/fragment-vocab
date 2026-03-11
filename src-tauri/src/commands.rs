use tauri::State;
use crate::db::{Database, CardsRepository, LogsRepository, StateRepository};
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
    
    let now = chrono::Utc::now().to_rfc3339();
    
    // 先尝试获取到期的复习卡片
    let due_cards = cards_repo.get_due_cards(&now, 1)
        .map_err(|e| format!("Failed to get due cards: {}", e))?;
    
    if let Some(word_with_card) = due_cards.first() {
        return Ok(Some(WordCardData {
            word_id: word_with_card.word.id,
            card_id: word_with_card.card.id,
            word: word_with_card.word.word.clone(),
            phonetic: word_with_card.word.phonetic.clone(),
            part_of_speech: word_with_card.word.part_of_speech.clone(),
            meaning_zh: word_with_card.word.meaning_zh.clone(),
        }));
    }
    
    // 如果没有到期的复习卡片，获取新词
    let new_cards = cards_repo.get_new_cards(&now, 1)
        .map_err(|e| format!("Failed to get new cards: {}", e))?;
    
    if let Some(word_with_card) = new_cards.first() {
        return Ok(Some(WordCardData {
            word_id: word_with_card.word.id,
            card_id: word_with_card.card.id,
            word: word_with_card.word.word.clone(),
            phonetic: word_with_card.word.phonetic.clone(),
            part_of_speech: word_with_card.word.part_of_speech.clone(),
            meaning_zh: word_with_card.word.meaning_zh.clone(),
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
    
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    
    // 获取当前卡片
    let mut card = cards_repo.get_by_id(card_id)
        .map_err(|e| format!("Failed to get card: {}", e))?
        .ok_or_else(|| "Card not found".to_string())?;
    
    // 计算新的状态
    match result.as_str() {
        "know" => {
            card.stage += 1;
            card.status = if card.stage >= 5 { "mastered".to_string() } else { "learning".to_string() };
            card.correct_streak += 1;
            card.lifetime_correct += 1;
            card.last_result = Some("know".to_string());
            card.last_seen_at = Some(now_str.clone());
            
            let interval_minutes = match card.stage {
                0 => 10,
                1 => 1440,      // 1 day
                2 => 4320,      // 3 days
                3 => 10080,     // 7 days
                4 => 20160,     // 14 days
                _ => 0,
            };
            
            card.due_at = if card.status == "mastered" {
                None
            } else {
                Some((now + chrono::Duration::minutes(interval_minutes)).to_rfc3339())
            };
            card.skip_cooldown_until = None;
        }
        "dont_know" => {
            card.stage = std::cmp::max(0, card.stage - 1);
            card.status = "learning".to_string();
            card.correct_streak = 0;
            card.lifetime_wrong += 1;
            card.last_result = Some("dont_know".to_string());
            card.last_seen_at = Some(now_str.clone());
            
            let interval_minutes = match card.stage {
                0 => 10,
                1 => 1440,
                2 => 4320,
                3 => 10080,
                4 => 20160,
                _ => 10,
            };
            
            card.due_at = Some((now + chrono::Duration::minutes(interval_minutes)).to_rfc3339());
            card.skip_cooldown_until = None;
        }
        "skip" => {
            // 跳过：设置冷却期，不改变其他状态
            card.last_result = Some("skip".to_string());
            card.last_seen_at = Some(now_str.clone());
            card.skip_cooldown_until = Some((now + chrono::Duration::minutes(30)).to_rfc3339());
        }
        _ => return Err(format!("Invalid result: {}", result)),
    };
    
    // 更新卡片
    cards_repo.update(&card, &now_str)
        .map_err(|e| format!("Failed to update card: {}", e))?;
    
    // 记录日志
    logs_repo.insert(card_id, &now_str, &result, "manual", None)
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
            if let Ok(shown_at) = chrono::DateTime::parse_from_rfc3339(&log.shown_at) {
                shown_at.naive_utc() >= today_start
            } else {
                false
            }
        })
        .collect();
    
    let total_reviews = today_logs.len() as i64;
    let know_count = today_logs.iter().filter(|log| log.result == "know").count() as i64;
    let dont_know_count = today_logs.iter().filter(|log| log.result == "dont_know").count() as i64;
    let skip_count = today_logs.iter().filter(|log| log.result == "skip").count() as i64;
    
    let accuracy = if know_count + dont_know_count > 0 {
        (know_count as f64 / (know_count + dont_know_count) as f64) * 100.0
    } else {
        0.0
    };
    
    // 统计今日新学词数
    let new_words_today = today_logs.iter()
        .filter(|log| log.result == "know" || log.result == "dont_know")
        .map(|log| log.card_id)
        .collect::<std::collections::HashSet<_>>()
        .len() as i64;
    
    // 统计待复习词数
    let now = chrono::Utc::now().to_rfc3339();
    let due_cards = cards_repo.get_due_cards(&now, 1000)
        .map_err(|e| format!("Failed to get due cards: {}", e))?;
    let due_cards_count = due_cards.len() as i64;
    
    // 统计已掌握词数
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
pub fn pause_scheduler(db: State<Database>, minutes: i64) -> Result<(), String> {
    let conn = db.get_connection();
    let state_repo = StateRepository::new(conn);
    
    let pause_until = chrono::Utc::now() + chrono::Duration::minutes(minutes);
    let now = chrono::Utc::now().to_rfc3339();
    
    state_repo.set("pause_until", &pause_until.to_rfc3339(), &now)
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
