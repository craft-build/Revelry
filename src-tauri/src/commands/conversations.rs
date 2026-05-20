use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::{Conversation, Message, MessageBranch, SearchResult};

#[tauri::command]
pub async fn get_conversations(db: State<'_, DbWorker>) -> Result<Vec<Conversation>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadConversations { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_conversation(
    db: State<'_, DbWorker>,
    character_id: String,
) -> Result<Conversation, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::StartConversation {
        character_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_conversation(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteConversation {
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
pub async fn search_conversations(
    db: State<'_, DbWorker>,
    query: String,
) -> Result<Vec<Conversation>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SearchConversations {
        query,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_messages(
    db: State<'_, DbWorker>,
    conversation_id: String,
    branch_id: Option<String>,
) -> Result<Vec<Message>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadMessages {
        conversation_id,
        branch_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_message(db: State<'_, DbWorker>, message: Message) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveMessage {
        message,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_message(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteMessage {
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
pub async fn update_conversation_persona(
    db: State<'_, DbWorker>,
    conversation_id: String,
    persona_id: Option<String>,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::UpdateConversationPersona {
        conversation_id,
        persona_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fork_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
    parent_message_id: String,
) -> Result<MessageBranch, String> {
    // Count existing branches for auto-labeling
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::CountBranches {
        conversation_id: conversation_id.clone(),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    let count = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;
    let label = format!("Branch {}", count + 1);

    let branch = MessageBranch::new(conversation_id, parent_message_id, label);
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::CreateBranch {
        branch: branch.clone(),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;
    Ok(branch)
}

#[tauri::command]
pub async fn get_branches_for_message(
    db: State<'_, DbWorker>,
    parent_message_id: String,
) -> Result<Vec<MessageBranch>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadBranchesForMessage {
        parent_message_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_branches_for_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
) -> Result<Vec<MessageBranch>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadAllBranches {
        conversation_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_branch(db: State<'_, DbWorker>, branch_id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteBranch {
        branch_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_messages(
    db: State<'_, DbWorker>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SearchMessages {
        query,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_conversation_generation(
    db: State<'_, DbWorker>,
    conversation_id: String,
    generation_preset_id: Option<String>,
    override_max_tokens: Option<i64>,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::UpdateConversationGeneration {
        conversation_id,
        generation_preset_id,
        override_max_tokens,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
