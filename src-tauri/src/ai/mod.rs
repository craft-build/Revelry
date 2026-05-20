use crate::domain::conversation::{Message as DomainMessage, MessageRole};
use crate::domain::lorebook::LorebookActivationDetail;
use crate::domain::provider::{ProviderConfig, ProviderType};
use crate::domain::regex_script::RegexScript;

use futures::StreamExt;
use rig_core::agent::{Agent, MultiTurnStreamItem};
use rig_core::client::CompletionClient;
use rig_core::completion::{CompletionModel, GetTokenUsage, Message};
use rig_core::streaming::{StreamedAssistantContent, StreamingChat};
use serde::Serialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

/// Generation parameters resolved from the provider defaults (D-03).
/// Passed to AiService::stream_chat to configure the AI request.
#[derive(Clone, Debug, Default)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub stop_sequences: Option<Vec<String>>,
}

/// Budget info from PromptEngine to include in StreamChunk::Complete.
#[derive(Clone, Copy, Debug, Default)]
pub struct BudgetInfo {
    pub tokens_used: usize,
    pub tokens_limit: usize,
    pub truncated_count: usize,
}

/// A chunk of data from a streaming AI response.
/// Serialized via Tauri Channel to the frontend using tagged enum pattern.
#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum StreamChunk {
    Token {
        content: String,
    },
    Complete {
        message_id: String,
        content: String,
        stop_reason: String,
        tokens_used: usize,
        tokens_limit: usize,
        truncated_count: usize,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        lorebook_details: Vec<LorebookActivationDetail>,
    },
    Error {
        error: String,
        error_type: String,
    },
}

/// Returns true if the given provider supports the named additional parameter.
/// Per D-02: silently skip unsupported params per-provider.
fn provider_supports_param(provider_type: &ProviderType, param_name: &str) -> bool {
    match param_name {
        // frequency_penalty and presence_penalty: not supported by Anthropic, Gemini, Cohere
        "frequency_penalty" | "presence_penalty" => !matches!(
            provider_type,
            ProviderType::Anthropic | ProviderType::Gemini | ProviderType::Cohere
        ),
        // top_p and stop_sequences are supported by all providers we use
        "top_p" | "stop" => true,
        _ => false,
    }
}

/// Maps error substrings to UI-SPEC error_type categories.
/// Returns one of: context_length_exceeded, authentication_failed,
/// rate_limited, network_error, provider_error.
fn classify_error(err: &str) -> String {
    let err_lower = err.to_lowercase();
    if err_lower.contains("context")
        || err_lower.contains("max_tokens")
        || err_lower.contains("token limit")
    {
        "context_length_exceeded".to_string()
    } else if err_lower.contains("auth")
        || err_lower.contains("api key")
        || err_lower.contains("unauthorized")
        || err_lower.contains("401")
    {
        "authentication_failed".to_string()
    } else if err_lower.contains("rate") || err_lower.contains("429") || err_lower.contains("quota")
    {
        "rate_limited".to_string()
    } else if err_lower.contains("network")
        || err_lower.contains("connection")
        || err_lower.contains("timeout")
        || err_lower.contains("dns")
    {
        "network_error".to_string()
    } else {
        "provider_error".to_string()
    }
}

/// Service for AI operations.
/// Provides utilities for building prompts and handling streaming responses.
pub struct AiService;

/// Helper macro to build a rig agent from a provider client with consistent parameter handling.
/// Each provider arm follows the same pattern: client → agent(model) → preamble → params → build.
macro_rules! build_agent_and_stream {
    ($client:expr, $model_name:expr, $system_prompt:expr, $params:expr, $additional:expr, $prompt:expr, $history:expr, $channel:expr, $message_id:expr, $cancel:expr, $budget:expr, $regex_scripts:expr, $lorebook_details:expr) => {{
        let mut builder = $client.agent($model_name).preamble($system_prompt);
        if let Some(temp) = $params.temperature {
            builder = builder.temperature(temp);
        }
        if let Some(max) = $params.max_tokens {
            builder = builder.max_tokens(max);
        }
        if !$additional.is_empty() {
            builder = builder.additional_params(serde_json::Value::Object($additional.clone()));
        }
        let agent = builder.build();
        AiService::stream_with_agent(
            &agent,
            $prompt,
            $history,
            $channel,
            $message_id,
            $cancel,
            $budget,
            $regex_scripts,
            $lorebook_details,
        )
        .await;
    }};
}

