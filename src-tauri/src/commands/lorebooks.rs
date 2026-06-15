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

// ── Lorebook import ────────────────────────────────────────────────────────

/// Import a lorebook from a CharacterBook V2 JSON file.
///
/// Reads and parses the file (standalone book or one embedded in a Character
/// Card V2), persists the lorebook and all of its entries, and returns the
/// saved lorebook. Imported lorebooks are assigned fresh UUIDs.
#[tauri::command]
pub async fn import_lorebook(
    db: State<'_, DbWorker>,
    file_path: String,
) -> Result<Lorebook, String> {
    let path = std::path::PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext != "json" {
        return Err(format!(
            "Unsupported file format: .{}. Only .json is supported for lorebook import.",
            ext
        ));
    }

    let json_str =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    let (lorebook, entries) =
        crate::service::lorebook_io::import_from_json(&json_str).map_err(|e| e.to_string())?;

    // Persist the lorebook first (DB canonicalizes timestamps; id is preserved).
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveLorebook {
        lorebook,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    let saved = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;

    // Persist each entry (already keyed to the lorebook's id).
    for entry in entries {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::SaveLorebookEntry {
            entry,
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
    }

    Ok(saved)
}
