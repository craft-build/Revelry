use jiff::Timestamp;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::ai::{AiService, BudgetInfo, GenerationParams, MessagePersistence, StreamChunk};
use crate::db::{DbCommand, DbWorker};
use crate::domain::conversation::{Message as DomainMessage, MessageBranch, MessageRole};
use crate::domain::lorebook::LorebookActivationDetail;
use crate::domain::regex_script::RegexScript;
use crate::error::AppError;
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
    /// Lorebook entries from character + conversation assigned lorebooks,
    /// merged with entries parsed from the character's embedded `character_book`.
    /// Junction-table (explicitly assigned) entries win on id collision.
    lorebook_entries: Vec<crate::domain::lorebook::LorebookEntry>,
    /// Lorebook id → display name, for activation-detail reporting.
    /// Covers junction-table lorebooks and the embedded character book.
    lorebook_name_map: std::collections::HashMap<String, String>,
    /// Regex scripts for the character.
    regex_scripts: Vec<RegexScript>,
    /// Generation preset assigned to the conversation (if any).
    generation_preset: Option<crate::domain::generation::GenerationPreset>,
    /// Per-conversation max output token override.
    override_max_tokens: Option<i64>,
}

/// Resolves generation parameters from provider defaults → preset → per-conversation override.
/// Note: `max_tokens` is NOT set from the user's response-length preference — it stays at the
/// provider default so the API never hard-truncates mid-sentence. The user's desired response
/// length is injected as a prompt instruction instead (see `assemble_prompt`).
fn resolve_generation_params(
    provider: &crate::domain::provider::ProviderConfig,
    preset: Option<&crate::domain::generation::GenerationPreset>,
    _override_max_tokens: Option<i64>,
) -> GenerationParams {
    // Step 1: Start from provider defaults (this sets API max_tokens as a safety ceiling)
    let mut params = AiService::from_provider_config(provider);

    // Step 2: Apply preset overrides for sampling params (temperature, top_p, penalties)
    //         but NOT max_tokens — that's a prompt instruction, not an API cap.
    if let Some(p) = preset {
        params.temperature = Some(p.temperature);
        params.top_p = Some(p.top_p);
        // max_tokens intentionally NOT applied from preset to API
        params.frequency_penalty = Some(p.frequency_penalty);
        params.presence_penalty = Some(p.presence_penalty);
        if !p.stop_sequences.is_empty() {
            params.stop_sequences = Some(p.stop_sequences.clone());
        }
    }

    // Step 3: Per-conversation override_max_tokens is also a prompt instruction, not an API cap.
    // No changes to params.max_tokens here.

    params
}

/// Resolves the target response length (in tokens) for prompt injection.
/// Returns None if no preference is set (no instruction will be injected).
/// Resolution order: override_max_tokens → preset.max_tokens → None
fn resolve_target_response_length(
    preset: Option<&crate::domain::generation::GenerationPreset>,
    override_max_tokens: Option<i64>,
) -> Option<u64> {
    // Per-conversation override takes highest precedence
    if let Some(override_val) = override_max_tokens {
        if override_val > 0 {
            return Some(override_val as u64);
        }
    }
    // Then preset
    if let Some(p) = preset {
        if p.max_tokens > 0 {
            return Some(p.max_tokens as u64);
        }
    }
    None
}