impl AiService {
    /// Stream an AI chat response via a Tauri Channel.
    /// Spawns a tokio task that processes the stream and sends StreamChunk events.
    /// The CancellationToken allows the caller to stop the stream mid-generation.
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
    ) {
        let provider_type = provider.provider_type.clone();
        let base_url = provider.base_url.clone();
        let model_name = provider.model_name.clone();
        let system = system_prompt.to_string();
        let key = api_key.to_string();

        // Last message is the active prompt, rest is chat history
        let (prompt, history) = Self::split_last_prompt(chat_history);

        tokio::spawn(async move {
            if let Err(e) = Self::run_stream(
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
            )
            .await
            {
                let _ = channel.send(StreamChunk::Error {
                    error: e.clone(),
                    error_type: classify_error(&e),
                });
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
    ) -> Result<(), String> {
        // Build provider-specific additional_params JSON (D-02)
        let additional = Self::build_additional_params(provider_type, params);

        match provider_type {
            ProviderType::OpenAI => {
                let client = Self::build_openai_client(api_key, base_url)?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Anthropic => {
                let client = rig_core::providers::anthropic::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Ollama => {
                let url = base_url.unwrap_or("http://localhost:11434");
                let client = rig_core::providers::ollama::Client::builder()
                    .api_key(String::new())
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Gemini => {
                let client = rig_core::providers::gemini::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Cohere => {
                let client = rig_core::providers::cohere::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Mistral => {
                let client = rig_core::providers::mistral::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Groq => {
                let client = rig_core::providers::groq::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::DeepSeek => {
                let client = rig_core::providers::deepseek::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::OpenRouter => {
                let client = rig_core::providers::openrouter::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Xai => {
                let client = rig_core::providers::xai::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Perplexity => {
                let client = rig_core::providers::perplexity::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::Together => {
                let client = rig_core::providers::together::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::AzureOpenAI => {
                let url = base_url.ok_or("Base URL required for Azure OpenAI")?;
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?
                    .completions_api();
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::HuggingFace => {
                let client = rig_core::providers::huggingface::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
            ProviderType::CustomOpenAICompatible => {
                let url = base_url.ok_or("Base URL required for custom provider")?;
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?
                    .completions_api();
                build_agent_and_stream!(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    history,
                    channel,
                    message_id,
                    cancel,
                    budget.clone(),
                    regex_scripts.clone(),
                    lorebook_details.clone()
                );
            }
        }
        Ok(())
    }

    /// Build provider-specific additional_params JSON for non-native parameters (D-02).
    /// Only includes parameters the provider actually supports.
    fn build_additional_params(
        provider_type: &ProviderType,
        params: &GenerationParams,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut additional = serde_json::Map::new();

        if let Some(top_p) = params.top_p {
            if provider_supports_param(provider_type, "top_p") {
                additional.insert("top_p".into(), serde_json::json!(top_p));
            }
        }

        if let Some(fp) = params.frequency_penalty {
            if provider_supports_param(provider_type, "frequency_penalty") {
                additional.insert("frequency_penalty".into(), serde_json::json!(fp));
            }
        }

        if let Some(pp) = params.presence_penalty {
            if provider_supports_param(provider_type, "presence_penalty") {
                additional.insert("presence_penalty".into(), serde_json::json!(pp));
            }
        }

        if let Some(ref stop) = params.stop_sequences {
            if provider_supports_param(provider_type, "stop") {
                additional.insert("stop".into(), serde_json::json!(stop));
            }
        }

        additional
    }

    /// Builds an OpenAI client, optionally with a custom base URL.
    fn build_openai_client(
        api_key: &str,
        base_url: Option<&str>,
    ) -> Result<rig_core::providers::openai::Client, String> {
        if let Some(url) = base_url {
            rig_core::providers::openai::Client::builder()
                .api_key(api_key)
                .base_url(url)
                .build()
                .map_err(|e| e.to_string())
        } else {
            rig_core::providers::openai::Client::builder()
                .api_key(api_key)
                .build()
                .map_err(|e| e.to_string())
        }
    }

    /// Generic streaming helper that works with any rig completion model.
    /// Uses rig-core 0.37 API: StreamAssistantItem(Text) for tokens,
    /// FinalResponse for completion.
    /// Supports cancellation via CancellationToken with tokio::select!.
    async fn stream_with_agent<M>(
        agent: &Agent<M>,
        prompt: Message,
        history: Vec<Message>,
        channel: &Channel<StreamChunk>,
        message_id: &str,
        cancel: &CancellationToken,
        budget: BudgetInfo,
        regex_scripts: Vec<RegexScript>,
        lorebook_details: Vec<LorebookActivationDetail>,
    ) where
        M: CompletionModel + 'static,
        M::StreamingResponse: GetTokenUsage,
    {
        let mut stream = agent.stream_chat(prompt, history).await;

        let msg_id = message_id.to_string();
        let channel_clone = channel.clone();
        let cancel_clone = cancel.clone();

        tokio::select! {
            _ = cancel_clone.cancelled() => {
                // Stream was cancelled — send partial Complete with cancelled stop_reason
                let _ = channel_clone.send(StreamChunk::Complete {
                    message_id: msg_id,
                    content: String::new(),
                    stop_reason: "cancelled".to_string(),
                    tokens_used: budget.tokens_used,
                    tokens_limit: budget.tokens_limit,
                    truncated_count: budget.truncated_count,
                    lorebook_details: Vec::new(),
                });
            }
            _ = async {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(MultiTurnStreamItem::StreamAssistantItem(
                            StreamedAssistantContent::Text(text),
                        )) => {
                            if channel_clone
                                .send(StreamChunk::Token {
                                    content: text.text.clone(),
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Ok(MultiTurnStreamItem::FinalResponse(res)) => {
                            // Apply regex transforms to the completed content
                            let raw_content = res.response().to_string();
                            let transformed_content = if !regex_scripts.is_empty() {
                                crate::service::regex::apply_regex_scripts(&raw_content, &regex_scripts)
                            } else {
                                raw_content
                            };

                            let _ = channel_clone.send(StreamChunk::Complete {
                                message_id: msg_id.clone(),
                                content: transformed_content,
                                stop_reason: "end".to_string(),
                                tokens_used: budget.tokens_used,
                                tokens_limit: budget.tokens_limit,
                                truncated_count: budget.truncated_count,
                                lorebook_details,
                            });
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let err_str = e.to_string();
                            let _ = channel_clone.send(StreamChunk::Error {
                                error: err_str.clone(),
                                error_type: classify_error(&err_str),
                            });
                            break;
                        }
                    }
                }
            } => {}
        }
    }

    /// Resolves GenerationParams from a ProviderConfig's default_params.
    /// Maps the provider domain's GenerationParams (i64 max_tokens) to the
    /// ai module's GenerationParams (u64 max_tokens).
    pub fn from_provider_config(provider: &ProviderConfig) -> GenerationParams {
        match &provider.default_params {
            Some(params) => GenerationParams {
                temperature: params.temperature,
                top_p: params.top_p,
                max_tokens: params
                    .max_tokens
                    .and_then(|v| if v > 0 { Some(v as u64) } else { None }),
                frequency_penalty: params.frequency_penalty,
                presence_penalty: params.presence_penalty,
                stop_sequences: None,
            },
            None => GenerationParams::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── split_last_prompt tests ──

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

    // ── provider_supports_param tests ──

    #[test]
    fn test_provider_supports_param_openai() {
        assert!(provider_supports_param(
            &ProviderType::OpenAI,
            "frequency_penalty"
        ));
        assert!(provider_supports_param(
            &ProviderType::OpenAI,
            "presence_penalty"
        ));
        assert!(provider_supports_param(&ProviderType::OpenAI, "top_p"));
        assert!(provider_supports_param(&ProviderType::OpenAI, "stop"));
    }

    #[test]
    fn test_provider_supports_param_anthropic() {
        assert!(!provider_supports_param(
            &ProviderType::Anthropic,
            "frequency_penalty"
        ));
        assert!(!provider_supports_param(
            &ProviderType::Anthropic,
            "presence_penalty"
        ));
        assert!(provider_supports_param(&ProviderType::Anthropic, "top_p"));
        assert!(provider_supports_param(&ProviderType::Anthropic, "stop"));
    }

    #[test]
    fn test_provider_supports_param_gemini() {
        assert!(!provider_supports_param(
            &ProviderType::Gemini,
            "frequency_penalty"
        ));
        assert!(!provider_supports_param(
            &ProviderType::Gemini,
            "presence_penalty"
        ));
        assert!(provider_supports_param(&ProviderType::Gemini, "top_p"));
    }

    #[test]
    fn test_provider_supports_param_cohere() {
        assert!(!provider_supports_param(
            &ProviderType::Cohere,
            "frequency_penalty"
        ));
        assert!(!provider_supports_param(
            &ProviderType::Cohere,
            "presence_penalty"
        ));
    }

    // ── classify_error tests ──

    #[test]
    fn test_classify_error_context_length() {
        assert_eq!(
            classify_error("The context length was exceeded"),
            "context_length_exceeded"
        );
        assert_eq!(
            classify_error("max_tokens limit reached"),
            "context_length_exceeded"
        );
        assert_eq!(
            classify_error("Token limit exceeded for this model"),
            "context_length_exceeded"
        );
    }

    #[test]
    fn test_classify_error_authentication() {
        assert_eq!(
            classify_error("Authentication failed"),
            "authentication_failed"
        );
        assert_eq!(
            classify_error("Invalid api key provided"),
            "authentication_failed"
        );
        assert_eq!(
            classify_error("Unauthorized access (401)"),
            "authentication_failed"
        );
    }

    #[test]
    fn test_classify_error_rate_limited() {
        assert_eq!(classify_error("Rate limit exceeded"), "rate_limited");
        assert_eq!(classify_error("429 Too Many Requests"), "rate_limited");
        assert_eq!(
            classify_error("Quota exceeded for this month"),
            "rate_limited"
        );
    }

    #[test]
    fn test_classify_error_network() {
        assert_eq!(classify_error("Network error occurred"), "network_error");
        assert_eq!(
            classify_error("Connection refused by server"),
            "network_error"
        );
        assert_eq!(
            classify_error("Timeout waiting for response"),
            "network_error"
        );
        assert_eq!(classify_error("DNS resolution failed"), "network_error");
    }

    #[test]
    fn test_classify_error_provider_default() {
        assert_eq!(
            classify_error("Something unexpected happened"),
            "provider_error"
        );
        assert_eq!(classify_error("Internal server error"), "provider_error");
    }

    // ── build_additional_params tests ──

    #[test]
    fn test_build_additional_params_provider_filtering() {
        let params = GenerationParams {
            temperature: Some(0.8),
            top_p: Some(0.9),
            max_tokens: Some(1024),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
            stop_sequences: Some(vec!["END".to_string()]),
        };

        // OpenAI should get all additional params
        let openai_additional = AiService::build_additional_params(&ProviderType::OpenAI, &params);
        assert!(openai_additional.contains_key("top_p"));
        assert!(openai_additional.contains_key("frequency_penalty"));
        assert!(openai_additional.contains_key("presence_penalty"));
        assert!(openai_additional.contains_key("stop"));

        // Anthropic should skip frequency/presence penalty
        let anthropic_additional =
            AiService::build_additional_params(&ProviderType::Anthropic, &params);
        assert!(anthropic_additional.contains_key("top_p"));
        assert!(!anthropic_additional.contains_key("frequency_penalty"));
        assert!(!anthropic_additional.contains_key("presence_penalty"));
        assert!(anthropic_additional.contains_key("stop"));

        // temperature and max_tokens should NOT be in additional params (native AgentBuilder)
        assert!(!openai_additional.contains_key("temperature"));
        assert!(!openai_additional.contains_key("max_tokens"));
    }

    // ── StreamChunk serde tests ──

    #[test]
    fn test_stream_chunk_serde_token() {
        let chunk = StreamChunk::Token {
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&chunk).expect("serialize failed");
        // Should produce: {"event":"token","data":{"content":"Hello"}}
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(parsed["event"], "token");
        assert_eq!(parsed["data"]["content"], "Hello");
    }

    #[test]
    fn test_stream_chunk_serde_complete() {
        let chunk = StreamChunk::Complete {
            message_id: "msg-123".to_string(),
            content: "Full response".to_string(),
            stop_reason: "end".to_string(),
            tokens_used: 4200,
            tokens_limit: 128000,
            truncated_count: 2,
            lorebook_details: Vec::new(),
        };
        let json = serde_json::to_string(&chunk).expect("serialize failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(parsed["event"], "complete");
        assert_eq!(parsed["data"]["messageId"], "msg-123");
        assert_eq!(parsed["data"]["tokensUsed"], 4200);
        assert_eq!(parsed["data"]["tokensLimit"], 128000);
        assert_eq!(parsed["data"]["truncatedCount"], 2);
        // Empty lorebook_details should be skipped by serde skip_serializing_if
        assert!(
            parsed["data"].get("lorebookDetails").is_none()
                || parsed["data"]["lorebookDetails"]
                    .as_array()
                    .unwrap()
                    .is_empty()
        );
    }

    #[test]
    fn test_stream_chunk_serde_error() {
        let chunk = StreamChunk::Error {
            error: "Something went wrong".to_string(),
            error_type: "provider_error".to_string(),
        };
        let json = serde_json::to_string(&chunk).expect("serialize failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(parsed["event"], "error");
        assert_eq!(parsed["data"]["error"], "Something went wrong");
        assert_eq!(parsed["data"]["errorType"], "provider_error");
    }

    // ── GenerationParams tests ──

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert!(params.temperature.is_none());
        assert!(params.top_p.is_none());
        assert!(params.max_tokens.is_none());
        assert!(params.frequency_penalty.is_none());
        assert!(params.presence_penalty.is_none());
        assert!(params.stop_sequences.is_none());
    }

    // ── from_provider_config tests ──

    #[test]
    fn test_from_provider_config_no_defaults() {
        let provider = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        let params = AiService::from_provider_config(&provider);
        assert!(params.temperature.is_none());
        assert!(params.max_tokens.is_none());
    }

    #[test]
    fn test_from_provider_config_with_defaults() {
        let mut provider = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        provider.default_params = Some(crate::domain::provider::GenerationParams {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(2048),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
        });
        let params = AiService::from_provider_config(&provider);
        assert_eq!(params.temperature, Some(0.7));
        assert_eq!(params.top_p, Some(0.9));
        assert_eq!(params.max_tokens, Some(2048u64));
        assert_eq!(params.frequency_penalty, Some(0.5));
        assert_eq!(params.presence_penalty, Some(0.3));
    }

    #[test]
    fn test_from_provider_config_zero_max_tokens() {
        let mut provider = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        provider.default_params = Some(crate::domain::provider::GenerationParams {
            temperature: Some(0.7),
            top_p: None,
            max_tokens: Some(0),
            frequency_penalty: None,
            presence_penalty: None,
        });
        let params = AiService::from_provider_config(&provider);
        // max_tokens of 0 should map to None (unset)
        assert!(params.max_tokens.is_none());
    }
}
