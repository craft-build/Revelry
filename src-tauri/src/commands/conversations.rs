use tauri::State;

use crate::ai::embeddings::{spawn_delete_embedding, spawn_embed_message, EmbeddingRuntime};
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
    // Drop the conversation's derived message vectors along with its messages.
    {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::DeleteEmbeddingsByNamespace {
            namespace: format!("messages:{id}"),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
    }

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
pub async fn rename_conversation(
    db: State<'_, DbWorker>,
    id: String,
    title: String,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::RenameConversation {
        id,
        title,
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
pub async fn save_message(
    db: State<'_, DbWorker>,
    runtime: State<'_, EmbeddingRuntime>,
    message: Message,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveMessage {
        message: message.clone(),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;

    // Write-reconcile the message vector (skips when content is unchanged).
    spawn_embed_message(
        (*runtime).clone(),
        db.sender(),
        message.conversation_id,
        message.id,
        message.content,
    );
    Ok(())
}

#[tauri::command]
pub async fn delete_message(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    spawn_delete_embedding(db.sender(), id.clone());
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
    runtime: State<'_, EmbeddingRuntime>,
    query: String,
    semantic: Option<bool>,
) -> Result<Vec<SearchResult>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SearchMessages {
        query: query.clone(),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    let lexical = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;

    // Semantic pass (Phase 4): embed the query, cosine over every indexed
    // message, and merge in conversations the lexical pass missed. Every
    // step fails open — errors degrade to plain FTS5 results.
    if !semantic.unwrap_or(false) {
        return Ok(lexical);
    }
    let semantic_hits = match semantic_message_search(&db, &runtime, &query).await {
        Ok(hits) => hits,
        Err(e) => {
            tracing::debug!("semantic search skipped: {e}");
            Vec::new()
        }
    };
    Ok(crate::db::conversations::merge_semantic_results(
        lexical,
        semantic_hits,
    ))
}

/// Maximum semantic hits merged into a search result set.
const SEMANTIC_SEARCH_LIMIT: usize = 20;

/// Pure-lexical fallback snippet length for semantic hits (no <mark>
/// highlighting — the match is semantic, not lexical).
const SEMANTIC_SNIPPET_CHARS: usize = 140;

async fn semantic_message_search(
    db: &State<'_, DbWorker>,
    runtime: &State<'_, EmbeddingRuntime>,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    use crate::ai::embeddings::db_call;
    use crate::service::vector::cosine_similarity;

    let sender = db.sender();
    let runtime = runtime.inner().clone();

    let query_vector = tokio::time::timeout(
        std::time::Duration::from_secs(crate::ai::embeddings::EMBED_TIMEOUT_SECS),
        runtime.embed_query(&sender, query),
    )
    .await
    .map_err(|_| "query embedding timed out".to_string())??;

    let loaded = runtime.resolve(&sender).await?;
    let vectors = db_call(&sender, |reply| DbCommand::LoadEmbeddingsByPrefix {
        prefix: "messages:".to_string(),
        model_key: loaded.model_key.clone(),
        reply,
    })
    .await
    .map_err(|e| e.to_string())?;

    // Rank all indexed messages by cosine, keep the top hits.
    let mut scored: Vec<(f64, String)> = vectors
        .into_iter()
        .map(|(_namespace, record)| {
            (
                cosine_similarity(&query_vector, &record.vector) as f64,
                record.ref_id,
            )
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(SEMANTIC_SEARCH_LIMIT);

    let ids: Vec<String> = scored.iter().map(|(_, id)| id.clone()).collect();
    let messages = db_call(&sender, |reply| DbCommand::LoadMessagesWithConversation {
        ids,
        reply,
    })
    .await
    .map_err(|e| e.to_string())?;

    let mut hits = Vec::new();
    for (similarity, message_id) in &scored {
        let Some((message, title, character_id)) = messages
            .iter()
            .find(|(m, _, _)| &m.id == message_id)
            .cloned()
        else {
            continue; // Message deleted since embedding — skip.
        };
        if message.content.trim().is_empty() {
            continue;
        }
        let snippet: String = message
            .content
            .chars()
            .take(SEMANTIC_SNIPPET_CHARS)
            .collect();
        hits.push(SearchResult {
            conversation_id: message.conversation_id.clone(),
            conversation_title: title,
            character_id,
            match_type: "semantic".to_string(),
            message_snippet: Some(snippet),
            message_id: Some(message.id.clone()),
            branch_id: message.branch_id.clone(),
            rank: *similarity,
        });
    }
    Ok(hits)
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