/// Loads all context needed for prompt assembly and streaming.
/// Returns Ok(Some(ChatContext)) on success, or sends an error StreamChunk and returns Ok(None).
async fn load_chat_context(
    db: &State<'_, DbWorker>,
    channel: &Channel<StreamChunk>,
    conversation_id: &str,
    branch_id: Option<String>,
) -> Result<Option<ChatContext>, AppError> {
    // 1. Load conversation
    let conversation = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadConversations { reply: reply_tx })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        let conversations = reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
        conversations
            .into_iter()
            .find(|c| c.id == conversation_id)
            .ok_or_else(|| AppError::not_found("Conversation not found"))?
    };

    // 2. Load messages
    let messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
            conversation_id: conversation_id.to_string(),
            branch_id: branch_id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    // 3. Load character
    let character = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadCharacters { reply: reply_tx })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        let characters = reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
        characters
            .into_iter()
            .find(|c| c.id == conversation.character_id)
    };

    // 4. Load persona
    let persona = match &conversation.creator_persona_id {
        Some(pid) => {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadPersonas { reply: reply_tx })
                .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            let personas = reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?;
            personas.into_iter().find(|p| p.id == *pid)
        }
        None => None,
    };

    // 5. Load provider
    let provider = match &conversation.provider_id {
        Some(pid) => {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadProviders { reply: reply_tx })
                .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            let providers = reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?;
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
    .map_err(|e| AppError::keyring(format!("Keyring task failed: {}", e)))?;

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
    let character_lorebooks: Vec<crate::domain::lorebook::Lorebook> =
        if let Some(ref ch) = character {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadCharacterLorebooks {
                character_id: ch.id.clone(),
                reply: reply_tx,
            })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            Vec::new()
        };
    let character_lorebook_ids: Vec<String> =
        character_lorebooks.iter().map(|lb| lb.id.clone()).collect();

    // 8. Load conversation-assigned lorebooks
    let conversation_lorebooks: Vec<crate::domain::lorebook::Lorebook> = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadConversationLorebooks {
            conversation_id: conversation_id.to_string(),
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?
    };
    let conversation_lorebook_ids: Vec<String> = conversation_lorebooks
        .iter()
        .map(|lb| lb.id.clone())
        .collect();

    // 9. Load entries for all assigned lorebooks (character + conversation)
    let all_lorebook_ids: Vec<String> = character_lorebook_ids
        .iter()
        .chain(conversation_lorebook_ids.iter())
        .cloned()
        .collect();
    let mut lorebook_entries: Vec<crate::domain::lorebook::LorebookEntry> =
        if !all_lorebook_ids.is_empty() {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadLorebookEntriesForLorebooks {
                lorebook_ids: all_lorebook_ids,
                reply: reply_tx,
            })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            Vec::new()
        };

    // 9b. Merge entries from the character's embedded character_book.
    // Junction-table (explicitly assigned) entries win on id collision; embedded
    // entries are only added when no junction entry shares their id.
    let mut lorebook_name_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for lb in character_lorebooks
        .iter()
        .chain(conversation_lorebooks.iter())
    {
        lorebook_name_map.insert(lb.id.clone(), lb.name.clone());
    }
    let existing_entry_ids: std::collections::HashSet<String> =
        lorebook_entries.iter().map(|e| e.id.clone()).collect();
    if let Some(ref ch) = character {
        let embedded = ch.character_book_entries();
        if !embedded.is_empty() {
            // Register the embedded book's display name once.
            if let Some(first) = embedded.first() {
                lorebook_name_map
                    .entry(first.lorebook_id.clone())
                    .or_insert_with(|| first.lorebook_id.clone());
            }
            for entry in embedded {
                if !existing_entry_ids.contains(&entry.id) {
                    lorebook_entries.push(entry);
                }
            }
        }
    }

    // 10. Load regex scripts for the character
    let regex_scripts: Vec<RegexScript> = if let Some(ref ch) = character {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadRegexScripts {
            character_id: ch.id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        Vec::new()
    };

    // 11. Load generation preset if assigned to the conversation
    let generation_preset = match &conversation.generation_preset_id {
        Some(pid) => {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::LoadPresets { reply: reply_tx })
                .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            let presets = reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?;
            presets.into_iter().find(|p| p.id == *pid)
        }
        None => None,
    };

    Ok(Some(ChatContext {
        character,
        persona,
        provider: provider.clone(),
        active_model_name: conversation
            .model_name
            .as_deref()
            .unwrap_or(&provider.model_name)
            .to_string(),
        messages,
        api_key,
        lorebook_entries,
        lorebook_name_map,
        regex_scripts,
        generation_preset,
        override_max_tokens: conversation.override_max_tokens,
    }))
}

