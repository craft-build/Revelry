use crate::domain::lorebook::LorebookActivationDetail;
use crate::domain::provider::{ProviderConfig, ProviderType};
use crate::domain::regex_script::RegexScript;

use futures::StreamExt;
use rig::agent::{Agent, MultiTurnStreamItem};
use rig::completion::Message;
use rig::streaming::{Item, StreamEvent};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::providers::{build_agent, with_native_client};
use super::util::classify_error;
use super::AiService;
use super::{BudgetInfo, GenerationParams, MessagePersistence, StreamChunk};

/// Callback invoked once after a chat stream finishes (success, error, or
/// cancellation). Used by the command layer for post-stream maintenance
/// (message embedding, summary regeneration) without touching stream internals.
pub type PostStreamHook = Box<dyn FnOnce() + Send + Sync>;

impl AiService {
    /// Stream an AI chat response via a Tauri Channel.
    /// Spawns a tokio task that processes the stream and sends StreamChunk events.
    /// The CancellationToken allows the caller to stop the stream mid-generation.
    /// `on_complete` (optional) runs after the stream task finishes.
    #[allow(clippy::too_many_arguments)]
    pub fn stream_chat(
        provider: &ProviderConfig,
        system_prompt: &str,
        chat_history: Vec<Message>,
        api_key: &str,
        params: GenerationParams,
        message_id: String,
        channel: Channel<StreamChunk>,
        cancel: CancellationToken,
        budget: BudgetInfo,
        regex_scripts: Vec<RegexScript>,
        lorebook_details: Vec<LorebookActivationDetail>,
        persistence: Option<MessagePersistence>,
        on_complete: Option<PostStreamHook>,
    ) {
        let provider_type = provider.provider_type.clone();
        let base_url = provider.base_url.clone();
        let model_name = provider.model_name.clone();
        let system = system_prompt.to_string();
        let key = api_key.to_string();

        // Last message is the active prompt, rest is chat history
        let (prompt, history) = Self::split_last_prompt(chat_history);

        tokio::spawn(async move {
            let result = Self::run_stream(
                &provider_type,
                base_url.as_deref(),
                &model_name,
                &system,
                prompt,
                history,
                &key,
                &params,
                &channel,
                &message_id,
                &cancel,
                budget,
                regex_scripts,
                lorebook_details,
                persistence,
            )
            .await;

            if let Err(e) = result {
                let _ = channel.send(StreamChunk::Error {
                    error: e.clone(),
                    error_type: classify_error(&e),
                });
            }

            // Post-stream maintenance (embedding, summary) — never fatal.
            if let Some(hook) = on_complete {
                hook();
            }
        });
    }

    /// Splits the last message off as the prompt, returns the remainder as history.
    fn split_last_prompt(messages: Vec<Message>) -> (Message, Vec<Message>) {
        let mut messages = messages;
        if messages.is_empty() {
            return (Message::user(""), Vec::new());
        }
        let prompt = messages.pop().unwrap_or_else(|| Message::user(""));
        (prompt, messages)
    }

    /// Dispatches to the correct rig provider client and runs the stream.
    #[allow(clippy::too_many_arguments)]
    async fn run_stream(
        provider_type: &ProviderType,
        base_url: Option<&str>,
        model_name: &str,
        system_prompt: &str,
        prompt: Message,
        history: Vec<Message>,
        api_key: &str,
        params: &GenerationParams,
        channel: &Channel<StreamChunk>,
        message_id: &str,
        cancel: &CancellationToken,
        budget: BudgetInfo,
        regex_scripts: Vec<RegexScript>,
        lorebook_details: Vec<LorebookActivationDetail>,
        persistence: Option<MessagePersistence>,
    ) -> Result<(), String> {
        let additional = Self::build_additional_params(provider_type, params);

        with_native_client!(provider_type, base_url, api_key, model_name, model, {
            let agent = build_agent(model, system_prompt, params, &additional);
            Self::stream_with_agent(
                &agent,
                prompt,
                history,
                channel,
                message_id,
                cancel,
                budget,
                regex_scripts,
                lorebook_details,
                persistence,
            )
            .await;
        });

        Ok(())
    }

