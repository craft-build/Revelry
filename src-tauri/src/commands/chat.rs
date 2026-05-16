use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::ai::{AiService, GenerationParams, StreamChunk};
use crate::db::{DbCommand, DbWorker};
use crate::domain::conversation::{Message as DomainMessage, MessageRole};

/// Managed state holding the cancellation token for the currently active stream.
/// Only one stream can be active at a time — starting a new stream cancels any existing one.
pub struct CancelState {
    pub token: Mutex<Option<CancellationToken>>,
}

#[tauri::command]
pub async fn start_chat(
    db: State<'_, DbWorker>,
    cancel_state: State<'_, CancelState>,
    channel: Channel<StreamChunk>,
    conversation_id: String,
    content: String,
    message_id: String,
    sort_order: i32,
) -> Result<(), String> {
    // 1. Cancel any existing stream
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = token_guard.take() {
            existing.cancel();
        }
    }

    // 2. Save the user message atomically using frontend-provided id and sort_order
    let user_message = DomainMessage {
        id: message_id,
        conversation_id: conversation_id.clone(),
        role: MessageRole::User,
        content: content.clone(),
        sort_order,
        timestamp: chrono::Utc::now(),
    };
    {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::SaveMessage {
            message: user_message.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
    }

    // 3. Load conversation
    let conversation = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadConversations { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let conversations = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
        conversations
            .into_iter()
            .find(|c| c.id == conversation_id)
            .ok_or_else(|| "Conversation not found".to_string())?
    };

    // 4. Load all messages for this conversation
    let mut messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
            conversation_id: conversation_id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };

    // 5. Load character for system prompt
    let character = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadCharacters { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let characters = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
        characters
            .into_iter()
            .find(|c| c.id == conversation.character_id)
    };

    // 6. Load provider configuration
    let provider = match &conversation.provider_id {
        Some(pid) => {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadProviders { reply: reply_tx })
                .map_err(|e| e.to_string())?;
            let providers = reply_rx
                .await
                .map_err(|e| format!("DB worker dropped: {}", e))?
                .map_err(|e| e.to_string())?;
            providers.into_iter().find(|p| p.id == *pid)
        }
        None => None,
    };

    let provider = match provider {
        Some(p) => p,
        None => {
            let _ = channel.send(StreamChunk::Error {
                error: "No provider configured for this conversation. Select a provider and model from the header.".to_string(),
                error_type: "no_provider_configured".to_string(),
            });
            return Ok(());
        }
    };

    // 7. Load API key (or empty string for Ollama)
    let provider_id_for_key = provider.id.clone();
    let provider_type = provider.provider_type.clone();
    let api_key_result = tokio::task::spawn_blocking(move || {
        if !provider_type.requires_api_key() {
            return Ok(String::new());
        }
        crate::secrets::KeyringStore::load_key(&provider_id_for_key)
    })
    .await
    .map_err(|e| format!("Keyring task failed: {}", e))?;

    let api_key = match api_key_result {
        Ok(key) => key,
        Err(_) => {
            let _ = channel.send(StreamChunk::Error {
                error: "No API key set for this provider. Add your key in Settings.".to_string(),
                error_type: "no_api_key".to_string(),
            });
            return Ok(());
        }
    };

    // 8. Build system prompt from character personality + scenario
    let system_prompt = match &character {
        Some(ch) => {
            let parts: Vec<&str> = [&ch.personality, &ch.scenario]
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .collect();
            if parts.is_empty() {
                "You are a helpful assistant.".to_string()
            } else {
                parts.join("\n\n")
            }
        }
        None => "You are a helpful assistant.".to_string(),
    };

    // 9. Resolve generation params from provider
    let params = AiService::from_provider_config(&provider);

    // 10. Create AI placeholder message ID and set up cancellation
    let ai_message_id = uuid::Uuid::new_v4().to_string();
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        *token_guard = Some(cancel_token.clone());
    }

    // Add the user message to the loaded messages for the prompt context
    messages.push(user_message);

    // 11. Spawn the streaming task
    let provider_config = provider.clone();
    tokio::spawn(async move {
        AiService::stream_chat(
            &provider_config,
            &system_prompt,
            &messages,
            &api_key,
            params,
            ai_message_id.clone(),
            channel,
            cancel_token,
        );
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_generation(cancel_state: State<'_, CancelState>) -> Result<(), String> {
    let token = {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        token_guard.take()
    };

    if let Some(token) = token {
        token.cancel();
    }

    Ok(())
}

#[tauri::command]
pub async fn regenerate(
    db: State<'_, DbWorker>,
    cancel_state: State<'_, CancelState>,
    channel: Channel<StreamChunk>,
    conversation_id: String,
) -> Result<(), String> {
    // Cancel any existing stream
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = token_guard.take() {
            existing.cancel();
        }
    }

    // Load messages for this conversation
    let messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
            conversation_id: conversation_id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };

    // Find the last AI message and delete it
    let last_ai_msg = messages
        .iter()
        .rev()
        .find(|m| m.role == MessageRole::Assistant);

    if let Some(ai_msg) = last_ai_msg {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::DeleteMessage {
            id: ai_msg.id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
    }

    // Now load conversation and provider/character to re-stream
    let conversation = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadConversations { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let conversations = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
        conversations
            .into_iter()
            .find(|c| c.id == conversation_id)
            .ok_or_else(|| "Conversation not found".to_string())?
    };

    // Reload messages after deletion
    let updated_messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
            conversation_id: conversation_id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };

    // Load character
    let character = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadCharacters { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let characters = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
        characters
            .into_iter()
            .find(|c| c.id == conversation.character_id)
    };

    // Load provider
    let provider = match &conversation.provider_id {
        Some(pid) => {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadProviders { reply: reply_tx })
                .map_err(|e| e.to_string())?;
            let providers = reply_rx
                .await
                .map_err(|e| format!("DB worker dropped: {}", e))?
                .map_err(|e| e.to_string())?;
            providers.into_iter().find(|p| p.id == *pid)
        }
        None => None,
    };

    let provider = match provider {
        Some(p) => p,
        None => {
            let _ = channel.send(StreamChunk::Error {
                error: "No provider configured for this conversation.".to_string(),
                error_type: "no_provider_configured".to_string(),
            });
            return Ok(());
        }
    };

    // Load API key
    let provider_id_for_key = provider.id.clone();
    let provider_type = provider.provider_type.clone();
    let api_key_result = tokio::task::spawn_blocking(move || {
        if !provider_type.requires_api_key() {
            return Ok(String::new());
        }
        crate::secrets::KeyringStore::load_key(&provider_id_for_key)
    })
    .await
    .map_err(|e| format!("Keyring task failed: {}", e))?;

    let api_key = match api_key_result {
        Ok(key) => key,
        Err(_) => {
            let _ = channel.send(StreamChunk::Error {
                error: "No API key set for this provider.".to_string(),
                error_type: "no_api_key".to_string(),
            });
            return Ok(());
        }
    };

    // Build system prompt
    let system_prompt = match &character {
        Some(ch) => {
            let parts: Vec<&str> = [&ch.personality, &ch.scenario]
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .collect();
            if parts.is_empty() {
                "You are a helpful assistant.".to_string()
            } else {
                parts.join("\n\n")
            }
        }
        None => "You are a helpful assistant.".to_string(),
    };

    let params = AiService::from_provider_config(&provider);

    // Set up cancellation and stream
    let ai_message_id = uuid::Uuid::new_v4().to_string();
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        *token_guard = Some(cancel_token.clone());
    }

    let provider_config = provider.clone();
    tokio::spawn(async move {
        AiService::stream_chat(
            &provider_config,
            &system_prompt,
            &updated_messages,
            &api_key,
            params,
            ai_message_id,
            channel,
            cancel_token,
        );
    });

    Ok(())
}

