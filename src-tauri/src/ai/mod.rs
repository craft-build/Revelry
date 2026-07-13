use crate::domain::conversation::Message as DomainMessage;
use crate::domain::lorebook::LorebookActivationDetail;
use crate::domain::provider::{ProviderConfig, ProviderModelEntry, ProviderType};
use crate::domain::regex_script::RegexScript;

use futures::StreamExt;
use rig_core::agent::{Agent, MultiTurnStreamItem};
use rig_core::client::{CompletionClient, ModelListingClient};
use rig_core::completion::{CompletionModel, GetTokenUsage, Message};
use rig_core::streaming::{StreamedAssistantContent, StreamingChat};
use serde::Serialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

/// Generation parameters resolved from the provider defaults (D-03).
/// Passed to AiService::stream_chat to configure the AI request.
///
/// This is the runtime representation used by the AI layer. The domain-layer
/// `GenerationParams` (in `domain::provider`) uses `i64` for `max_tokens` to
/// match SQLite's INTEGER type; this type uses `u64` since API `max_tokens`
/// is always non-negative.
#[derive(Clone, Debug, Default)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub stop_sequences: Option<Vec<String>>,
}

impl From<crate::domain::provider::GenerationParams> for GenerationParams {
    fn from(params: crate::domain::provider::GenerationParams) -> Self {
        Self {
            temperature: params.temperature,
            top_p: params.top_p,
            max_tokens: params
                .max_tokens
                .and_then(|v| if v > 0 { Some(v as u64) } else { None }),
            frequency_penalty: params.frequency_penalty,
            presence_penalty: params.presence_penalty,
            stop_sequences: params.stop_sequences,
        }
    }
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

/// Partial character data returned by the generator. Creative fields only —
/// id, created_at, updated_at are assigned on save.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GeneratedCharacterData {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    pub system_prompt: String,
    pub post_history_instructions: String,
    pub creator_notes: String,
    pub tags: Vec<String>,
    pub alternate_greetings: Vec<String>,
}

/// A chunk of data from the character generation stream.
/// Mirrors the frontend GenerateCharacterChunk discriminated union exactly.
#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum GenerateCharacterChunk {
    Token { content: String },
    Complete { character: GeneratedCharacterData },
    Error { error: String, error_type: String },
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

/// System prompt for the character generation command.
/// Instructs the LLM to output a CharacterCardV2-compatible JSON object.
const GENERATE_CHARACTER_SYSTEM_PROMPT: &str = r#"You are a character creation assistant for an AI roleplay application. The user will describe a character idea, and you must generate a complete character card in JSON format.

Output ONLY valid JSON matching this schema — no explanation, no markdown, no commentary:

{
  "name": "Character's full name",
  "description": "A vivid 2-3 sentence description of the character's appearance, nature, and what makes them interesting",
  "personality": "Detailed personality description — traits, quirks, emotional patterns, how they behave with others",
  "scenario": "The starting scenario for the roleplay — set the scene, establish tension or intrigue, give the user a reason to engage",
  "first_mes": "The character's first message to the user — written in character, using *actions* and dialogue. Make it compelling and open-ended",
  "mes_example": "Example dialogue showing the character's speech patterns — use <START> to separate exchanges, {{user}} and {{char}} as placeholders",
  "system_prompt": "The system prompt that defines how the AI should portray this character — writing style, key behaviors, dos and don'ts",
  "post_history_instructions": "Optional instruction appended after chat history — leave empty string if not needed",
  "creator_notes": "Notes for the user about the character — suggested usage, tone, potential story directions",
  "tags": ["relevant", "descriptive", "tags"],
  "alternate_greetings": []
}