    /// Generic streaming helper for any rig completion model. Streams
    /// StreamAssistantItem(Text) tokens and a FinalResponse; cancels via
    /// CancellationToken with tokio::select!.
    #[allow(clippy::too_many_arguments)]
    async fn stream_with_agent(
        agent: &Agent,
        prompt: Message,
        history: Vec<Message>,
        channel: &Channel<StreamChunk>,
        message_id: &str,
        cancel: &CancellationToken,
        budget: BudgetInfo,
        regex_scripts: Vec<RegexScript>,
        lorebook_details: Vec<LorebookActivationDetail>,
        persistence: Option<MessagePersistence>,
    ) {
        let mut stream = agent.prompt(prompt).history(history).stream();

        let msg_id = message_id.to_string();
        let channel_clone = channel.clone();
        let cancel_clone = cancel.clone();

        let mut accumulated = String::new();

        loop {
            tokio::select! {
                _ = cancel_clone.cancelled() => {
                    if let Some(p) = &persistence {
                        p.flush(&accumulated);
                    }
                    let _ = channel_clone.send(StreamChunk::Complete {
                        message_id: msg_id.clone(),
                        content: accumulated,
                        stop_reason: "cancelled".to_string(),
                        tokens_used: budget.tokens_used,
                        tokens_limit: budget.tokens_limit,
                        truncated_count: budget.truncated_count,
                        lorebook_details: Vec::new(),
                        cancelled: true,
                        memory: budget.memory,
                    });
                    return;
                }
                item = stream.next() => {
                    match item {
                        None => {
                            if let Some(p) = &persistence {
                                p.flush(&accumulated);
                            }
                            let _ = channel_clone.send(StreamChunk::Complete {
                                message_id: msg_id,
                                content: accumulated,
                                stop_reason: "end".to_string(),
                                tokens_used: budget.tokens_used,
                                tokens_limit: budget.tokens_limit,
                                truncated_count: budget.truncated_count,
                                lorebook_details,
                                cancelled: false,
                                memory: budget.memory,
                            });
                            return;
                        }
                        Some(Ok(MultiTurnStreamItem::StreamAssistantItem(
                            Item::Event(StreamEvent::Text { text, .. }),
                        ))) => {
                            accumulated.push_str(&text);
                            // Persist on every token (fire-and-forget) so a
                            // crash/close mid-stream leaves the latest partial
                            // in SQLite — the only loss window is the DbWorker's
                            // inherent processing lag, not an artificial delay.
                            if let Some(p) = &persistence {
                                p.flush(&accumulated);
                            }
                            if channel_clone
                                .send(StreamChunk::Token { content: text })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Some(Ok(MultiTurnStreamItem::FinalResponse(res))) => {
                            let raw_content = res.output();
                            let transformed_content = if !regex_scripts.is_empty() {
                                crate::service::regex::apply_regex_scripts(&raw_content, &regex_scripts)
                            } else {
                                raw_content
                            };
                            let final_content = if transformed_content.is_empty() {
                                accumulated
                            } else {
                                transformed_content
                            };
                            if let Some(p) = &persistence {
                                p.flush(&final_content);
                            }
                            // Extract response token usage from the provider
                            // when reported; otherwise estimate via tiktoken.
                            // The reported `tokens_used` is prompt budget +
                            // response tokens (total context consumed).
                            let usage = res.usage;
                            let response_tokens = usage
                                .output_tokens
                                .filter(|&n| n > 0)
                                .map(|n| n as usize)
                                .unwrap_or_else(|| {
                                    crate::service::token_count::TokenCountService::new()
                                        .count_tokens(&final_content)
                                });
                            let _ = channel_clone.send(StreamChunk::Complete {
                                message_id: msg_id,
                                content: final_content,
                                stop_reason: "end".to_string(),
                                tokens_used: budget.tokens_used.saturating_add(response_tokens),
                                tokens_limit: budget.tokens_limit,
                                truncated_count: budget.truncated_count,
                                lorebook_details,
                                cancelled: false,
                                memory: budget.memory,
                            });
                            return;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            let err_str = e.to_string();
                            let _ = channel_clone.send(StreamChunk::Error {
                                error: err_str.clone(),
                                error_type: classify_error(&err_str),
                            });
                            return;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_last_prompt_empty() {
        let (prompt, history) = AiService::split_last_prompt(vec![]);
        assert!(matches!(prompt, Message::User { .. }));
        assert!(history.is_empty());
    }

    #[test]
    fn test_split_last_prompt_single() {
        let msgs = vec![Message::user("hello")];
        let (prompt, history) = AiService::split_last_prompt(msgs);
        assert!(matches!(prompt, Message::User { .. }));
        assert!(history.is_empty());
    }

    #[test]
    fn test_split_last_prompt_multiple() {
        let msgs = vec![
            Message::user("first"),
            Message::assistant("second"),
            Message::user("third"),
        ];
        let (prompt, history) = AiService::split_last_prompt(msgs);
        assert!(matches!(prompt, Message::User { .. }));
        assert_eq!(history.len(), 2);
    }
}
