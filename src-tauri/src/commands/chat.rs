use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::ai::{AiService, BudgetInfo, GenerationParams, StreamChunk};
use crate::db::{DbCommand, DbWorker};
use crate::domain::conversation::{Message as DomainMessage, MessageRole};
use crate::domain::lorebook::LorebookActivationDetail;
use crate::domain::regex_script::RegexScript;
use crate::service::prompt::PromptEngine;
use crate::service::token_count::TokenCountService;

/// Managed state holding the cancellation token for the currently active stream.
/// Only one stream can be active at a time — starting a new stream cancels any existing one.
pub struct CancelState {
    pub token: Mutex<Option<CancellationToken>>,
}

/// Shared context loaded for all chat commands.
struct ChatContext {
    character: Option<crate::domain::character::CharacterCardV2>,
    persona: Option<crate::domain::persona::Persona>,
    provider: crate::domain::provider::ProviderConfig,
    /// The active model name — conversation override or provider default.
    active_model_name: String,
    messages: Vec<DomainMessage>,
    api_key: String,
    /// Lorebook entries from character + conversation assigned lorebooks.
    lorebook_entries: Vec<crate::domain::lorebook::LorebookEntry>,
    /// IDs of lorebooks assigned to the character.
    character_lorebook_ids: Vec<String>,
    /// IDs of lorebooks assigned to the conversation.
    conversation_lorebook_ids: Vec<String>,
    /// Regex scripts for the character.
    regex_scripts: Vec<RegexScript>,
}

