use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::lorebook::{Lorebook, LorebookEntry};

// ── Lorebook commands ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_lorebooks(db: State<'_, DbWorker>) -> Result<Vec<Lorebook>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadLorebooks { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_lorebook(db: State<'_, DbWorker>, id: String) -> Result<Option<Lorebook>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadLorebook {
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
pub async fn save_lorebook(
    db: State<'_, DbWorker>,
    lorebook: Lorebook,
) -> Result<Lorebook, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveLorebook {
        lorebook,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_lorebook(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteLorebook {
        id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

// ── Lorebook entry commands ────────────────────────────────────────────────

#[tauri::command]
pub async fn get_lorebook_entries(
    db: State<'_, DbWorker>,
    lorebook_id: String,
) -> Result<Vec<LorebookEntry>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadLorebookEntries {
        lorebook_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_lorebook_entry(
    db: State<'_, DbWorker>,
    entry: LorebookEntry,
) -> Result<LorebookEntry, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveLorebookEntry {
        entry,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_lorebook_entry(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteLorebookEntry {
        id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

// ── Character-lorebook junction commands ────────────────────────────────────

#[tauri::command]
pub async fn assign_lorebook_to_character(
    db: State<'_, DbWorker>,
    character_id: String,
    lorebook_id: String,
    sort_order: i32,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::AssignLorebookToCharacter {
        character_id,
        lorebook_id,
        sort_order,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_lorebook_from_character(
    db: State<'_, DbWorker>,
    character_id: String,
    lorebook_id: String,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::RemoveLorebookFromCharacter {
        character_id,
        lorebook_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_character_lorebooks(
    db: State<'_, DbWorker>,
    character_id: String,
) -> Result<Vec<Lorebook>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadCharacterLorebooks {
        character_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

// ── Conversation-lorebook junction commands ────────────────────────────────

#[tauri::command]
pub async fn assign_lorebook_to_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
    lorebook_id: String,
    sort_order: i32,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::AssignLorebookToConversation {
        conversation_id,
        lorebook_id,
        sort_order,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_lorebook_from_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
    lorebook_id: String,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::RemoveLorebookFromConversation {
        conversation_id,
        lorebook_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_conversation_lorebooks(
    db: State<'_, DbWorker>,
    conversation_id: String,
) -> Result<Vec<Lorebook>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadConversationLorebooks {
        conversation_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
