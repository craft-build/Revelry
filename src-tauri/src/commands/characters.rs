use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::CharacterCardV2;

// Note: Tauri v2 uses the Rust function name as-is (snake_case) for the command name.
// Frontend invoke() must use the same snake_case string.
// Tauri v2 applies camelCase-to-snake_case conversion for command arguments.

#[tauri::command]
pub async fn get_characters(db: State<'_, DbWorker>) -> Result<Vec<CharacterCardV2>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadCharacters { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_character(
    db: State<'_, DbWorker>,
    character: CharacterCardV2,
) -> Result<CharacterCardV2, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveCharacter {
        character,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_character(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteCharacter {
        id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_characters(
    db: State<'_, DbWorker>,
    query: String,
) -> Result<Vec<CharacterCardV2>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SearchCharacters {
        query,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