/// Assembles a prompt using PromptEngine with budget awareness.
/// If `target_response_length` is Some, a response-length instruction is appended to the
/// system prompt preamble (e.g., "Keep your response around 300 tokens.").
/// Returns (system_prompt, chat_history_messages, budget_info, lorebook_details).
fn assemble_prompt(
    ctx: &ChatContext,
    messages: &[DomainMessage],
    target_response_length: Option<u64>,
) -> (
    String,
    Vec<rig::completion::Message>,
    BudgetInfo,
    Vec<LorebookActivationDetail>,
) {
    let token_svc = TokenCountService::new();
    let context_window =
        token_svc.resolve_context_window_size(&ctx.provider, &ctx.active_model_name);

    let assembled = match &ctx.character {
        Some(ch) => PromptEngine::assemble_with_budget(
            ch,
            ctx.persona.as_ref(),
            messages,
            &ctx.lorebook_entries,
            &ctx.lorebook_name_map,
            &token_svc,
            context_window,
        ),
        None => {
            // No character — use minimal fallback prompt
            let preamble = "You are a helpful assistant.".to_string();
            let chat_history: Vec<rig::completion::Message> = messages
                .iter()
                .filter(|m| !m.content.is_empty())
                .map(|msg| match msg.role {
                    MessageRole::User => rig::completion::Message::user(&msg.content),
                    MessageRole::Assistant => rig::completion::Message::assistant(&msg.content),
                    MessageRole::System => rig::completion::Message::system(&msg.content),
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
                tokens_used,
                truncated_messages: 0,
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

    let mut preamble = assembled.preamble;

    // Inject the most recent steering instruction into the preamble so it carries
    // the full weight of the system prompt. It is also present in chat_history as
    // a system message — the preamble injection is a reinforcement layer.
    let latest_steer = messages
        .iter()
        .rev()
        .find(|m| {
            m.role == MessageRole::System && m.content.starts_with("[OOC — OVERRIDE INSTRUCTION")
        })
        .map(|m| m.content.clone());
    if let Some(steer) = latest_steer {
        preamble.push_str("\n\n");
        preamble.push_str(&steer);
    }

    // Inject response-length instruction as a post-history directive.
    // This tells the AI to aim for a target length without hard-truncating via the API.
    // The instruction uses both token count and approximate word count for stronger
    // compliance, and places it as the final instruction in the preamble.
    if let Some(target) = target_response_length {
        // Rough token-to-word ratio: ~0.75 words per token for English
        let approx_words = (target as f64 * 0.75).round() as u64;
        let length_instruction = format!(
            "\n\n[IMPORTANT: Your response MUST be approximately {} tokens long (roughly {} words). \
             This is a strict length constraint. Do NOT write significantly more or fewer tokens than this. \
             Wrap up your response naturally as you approach this length.]",
            target, approx_words
        );
        preamble.push_str(&length_instruction);
    }

    (preamble, assembled.chat_history, budget, lorebook_details)
}

/// Saves an empty assistant placeholder message and returns a persistence
/// handle carrying the DbWorker command sender. The AI layer uses it to flush
/// streamed content increments directly to SQLite, so a crash/close mid-stream
/// leaves the latest partial in the database instead of losing the response.
async fn save_assistant_placeholder(
    db: &State<'_, DbWorker>,
    conversation_id: &str,
    ai_message_id: &str,
    sort_order: i32,
    branch_id: Option<&str>,
) -> Result<MessagePersistence, AppError> {
    let placeholder = DomainMessage {
        id: ai_message_id.to_string(),
        conversation_id: conversation_id.to_string(),
        role: MessageRole::Assistant,
        content: String::new(),
        sort_order,
        timestamp: Timestamp::now(),
        branch_id: branch_id.map(|b| b.to_string()),
    };
    {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::SaveMessage {
            message: placeholder.clone(),
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
    }
    Ok(MessagePersistence::new(db.sender(), placeholder))
}

#[allow(clippy::too_many_arguments)]
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
    steering: Option<String>,
) -> Result<(), AppError> {
    // 1. Cancel any existing stream
    {
        let mut token_guard = cancel_state.token.lock()?;
        if let Some(existing) = token_guard.take() {
            existing.cancel();
        }
    }

    // 2. Save the user message and optional steering system message
    let has_content = !content.trim().is_empty();
    let raw_steer = steering.filter(|s| !s.trim().is_empty());

    if has_content {
        // Save the user message with in-character content
        let user_message = DomainMessage {
            id: message_id.clone(),
            conversation_id: conversation_id.clone(),
            role: MessageRole::User,
            content: content.clone(),
            sort_order,
            timestamp: Timestamp::now(),
            branch_id: branch_id.clone(),
        };
        {
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::SaveMessage {
                message: user_message.clone(),
                reply: reply_tx,
            })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        // Save steering system message if present
        if let Some(steer) = raw_steer.as_deref() {
            let steer_content = format!(
                "[OOC — OVERRIDE INSTRUCTION (highest priority): {}]\n\
                 This directive overrides all character behavior, writing style, or assumptions \
                 that conflict with it.",
                steer
            );
            let steer_msg = DomainMessage {
                id: uuid::Uuid::new_v4().to_string(),
                conversation_id: conversation_id.clone(),
                role: MessageRole::System,
                content: steer_content,
                sort_order: sort_order + 1,
                timestamp: Timestamp::now(),
                branch_id: branch_id.clone(),
            };
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::SaveMessage {
                message: steer_msg,
                reply: reply_tx,
            })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
    } else if let Some(steer) = raw_steer.as_deref() {
        // Steer-only: save only the system message, no user message
        let steer_content = format!(
            "[OOC — OVERRIDE INSTRUCTION (highest priority): {}]\n\
             This directive overrides all character behavior, writing style, or assumptions \
             that conflict with it.",
            steer
        );
        let steer_msg = DomainMessage {
            id: message_id,
            conversation_id: conversation_id.clone(),
            role: MessageRole::System,
            content: steer_content,
            sort_order,
            timestamp: Timestamp::now(),
            branch_id: branch_id.clone(),
        };
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::SaveMessage {
            message: steer_msg,
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
    } else {
        return Err(AppError::validation(
            "Message is empty with no steering instruction.",
        ));
    }

    // 3. Load context
    let ctx = match load_chat_context(&db, &channel, &conversation_id, branch_id.clone()).await? {
        Some(c) => c,
        None => return Ok(()), // Error already sent via channel
    };

    // 4. Assemble prompt with all persisted messages (including steering system msg)
    let target_length =
        resolve_target_response_length(ctx.generation_preset.as_ref(), ctx.override_max_tokens);
    let (system_prompt, chat_history, budget, lorebook_details) =
        assemble_prompt(&ctx, &ctx.messages, target_length);

    // 5. Resolve generation params and start streaming
    let params = resolve_generation_params(
        &ctx.provider,
        ctx.generation_preset.as_ref(),
        ctx.override_max_tokens,
    );
    let regex_scripts = ctx.regex_scripts.clone();

    // Save an assistant placeholder so the AI layer can flush streamed content
    // increments to the database (crash-safe persistence).
    let ai_sort_order = ctx
        .messages
        .iter()
        .map(|m| m.sort_order)
        .max()
        .unwrap_or(-1)
        + 1;
    let persistence = save_assistant_placeholder(
        &db,
        &conversation_id,
        &ai_message_id,
        ai_sort_order,
        branch_id.as_deref(),
    )
    .await?;

    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock()?;
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
            Some(persistence),
        );
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_generation(cancel_state: State<'_, CancelState>) -> Result<(), AppError> {
    let token = {
        let mut token_guard = cancel_state.token.lock()?;
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
) -> Result<Option<String>, AppError> {
    // Cancel any existing stream
    {
        let mut token_guard = cancel_state.token.lock()?;
        if let Some(existing) = token_guard.take() {
            existing.cancel();
        }
    }

    // Load messages for this conversation (current branch view)
    let messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
            conversation_id: conversation_id.clone(),
            branch_id: branch_id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    // Find the index of the last AI message in the current view.
    let last_ai_idx = messages
        .iter()
        .rposition(|m| m.role == MessageRole::Assistant);

    // Regenerate by forking a new branch from the message preceding the last
    // AI response, so the old response is preserved on the prior branch instead
    // of being hard-deleted. When the AI message is the first message in the
    // view (no predecessor to fork from), fall back to deleting it.
    let effective_branch_id: Option<String> = match last_ai_idx {
        Some(idx) if idx > 0 => {
            let parent_message_id = messages[idx - 1].id.clone();

            // Count existing branches for auto-labeling
            let count = {
                let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
                db.send(DbCommand::CountBranches {
                    conversation_id: conversation_id.clone(),
                    reply: reply_tx,
                })
                .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
                reply_rx
                    .await
                    .map_err(|e| {
                        AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e))
                    })?
                    .map_err(|e| AppError::Database(e.to_string()))?
            };
            let label = format!("Branch {}", count + 1);
            let branch = MessageBranch::new(conversation_id.clone(), parent_message_id, label);
            let new_branch_id = branch.id.clone();
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::CreateBranch {
                branch,
                reply: reply_tx,
            })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?;
            Some(new_branch_id)
        }
        Some(idx) => {
            // AI is the first message in the view — no predecessor to fork from.
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            db.send(DbCommand::DeleteMessage {
                id: messages[idx].id.clone(),
                reply: reply_tx,
            })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
            reply_rx
                .await
                .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
                .map_err(|e| AppError::Database(e.to_string()))?;
            branch_id
        }
        None => branch_id,
    };

    // Load context for the effective branch
    let ctx = match load_chat_context(&db, &channel, &conversation_id, effective_branch_id.clone())
        .await?
    {
        Some(c) => c,
        None => return Ok(effective_branch_id),
    };

    // Reload messages for the effective branch
    let updated_messages = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadMessages {
            conversation_id: conversation_id.clone(),
            branch_id: effective_branch_id.clone(),
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    // Assemble prompt
    let target_length =
        resolve_target_response_length(ctx.generation_preset.as_ref(), ctx.override_max_tokens);
    let (system_prompt, chat_history, budget, lorebook_details) =
        assemble_prompt(&ctx, &updated_messages, target_length);

    // Resolve generation params and start streaming
    let params = resolve_generation_params(
        &ctx.provider,
        ctx.generation_preset.as_ref(),
        ctx.override_max_tokens,
    );
    let regex_scripts = ctx.regex_scripts.clone();

    // Save an assistant placeholder so the AI layer can flush streamed content
    // increments to the database (crash-safe persistence).
    let ai_sort_order = updated_messages
        .iter()
        .map(|m| m.sort_order)
        .max()
        .unwrap_or(-1)
        + 1;
    let persistence = save_assistant_placeholder(
        &db,
        &conversation_id,
        &ai_message_id,
        ai_sort_order,
        effective_branch_id.as_deref(),
    )
    .await?;

    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock()?;
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
            Some(persistence),
        );
    });

    Ok(effective_branch_id)
}

#[allow(clippy::too_many_arguments)]
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
) -> Result<(), AppError> {
    // Cancel any existing stream
    {
        let mut token_guard = cancel_state.token.lock()?;
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
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        let msgs = reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
        msgs.iter()
            .find(|m| m.id == message_id)
            .map(|m| m.sort_order)
            .ok_or_else(|| AppError::not_found("Message not found"))?
    };

    // 2. Update message content
    {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::UpdateMessageContent {
            id: message_id,
            content: new_content,
            reply: reply_tx,
        })
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
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
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
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
        .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    // 6. Assemble prompt
    let target_length =
        resolve_target_response_length(ctx.generation_preset.as_ref(), ctx.override_max_tokens);
    let (system_prompt, chat_history, budget, lorebook_details) =
        assemble_prompt(&ctx, &updated_messages, target_length);

    // 7. Resolve generation params and start streaming
    let params = resolve_generation_params(
        &ctx.provider,
        ctx.generation_preset.as_ref(),
        ctx.override_max_tokens,
    );
    let regex_scripts = ctx.regex_scripts.clone();

    // Save an assistant placeholder so the AI layer can flush streamed content
    // increments to the database (crash-safe persistence).
    let ai_sort_order = updated_messages
        .iter()
        .map(|m| m.sort_order)
        .max()
        .unwrap_or(-1)
        + 1;
    let persistence = save_assistant_placeholder(
        &db,
        &conversation_id,
        &ai_message_id,
        ai_sort_order,
        branch_id.as_deref(),
    )
    .await?;

    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock()?;
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
            Some(persistence),
        );
    });

    Ok(())
}