Rules:
- The "name" field is REQUIRED and must be non-empty
- "first_mes" should be at least 2-3 paragraphs, written in first person with *actions* in asterisks
- "personality" and "scenario" should be rich and detailed — aim for 3-5+ sentences each
- "mes_example" should demonstrate at least 2 exchanges
- "tags" should include 3-6 relevant genre/theme tags
- "alternate_greetings" can be empty array if not needed
- All string fields default to empty string if not applicable
- Be creative and specific — avoid generic templates"#;

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

    /// Fetch the list of available models from a provider's Models API.
    ///
    /// Routes to rig-core's native `list_models()` for the 7 providers that
    /// implement it (OpenAI, Anthropic, Ollama, Gemini, Mistral, DeepSeek,
    /// OpenRouter). For the OpenAI-compatible family that rig does not natively
    /// list (Groq, Together, xAI, Perplexity, HuggingFace, Cohere, and any
    /// custom OpenAI-compatible endpoint), it falls back to an OpenAI client
    /// pointed at the resolved base URL hitting `GET /models`. Azure OpenAI is
    /// not supported (different, deployment-based auth scheme).
    ///
    /// Results are sorted alphabetically by name and deduped.
    pub async fn list_provider_models(
        provider_type: &ProviderType,
        base_url: Option<&str>,
        api_key: &str,
    ) -> Result<Vec<ProviderModelEntry>, String> {
        use rig_core::model::ModelList;

        let list: ModelList = match provider_type {
            ProviderType::OpenAI => {
                let client = Self::build_openai_client(api_key, base_url)?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Anthropic => {
                let client = rig_core::providers::anthropic::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Ollama => {
                let url = base_url.unwrap_or("http://localhost:11434");
                let client = rig_core::providers::ollama::Client::builder()
                    .api_key(String::new())
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Gemini => {
                let client = rig_core::providers::gemini::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Mistral => {
                let client = rig_core::providers::mistral::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::DeepSeek => {
                let client = rig_core::providers::deepseek::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::OpenRouter => {
                let client = rig_core::providers::openrouter::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            // OpenAI-compatible fallback: these providers expose an
            // OpenAI-shaped `/models` endpoint even though rig does not
            // natively list models for them.
            ProviderType::Groq => {
                let url = base_url.unwrap_or("https://api.groq.com/openai/v1");
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Together => {
                let url = base_url.unwrap_or("https://api.together.xyz/v1");
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Xai => {
                let url = base_url.unwrap_or("https://api.x.ai/v1");
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Perplexity => {
                let url = base_url.unwrap_or("https://api.perplexity.ai");
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::HuggingFace => {
                let url = base_url.unwrap_or("https://api-inference.huggingface.co/v1");
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Cohere => {
                let url = base_url.unwrap_or("https://api.cohere.com/v1");
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::CustomOpenAICompatible => {
                let url = base_url.ok_or(
                    "Base URL is required to detect models for a custom OpenAI-compatible provider.",
                )?;
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::AzureOpenAI => {
                return Err(
                    "Model detection is not supported for Azure OpenAI. Azure uses deployment-based \
                     model naming and a different authentication scheme — add models manually."
                        .to_string(),
                );
            }
        };

        Ok(Self::model_list_to_entries(list))
    }

    /// Convert a rig `ModelList` into sorted, deduped `ProviderModelEntry` values.
    /// `id` becomes `name`; `context_length` (u32) becomes `context_window_size` (i64).
    fn model_list_to_entries(list: rig_core::model::ModelList) -> Vec<ProviderModelEntry> {
        let mut entries: Vec<ProviderModelEntry> = list
            .into_iter()
            .map(|m| ProviderModelEntry {
                name: m.id,
                context_window_size: m.context_length.map(|c| c as i64),
                enabled: true,
            })
            .collect();
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        entries.dedup_by(|a, b| a.name == b.name);
        entries
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
                            let raw_content = res.output.clone();
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
    /// Uses the `From<domain::provider::GenerationParams>` impl to convert
    /// the domain type (i64 max_tokens) to the AI type (u64 max_tokens).
    pub fn from_provider_config(provider: &ProviderConfig) -> GenerationParams {
        match &provider.default_params {
            Some(params) => params.clone().into(),
            None => GenerationParams::default(),
        }
    }

    /// Stream an AI-generated character card via a Tauri Channel.
    /// Sends the user's idea as a prompt with a system instruction to output
    /// CharacterCardV2-compatible JSON. Tokens are streamed for preview, then
    /// the full response is parsed into GeneratedCharacterData on completion.
    pub fn stream_generate_character(
        provider: &ProviderConfig,
        idea: &str,
        api_key: &str,
        params: GenerationParams,
        channel: Channel<GenerateCharacterChunk>,
        cancel: CancellationToken,
    ) {
        let provider_type = provider.provider_type.clone();
        let base_url = provider.base_url.clone();
        let model_name = provider.model_name.clone();
        let key = api_key.to_string();
        let user_prompt = idea.to_string();

        let system_prompt = GENERATE_CHARACTER_SYSTEM_PROMPT.to_string();

        tokio::spawn(async move {
            if let Err(e) = Self::run_generate_stream(
                &provider_type,
                base_url.as_deref(),
                &model_name,
                &system_prompt,
                &user_prompt,
                &key,
                &params,
                &channel,
                &cancel,
            )
            .await
            {
                let _ = channel.send(GenerateCharacterChunk::Error {
                    error: e.clone(),
                    error_type: classify_error(&e),
                });
            }
        });
    }

    /// Dispatches to the correct rig provider client and runs the generation stream.
    /// On completion, parses the accumulated JSON into GeneratedCharacterData.
    async fn run_generate_stream(
        provider_type: &ProviderType,
        base_url: Option<&str>,
        model_name: &str,
        system_prompt: &str,
        idea: &str,
        api_key: &str,
        params: &GenerationParams,
        channel: &Channel<GenerateCharacterChunk>,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        let additional = Self::build_additional_params(provider_type, params);

        let prompt = Message::user(idea);

        match provider_type {
            ProviderType::OpenAI => {
                let client = Self::build_openai_client(api_key, base_url)?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Anthropic => {
                let client = rig_core::providers::anthropic::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Ollama => {
                let url = base_url.unwrap_or("http://localhost:11434");
                let client = rig_core::providers::ollama::Client::builder()
                    .api_key(String::new())
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Gemini => {
                let client = rig_core::providers::gemini::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Cohere => {
                let client = rig_core::providers::cohere::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Mistral => {
                let client = rig_core::providers::mistral::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Groq => {
                let client = rig_core::providers::groq::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::DeepSeek => {
                let client = rig_core::providers::deepseek::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::OpenRouter => {
                let client = rig_core::providers::openrouter::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Xai => {
                let client = rig_core::providers::xai::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Perplexity => {
                let client = rig_core::providers::perplexity::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::Together => {
                let client = rig_core::providers::together::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::AzureOpenAI => {
                let url = base_url.ok_or("Base URL required for Azure OpenAI")?;
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?
                    .completions_api();
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::HuggingFace => {
                let client = rig_core::providers::huggingface::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
            ProviderType::CustomOpenAICompatible => {
                let url = base_url.ok_or("Base URL required for custom provider")?;
                let client = rig_core::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?
                    .completions_api();
                Self::build_and_stream_generation(
                    client,
                    model_name,
                    system_prompt,
                    params,
                    additional,
                    prompt,
                    channel,
                    cancel,
                )
                .await
            }
        }
    }

    /// Build a rig agent, stream the response, and parse it as GeneratedCharacterData.
    async fn build_and_stream_generation<C>(
        client: C,
        model_name: &str,
        system_prompt: &str,
        params: &GenerationParams,
        additional: serde_json::Map<String, serde_json::Value>,
        prompt: Message,
        channel: &Channel<GenerateCharacterChunk>,
        cancel: &CancellationToken,
    ) -> Result<(), String>
    where
        C: CompletionClient,
        <C as CompletionClient>::CompletionModel: 'static,
        <<C as CompletionClient>::CompletionModel as CompletionModel>::StreamingResponse:
            GetTokenUsage,
    {
        let mut builder = client.agent(model_name).preamble(system_prompt);
        if let Some(temp) = params.temperature {
            builder = builder.temperature(temp);
        }
        if let Some(max) = params.max_tokens {
            builder = builder.max_tokens(max);
        }
        if !additional.is_empty() {
            builder = builder.additional_params(serde_json::Value::Object(additional.clone()));
        }
        let agent = builder.build();

        let mut stream = agent.stream_chat(prompt, Vec::<Message>::new()).await;

        let channel_clone = channel.clone();
        let cancel_clone = cancel.clone();

        let mut accumulated = String::new();

        tokio::select! {
            _ = cancel_clone.cancelled() => {
                let _ = channel_clone.send(GenerateCharacterChunk::Error {
                    error: "Generation cancelled.".to_string(),
                    error_type: "cancelled".to_string(),
                });
            }
            _ = async {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(MultiTurnStreamItem::StreamAssistantItem(
                            StreamedAssistantContent::Text(text),
                        )) => {
                            accumulated.push_str(&text.text);
                            if channel_clone
                                .send(GenerateCharacterChunk::Token {
                                    content: text.text.clone(),
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Ok(MultiTurnStreamItem::FinalResponse(_)) => {
                            // Parse accumulated text as GeneratedCharacterData
                            let parsed = Self::parse_generated_character(&accumulated);
                            match parsed {
                                Ok(character) => {
                                    let _ = channel_clone.send(GenerateCharacterChunk::Complete {
                                        character,
                                    });
                                }
                                Err(parse_err) => {
                                    let _ = channel_clone.send(GenerateCharacterChunk::Error {
                                        error: format!(
                                            "Failed to parse generated character: {}",
                                            parse_err
                                        ),
                                        error_type: "parse_error".to_string(),
                                    });
                                }
                            }
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let err_str = e.to_string();
                            let _ = channel_clone.send(GenerateCharacterChunk::Error {
                                error: err_str.clone(),
                                error_type: classify_error(&err_str),
                            });
                            break;
                        }
                    }
                }
            } => {}
        }

        Ok(())
    }

    /// Parse the LLM's raw text output into GeneratedCharacterData.
    /// Tries to extract JSON from the response (the LLM may wrap it in
    /// markdown code fences or prefix it with explanation text).
    fn parse_generated_character(raw: &str) -> Result<GeneratedCharacterData, String> {
        // Try to find JSON in the response — may be wrapped in ```json ... ```
        let json_text = Self::extract_json(raw)?;

        // Parse into a generic Value first, then coerce each field.
        // LLMs may output objects/arrays where strings are expected, so we
        // flatten/convert liberally rather than failing on type mismatches.
        let value: serde_json::Value =
            serde_json::from_str(&json_text).map_err(|e| format!("JSON parse error: {}", e))?;

        let obj = value
            .as_object()
            .ok_or_else(|| "Generated JSON is not an object".to_string())?;

        let coerce_string = |key: &str| -> String {
            match obj.get(key) {
                None | Some(serde_json::Value::Null) => String::new(),
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::Bool(b)) => b.to_string(),
                Some(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|v| {
                        v.as_str()
                            .map(|s| s.to_string())
                            .or_else(|| Some(v.to_string()))
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                Some(serde_json::Value::Object(map)) => {
                    // Flatten nested objects into key=value pairs
                    map.iter()
                        .map(|(k, v)| {
                            format!(
                                "{}={}",
                                k,
                                match v {
                                    serde_json::Value::String(s) => s.clone(),
                                    other => other.to_string().trim_matches('"').to_string(),
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ")
                }
            }
        };

        let coerce_string_array = |key: &str| -> Vec<String> {
            match obj.get(key) {
                None | Some(serde_json::Value::Null) => Vec::new(),
                Some(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                Some(serde_json::Value::String(s)) => {
                    // Comma-separated string → array
                    s.split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect()
                }
                _ => Vec::new(),
            }
        };

        Ok(GeneratedCharacterData {
            name: coerce_string("name"),
            description: coerce_string("description"),
            personality: coerce_string("personality"),
            scenario: coerce_string("scenario"),
            first_mes: coerce_string("first_mes"),
            mes_example: coerce_string("mes_example"),
            system_prompt: coerce_string("system_prompt"),
            post_history_instructions: coerce_string("post_history_instructions"),
            creator_notes: coerce_string("creator_notes"),
            tags: coerce_string_array("tags"),
            alternate_greetings: coerce_string_array("alternate_greetings"),
        })
    }

    /// Extract JSON from potentially markdown-wrapped LLM output.
    /// Tries three strategies in order:
    /// 1. Code fence extraction (```json ... ```)
    /// 2. Brace matching (first { to last })
    /// 3. Raw parse (assume the entire text is JSON)
    fn extract_json(raw: &str) -> Result<String, String> {
        // Strategy 1: Extract from ```json ... ``` fences
        if let Some(start) = raw.find("```json") {
            let json_start = start + 7; // length of "```json"
                                        // Skip past the opening fence line (to newline or end)
            let content_start = raw[json_start..]
                .find('\n')
                .map(|i| json_start + i + 1)
                .unwrap_or(json_start);
            // Search for closing ``` after the opening fence content
            if let Some(end) = raw[content_start..].find("```") {
                let end_abs = content_start + end;
                let json = raw[content_start..end_abs].trim();
                if !json.is_empty() {
                    return Ok(json.to_string());
                }
            }
        }
        // Try generic code fences (``` ... ```)
        if let Some(start) = raw.find("```") {
            let after_fence = start + 3;
            // Skip optional language tag on first line
            let json_start = raw[after_fence..]
                .find('\n')
                .map(|i| after_fence + i + 1)
                .unwrap_or(after_fence);
            if let Some(end) = raw[json_start..].rfind("```") {
                let json = raw[json_start..json_start + end].trim();
                if !json.is_empty() {
                    return Ok(json.to_string());
                }
            }
        }

        // Strategy 2: Brace matching
        if let Some(first_brace) = raw.find('{') {
            if let Some(last_brace) = raw.rfind('}') {
                if last_brace > first_brace {
                    let json = &raw[first_brace..=last_brace];
                    if serde_json::from_str::<serde_json::Value>(json).is_ok() {
                        return Ok(json.to_string());
                    }
                }
            }
        }

        // Strategy 3: Raw parse
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }

        Err("No JSON content found in LLM response".to_string())
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
            stop_sequences: None,
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
            stop_sequences: None,
        });
        let params = AiService::from_provider_config(&provider);
        // max_tokens of 0 should map to None (unset)
        assert!(params.max_tokens.is_none());
    }

    // ── model_list_to_entries tests ──

    #[test]
    fn test_model_list_to_entries_basic() {
        use rig_core::model::{Model, ModelList};
        let list = ModelList::new(vec![
            Model::from_id("gpt-4o-mini"),
            Model::from_id("gpt-4o"),
        ]);
        let entries = AiService::model_list_to_entries(list);
        assert_eq!(entries.len(), 2);
        // Sorted alphabetically by name (id)
        assert_eq!(entries[0].name, "gpt-4o");
        assert_eq!(entries[1].name, "gpt-4o-mini");
        assert!(entries.iter().all(|e| e.enabled));
        assert!(entries.iter().all(|e| e.context_window_size.is_none()));
    }

    #[test]
    fn test_model_list_to_entries_with_context_length() {
        use rig_core::model::{Model, ModelList};
        let mut model = Model::from_id("claude-3");
        model.context_length = Some(200000);
        let list = ModelList::new(vec![model]);
        let entries = AiService::model_list_to_entries(list);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].context_window_size, Some(200000));
    }

    #[test]
    fn test_model_list_to_entries_dedup_and_case_insensitive_sort() {
        use rig_core::model::{Model, ModelList};
        let list = ModelList::new(vec![
            Model::from_id("zeta"),
            Model::from_id("Alpha"),
            Model::from_id("zeta"), // duplicate
        ]);
        let entries = AiService::model_list_to_entries(list);
        // Deduped to 2 entries
        assert_eq!(entries.len(), 2);
        // Case-insensitive sort: Alpha < zeta
        assert_eq!(entries[0].name, "Alpha");
        assert_eq!(entries[1].name, "zeta");
    }
}