/// Loads all context needed for prompt assembly and streaming.
/// Returns Ok(Some(ChatContext)) on success, or sends an error StreamChunk and returns Ok(None).
async fn load_chat_context(
    db: &State<'_, DbWorker>,
    channel: &Channel<StreamChunk>,
    conversation_id: &str,
    branch_id: Option<String>,
) -> Result<Option<ChatContext>, String> {
    // 1. Load conversation
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

    // 2. Load messages
    let messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
                    conversation_id: conversation_id.to_string(),
                    branch_id: branch_id.clone(),
                    reply: reply_tx,
                })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };

    // 3. Load character
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

    // 4. Load persona
    let persona = match &conversation.creator_persona_id {
        Some(pid) => {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadPersonas { reply: reply_tx })
                .map_err(|e| e.to_string())?;
            let personas = reply_rx
                .await
                .map_err(|e| format!("DB worker dropped: {}", e))?
                .map_err(|e| e.to_string())?;
            personas.into_iter().find(|p| p.id == *pid)
        }
        None => None,
    };

    // 5. Load provider
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
            return Ok(None);
        }
    };

    // 6. Load API key
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
            return Ok(None);
        }
    };

    // 7. Load character-assigned lorebooks
    let character_lorebooks: Vec<crate::domain::lorebook::Lorebook> = if let Some(ref ch) = character {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadCharacterLorebooks {
            character_id: ch.id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let character_lorebook_ids: Vec<String> = character_lorebooks.iter().map(|lb| lb.id.clone()).collect();

    // 8. Load conversation-assigned lorebooks
    let conversation_lorebooks: Vec<crate::domain::lorebook::Lorebook> = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadConversationLorebooks {
            conversation_id: conversation_id.to_string(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };
    let conversation_lorebook_ids: Vec<String> = conversation_lorebooks.iter().map(|lb| lb.id.clone()).collect();

    // 9. Load entries for all assigned lorebooks (character + conversation)
    let all_lorebook_ids: Vec<String> = character_lorebook_ids
        .iter()
        .chain(conversation_lorebook_ids.iter())
        .cloned()
        .collect();
    let lorebook_entries: Vec<crate::domain::lorebook::LorebookEntry> = if !all_lorebook_ids.is_empty() {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadLorebookEntriesForLorebooks {
            lorebook_ids: all_lorebook_ids,
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    // 10. Load regex scripts for the character
    let regex_scripts: Vec<RegexScript> = if let Some(ref ch) = character {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadRegexScripts {
            character_id: ch.id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    Ok(Some(ChatContext {
        character,
        persona,
        provider: provider.clone(),
        active_model_name: conversation.model_name.as_deref().unwrap_or(&provider.model_name).to_string(),
        messages,
        api_key,
        lorebook_entries,
        character_lorebook_ids,
        conversation_lorebook_ids,
        regex_scripts,
    }))
}

/// Assembles a prompt using PromptEngine with budget awareness.
/// Returns (system_prompt, chat_history_messages, budget_info, lorebook_details).
fn assemble_prompt(
    ctx: &ChatContext,
    messages: &[DomainMessage],
) -> (String, Vec<rig_core::completion::Message>, BudgetInfo, Vec<LorebookActivationDetail>) {
    let token_svc = TokenCountService::new();
    let context_window = token_svc.resolve_context_window_size(&ctx.provider, &ctx.active_model_name);

    let assembled = match &ctx.character {
        Some(ch) => PromptEngine::assemble_with_budget(
            ch,
            ctx.persona.as_ref(),
            messages,
            &ctx.lorebook_entries,
            &token_svc,
            context_window,
        ),
        None => {
            // No character — use minimal fallback prompt
            let preamble = "You are a helpful assistant.".to_string();
            let chat_history: Vec<rig_core::completion::Message> = messages
                .iter()
                .filter(|m| !m.content.is_empty())
                .map(|msg| match msg.role {
                    MessageRole::User => rig_core::completion::Message::user(&msg.content),
                    MessageRole::Assistant => {
                        rig_core::completion::Message::assistant(&msg.content)
                    }
                    MessageRole::System => rig_core::completion::Message::system(&msg.content),
                })
                .collect();
            let preamble_tokens = token_svc.count_tokens(&preamble);
            let msg_tokens: usize = messages
                .iter()
                .map(|m| 4 + token_svc.count_tokens(&m.content))
                .sum();
            let tokens_used = preamble_tokens + msg_tokens;
            crate::service::prompt::AssembledPrompt {
                preamble,
                chat_history,
                context_utilization: if context_window > 0 {
                    (tokens_used as f64) / (context_window as f64)
                } else {
                    0.0
                },
                tokens_used,
                truncated_messages: 0,
                active_lorebook_count: 0,
                lorebook_details: Vec::new(),
            }
        }
    };

    let lorebook_details = assembled.lorebook_details;

    let budget = BudgetInfo {
        tokens_used: assembled.tokens_used,
        tokens_limit: context_window as usize,
        truncated_count: assembled.truncated_messages,
    };

    (assembled.preamble, assembled.chat_history, budget, lorebook_details)
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
    ai_message_id: String,
    branch_id: Option<String>,
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
        branch_id: branch_id.clone(),
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

    // 3. Load context
    let ctx = match load_chat_context(&db, &channel, &conversation_id, branch_id).await? {
        Some(c) => c,
        None => return Ok(()), // Error already sent via channel
    };

    // 4. Append user message and assemble prompt
    let mut all_messages = ctx.messages.clone();
    all_messages.push(user_message);
    let (system_prompt, chat_history, budget, lorebook_details) = assemble_prompt(&ctx, &all_messages);

    // 5. Resolve generation params and start streaming
    let params = AiService::from_provider_config(&ctx.provider);
    let regex_scripts = ctx.regex_scripts.clone();
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        *token_guard = Some(cancel_token.clone());
    }

    let provider_config = ctx.provider.clone();
    let api_key = ctx.api_key.clone();
    tokio::spawn(async move {
        AiService::stream_chat(
            &provider_config,
            &system_prompt,
            chat_history,
            &api_key,
            params,
            ai_message_id,
            channel,
            cancel_token,
            budget,
            regex_scripts,
            lorebook_details,
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
    ai_message_id: String,
    branch_id: Option<String>,
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
                    branch_id: branch_id.clone(),
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

    // Load context
    let ctx = match load_chat_context(&db, &channel, &conversation_id, branch_id.clone()).await? {
        Some(c) => c,
        None => return Ok(()),
    };

    // Reload messages after deletion
    let updated_messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
                    conversation_id: conversation_id.clone(),
                    branch_id: branch_id.clone(),
                    reply: reply_tx,
                })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };

    // Assemble prompt
    let (system_prompt, chat_history, budget, lorebook_details) = assemble_prompt(&ctx, &updated_messages);

    // Resolve generation params and start streaming
    let params = AiService::from_provider_config(&ctx.provider);
    let regex_scripts = ctx.regex_scripts.clone();
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        *token_guard = Some(cancel_token.clone());
    }

    let provider_config = ctx.provider.clone();
    let api_key = ctx.api_key.clone();
    tokio::spawn(async move {
        AiService::stream_chat(
            &provider_config,
            &system_prompt,
            chat_history,
            &api_key,
            params,
            ai_message_id,
            channel,
            cancel_token,
            budget,
            regex_scripts,
            lorebook_details,
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
    conversation_id: String,
    ai_message_id: String,
    branch_id: Option<String>,
) -> Result<(), String> {
    // Cancel any existing stream
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = token_guard.take() {
            existing.cancel();
        }
    }

    // 1. Find the message directly using conversation_id
    let sort_order = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
                    conversation_id: conversation_id.clone(),
                    branch_id: branch_id.clone(),
                    reply: reply_tx,
                })
        .map_err(|e| e.to_string())?;
        let msgs = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
        msgs.iter()
            .find(|m| m.id == message_id)
            .map(|m| m.sort_order)
            .ok_or_else(|| "Message not found".to_string())?
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
                    branch_id: branch_id.clone(),
                    reply: reply_tx,
                })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?;
    }

    // 4. Load context
    let ctx = match load_chat_context(&db, &channel, &conversation_id, branch_id.clone()).await? {
        Some(c) => c,
        None => return Ok(()),
    };

    // 5. Reload messages after edit and truncation
    let updated_messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
                    conversation_id: conversation_id.clone(),
                    branch_id: branch_id.clone(),
                    reply: reply_tx,
                })
        .map_err(|e| e.to_string())?;
        reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {}", e))?
            .map_err(|e| e.to_string())?
    };

    // 6. Assemble prompt
    let (system_prompt, chat_history, budget, lorebook_details) = assemble_prompt(&ctx, &updated_messages);

    // 7. Resolve generation params and start streaming
    let params = AiService::from_provider_config(&ctx.provider);
    let regex_scripts = ctx.regex_scripts.clone();
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock().map_err(|e| e.to_string())?;
        *token_guard = Some(cancel_token.clone());
    }

    let provider_config = ctx.provider.clone();
    let api_key = ctx.api_key.clone();
    tokio::spawn(async move {
        AiService::stream_chat(
            &provider_config,
            &system_prompt,
            chat_history,
            &api_key,
            params,
            ai_message_id,
            channel,
            cancel_token,
            budget,
            regex_scripts,
            lorebook_details,
        );
    });

    Ok(())
}