#[tauri::command]
pub async fn edit_and_resend(
    db: State<'_, DbWorker>,
    cancel_state: State<'_, CancelState>,
    channel: Channel<StreamChunk>,
    message_id: String,
    new_content: String,
) -> Result<(), String> {
    // Cancel any existing stream
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = token_guard.take() {
            existing.cancel();
        }
    }

    // 1. Load the message to edit by loading all conversations' messages and finding it
    //    We need the conversation_id, so we load conversations first and find the message.
    let (conversation_id, sort_order) = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadConversations { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let conversations = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;

        let mut found = None;
        for conv in &conversations {
            let (msg_tx, msg_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadMessages {
                conversation_id: conv.id.clone(),
                reply: msg_tx,
            })
            .map_err(|e| e.to_string())?;
            let msgs = msg_rx
                .await
                .map_err(|e| format!("DB worker dropped: {}", e))?
                .map_err(|e| e.to_string())?;

            if let Some(msg) = msgs.iter().find(|m| m.id == message_id) {
                found = Some((conv.id.clone(), msg.sort_order));
                break;
            }
        }
        found.ok_or_else(|| "Message not found".to_string())?
    };

    // 2. Update message content
    {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::UpdateMessageContent {
            id: message_id,
            content: new_content,
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
    }

    // 3. Delete all subsequent messages
    {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::DeleteMessagesAfterSortOrder {
            conversation_id: conversation_id.clone(),
            sort_order,
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
    }

    // 4. Load conversation and context for re-streaming
    let conversation = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadConversations { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let conversations = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
        conversations
            .into_iter()
            .find(|c| c.id == conversation_id)
            .ok_or_else(|| "Conversation not found".to_string())?
    };

    // Reload messages after edit and truncation
    let updated_messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
            conversation_id: conversation_id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };

    // Load character
    let character = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadCharacters { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let characters = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
        characters
            .into_iter()
            .find(|c| c.id == conversation.character_id)
    };

    // Load provider
    let provider = match &conversation.provider_id {
        Some(pid) => {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadProviders { reply: reply_tx })
                .map_err(|e| e.to_string())?;
            let providers = reply_rx
                .await
                .map_err(|e| format!("DB worker dropped: {}", e))?
                .map_err(|e| e.to_string())?;
            providers.into_iter().find(|p| p.id == *pid)
        }
        None => None,
    };

    let provider = match provider {
        Some(p) => p,
        None => {
            let _ = channel.send(StreamChunk::Error {
                error: "No provider configured for this conversation.".to_string(),
                error_type: "no_provider_configured".to_string(),
            });
            return Ok(());
        }
    };

    // Load API key
    let provider_id_for_key = provider.id.clone();
    let provider_type = provider.provider_type.clone();
    let api_key_result = tokio::task::spawn_blocking(move || {
        if !provider_type.requires_api_key() {
            return Ok(String::new());
        }
        crate::secrets::KeyringStore::load_key(&provider_id_for_key)
    })
    .await
    .map_err(|e| format!("Keyring task failed: {}", e))?;

    let api_key = match api_key_result {
        Ok(key) => key,
        Err(_) => {
            let _ = channel.send(StreamChunk::Error {
                error: "No API key set for this provider.".to_string(),
                error_type: "no_api_key".to_string(),
            });
            return Ok(());
        }
    };

    // Build system prompt
    let system_prompt = match &character {
        Some(ch) => {
            let parts: Vec<&str> = [&ch.personality, &ch.scenario]
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .collect();
            if parts.is_empty() {
                "You are a helpful assistant.".to_string()
            } else {
                parts.join("\n\n")
            }
        }
        None => "You are a helpful assistant.".to_string(),
    };

    let params = AiService::from_provider_config(&provider);

    // Set up cancellation and stream
    let ai_message_id = uuid::Uuid::new_v4().to_string();
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        *token_guard = Some(cancel_token.clone());
    }

    let provider_config = provider.clone();
    tokio::spawn(async move {
        AiService::stream_chat(
            &provider_config,
            &system_prompt,
            &updated_messages,
            &api_key,
            params,
            ai_message_id,
            channel,
            cancel_token,
        );
    });

    Ok(())
}
