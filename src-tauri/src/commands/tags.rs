use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{Database, TagsRepository};
use crate::db::repositories::tags::{Tag, TagWithCount};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTagsInfo {
    pub word_id: i64,
    pub tags: Vec<Tag>,
}

#[tauri::command]
pub fn create_tag(db: State<Database>, name: String) -> Result<TagWithCount, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("标签名称不能为空".to_string());
    }
    if trimmed.len() > 50 {
        return Err("标签名称不能超过 50 个字符".to_string());
    }

    let conn = db.get_connection();
    let tags_repo = TagsRepository::new(conn);

    tags_repo
        .create(trimmed)
        .map(|tag| TagWithCount {
            id: tag.id,
            name: tag.name,
            word_count: 0,
            created_at: tag.created_at,
        })
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                format!("标签「{}」已存在", trimmed)
            } else {
                format!("创建标签失败: {}", e)
            }
        })
}

#[tauri::command]
pub fn delete_tag(db: State<Database>, tag_id: i64) -> Result<Vec<TagWithCount>, String> {
    let conn = db.get_connection();
    let tags_repo = TagsRepository::new(conn);

    tags_repo
        .delete(tag_id)
        .map_err(|e| format!("删除标签失败: {}", e))?;

    tags_repo
        .list_with_counts()
        .map_err(|e| format!("刷新标签列表失败: {}", e))
}

#[tauri::command]
pub fn list_tags(db: State<Database>) -> Result<Vec<TagWithCount>, String> {
    let conn = db.get_connection();
    let tags_repo = TagsRepository::new(conn);

    tags_repo
        .list_with_counts()
        .map_err(|e| format!("获取标签列表失败: {}", e))
}

#[tauri::command]
pub fn add_word_tag(db: State<Database>, word_id: i64, tag_id: i64) -> Result<Vec<Tag>, String> {
    let conn = db.get_connection();
    let tags_repo = TagsRepository::new(conn);

    tags_repo
        .add_word_tag(word_id, tag_id)
        .map_err(|e| format!("添加标签失败: {}", e))?;

    tags_repo
        .get_word_tags(word_id)
        .map_err(|e| format!("获取单词标签失败: {}", e))
}

#[tauri::command]
pub fn remove_word_tag(
    db: State<Database>,
    word_id: i64,
    tag_id: i64,
) -> Result<Vec<Tag>, String> {
    let conn = db.get_connection();
    let tags_repo = TagsRepository::new(conn);

    tags_repo
        .remove_word_tag(word_id, tag_id)
        .map_err(|e| format!("移除标签失败: {}", e))?;

    tags_repo
        .get_word_tags(word_id)
        .map_err(|e| format!("获取单词标签失败: {}", e))
}

#[tauri::command]
pub fn get_word_tags(db: State<Database>, word_id: i64) -> Result<Vec<Tag>, String> {
    let conn = db.get_connection();
    let tags_repo = TagsRepository::new(conn);

    tags_repo
        .get_word_tags(word_id)
        .map_err(|e| format!("获取单词标签失败: {}", e))
}
