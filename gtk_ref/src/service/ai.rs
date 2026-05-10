use crate::domain::conversation::{Message, MessageRole};
use crate::domain::generation::GenerationPreset;
use crate::domain::provider::{ProviderConfig, ProviderType};

use futures::StreamExt;
use rig::client::CompletionClient;
use rig::streaming::StreamingChat;

/// Generation parameters resolved from the preset hierarchy (D-03).
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

impl GenerationParams {
    /// Create from a resolved GenerationPreset (D-01, D-13).
    /// Returns default (all None) if no preset is assigned.
    pub fn from_preset(preset: Option<&GenerationPreset>) -> Self {
        preset
            .map(|p| Self {
                temperature: Some(p.temperature),
                top_p: Some(p.top_p),
                max_tokens: if p.max_tokens > 0 {
                    Some(p.max_tokens as u64)
                } else {
                    None
                },
                frequency_penalty: if p.frequency_penalty != 0.0 {
                    Some(p.frequency_penalty)
                } else {
                    None
                },
                presence_penalty: if p.presence_penalty != 0.0 {
                    Some(p.presence_penalty)
                } else {
                    None
                },
                stop_sequences: if p.stop_sequences.is_empty() {
                    None
                } else {
                    Some(p.stop_sequences.clone())
                },
            })
            .unwrap_or_default()
    }
}

/// Returns true if the given provider supports the named additional parameter.
/// Per D-02: silently skip unsupported params per-provider.
fn provider_supports_param(provider_type: &ProviderType, param_name: &str) -> bool {
    match param_name {
        // frequency_penalty and presence_penalty: not supported by Anthropic, Gemini, Cohere
        "frequency_penalty" | "presence_penalty" => {
            !matches!(
                provider_type,
                ProviderType::Anthropic | ProviderType::Gemini | ProviderType::Cohere
            )
        }
        // top_p and stop_sequences are supported by all providers we use
        "top_p" | "stop" => true,
        _ => false,
    }
}

/// A chunk of data from a streaming AI response.
#[derive(Clone, Debug)]
pub enum StreamChunk {
    /// A token/text fragment from the streaming response
    Token(String),
    /// The complete final response
    Complete(String),
    /// An error occurred during streaming
    Error(String),
}

/// Service for AI operations.
/// Provides utilities for building prompts and handling streaming responses.
pub struct AiService;

impl AiService {
    /// Returns a boxed, pinned stream for handling streaming AI responses.
    /// The stream yields [`StreamChunk`] items and is safe to send across threads.
    pub fn stream_chat(
        provider: &ProviderConfig,
        system_prompt: &str,
        messages: &[Message],
        api_key: &str,
        params: GenerationParams,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = StreamChunk> + Send>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<StreamChunk>(32);

        let provider_type = provider.provider_type.clone();
        let base_url = provider.base_url.clone();
        let model_name = provider.model_name.clone();
        let system = system_prompt.to_string();
        let key = api_key.to_string();

        // Convert domain messages to rig messages
        let rig_messages: Vec<rig::message::Message> = messages
            .iter()
            .filter(|m| !m.content.is_empty())
            .map(|msg| match msg.role {
                MessageRole::User => rig::message::Message::user(&msg.content),
                MessageRole::Assistant => rig::message::Message::assistant(&msg.content),
                MessageRole::System => rig::message::Message::system(&msg.content),
            })
            .collect();

        // Last message is the active prompt, rest is chat history
        let (prompt, history) = Self::split_last_prompt(rig_messages);

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
                &tx,
            )
            .await
            {
                let _ = tx.send(StreamChunk::Error(e)).await;
            }
        });

        Box::pin(futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|chunk| (chunk, rx))
        }))
    }

    /// Splits the last message off as the prompt, returns the remainder as history.
    fn split_last_prompt(
        messages: Vec<rig::message::Message>,
    ) -> (rig::message::Message, Vec<rig::message::Message>) {
        let mut messages = messages;
        if messages.is_empty() {
            return (rig::message::Message::user(""), Vec::new());
        }
        let prompt = messages.pop().unwrap_or_else(|| rig::message::Message::user(""));
        (prompt, messages)
    }

    /// Dispatches to the correct rig provider client and runs the stream.
    async fn run_stream(
        provider_type: &ProviderType,
        base_url: Option<&str>,
        model_name: &str,
        system_prompt: &str,
        prompt: rig::message::Message,
        history: Vec<rig::message::Message>,
        api_key: &str,
        params: &GenerationParams,
        tx: &tokio::sync::mpsc::Sender<StreamChunk>,
    ) -> Result<(), String> {
        // Build provider-specific additional_params JSON (D-02)
        let additional = Self::build_additional_params(provider_type, params);

        match provider_type {
            ProviderType::OpenAI => {
                let client = Self::build_openai_client(api_key, base_url)?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Anthropic => {
                let client = rig::providers::anthropic::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Ollama => {
                let url = base_url.unwrap_or("http://localhost:11434");
                let client = rig::providers::ollama::Client::builder()
                    .api_key(String::new())
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Gemini => {
                let client = rig::providers::gemini::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Cohere => {
                let client = rig::providers::cohere::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Mistral => {
                let client = rig::providers::mistral::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Groq => {
                let client = rig::providers::groq::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::DeepSeek => {
                let client = rig::providers::deepseek::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::OpenRouter => {
                let client = rig::providers::openrouter::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Xai => {
                let client = rig::providers::xai::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Perplexity => {
                let client = rig::providers::perplexity::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Together => {
                let client = rig::providers::together::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::AzureOpenAI => {
                let url = base_url.ok_or("Base URL required for Azure OpenAI")?;
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::HuggingFace => {
                let client = rig::providers::huggingface::Client::new(api_key)
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
            }
            ProviderType::Custom => {
                let url = base_url.ok_or("Base URL required for custom provider")?;
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                let mut builder = client.agent(model_name).preamble(system_prompt);
                if let Some(temp) = params.temperature {
                    builder = builder.temperature(temp);
                }
                if let Some(max) = params.max_tokens {
                    builder = builder.max_tokens(max);
                }
                if !additional.is_empty() {
                    builder = builder.additional_params(serde_json::Value::Object(additional));
                }
                let agent = builder.build();
                Self::stream_with_agent(&agent, prompt, history, tx).await;
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
    ) -> Result<rig::providers::openai::Client, String> {
        if let Some(url) = base_url {
            rig::providers::openai::Client::builder()
                .api_key(api_key)
                .base_url(url)
                .build()
                .map_err(|e| e.to_string())
        } else {
            rig::providers::openai::Client::new(api_key).map_err(|e| e.to_string())
        }
    }

    /// Generic streaming helper that works with any rig completion model.
    async fn stream_with_agent<M>(
        agent: &rig::agent::Agent<M, ()>,
        prompt: rig::message::Message,
        history: Vec<rig::message::Message>,
        tx: &tokio::sync::mpsc::Sender<StreamChunk>,
    ) where
        M: rig::completion::CompletionModel + 'static,
        M::StreamingResponse: rig::completion::GetTokenUsage,
    {
        let mut stream = agent
            .stream_chat(prompt, history)
            .await;

        while let Some(item) = stream.next().await {
            match item {
                Ok(
                    rig::agent::MultiTurnStreamItem::StreamAssistantItem(
                        rig::streaming::StreamedAssistantContent::Text(t),
                    ),
                ) => {
                    if tx.send(StreamChunk::Token(t.text)).await.is_err() {
                        break;
                    }
                }
                Ok(
                    rig::agent::MultiTurnStreamItem::FinalResponse(
                        res,
                    ),
                ) => {
                    let _ = tx.send(StreamChunk::Complete(res.response().to_string())).await;
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    let _ = tx.send(StreamChunk::Error(e.to_string())).await;
                    break;
                }
            }
        }
    }

    /// Returns the provider type for a given provider config.
    pub fn provider_type(provider: &ProviderConfig) -> ProviderType {
        provider.provider_type.clone()
    }

    /// Returns true if the provider requires an API key.
    pub fn requires_api_key(provider: &ProviderConfig) -> bool {
        provider.provider_type.requires_api_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::provider::{ProviderConfig, ProviderType};

    #[test]
    fn test_provider_type_helper() {
        let config = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        assert_eq!(AiService::provider_type(&config), ProviderType::OpenAI);
    }

    #[test]
    fn test_requires_api_key_openai() {
        let config = ProviderConfig::new("OpenAI".to_string(), ProviderType::OpenAI);
        assert!(AiService::requires_api_key(&config));
    }

    #[test]
    fn test_requires_api_key_ollama() {
        let config = ProviderConfig::new("Ollama".to_string(), ProviderType::Ollama);
        assert!(!AiService::requires_api_key(&config));
    }

    #[test]
    fn test_stream_chunk_variants() {
        let token = StreamChunk::Token("Hello".to_string());
        let complete = StreamChunk::Complete("Full response".to_string());
        let error = StreamChunk::Error("Something went wrong".to_string());

        match token {
            StreamChunk::Token(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected Token variant"),
        }

        match complete {
            StreamChunk::Complete(s) => assert_eq!(s, "Full response"),
            _ => panic!("Expected Complete variant"),
        }

        match error {
            StreamChunk::Error(s) => assert_eq!(s, "Something went wrong"),
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_split_last_prompt_empty() {
        let (prompt, history) = AiService::split_last_prompt(vec![]);
        assert!(matches!(prompt, rig::message::Message::User { .. }));
        assert!(history.is_empty());
    }

    #[test]
    fn test_split_last_prompt_single() {
        let msgs = vec![rig::message::Message::user("hello")];
        let (prompt, history) = AiService::split_last_prompt(msgs);
        assert!(matches!(prompt, rig::message::Message::User { .. }));
        assert!(history.is_empty());
    }

    #[test]
    fn test_split_last_prompt_multiple() {
        let msgs = vec![
            rig::message::Message::user("first"),
            rig::message::Message::assistant("second"),
            rig::message::Message::user("third"),
        ];
        let (prompt, history) = AiService::split_last_prompt(msgs);
        assert!(matches!(prompt, rig::message::Message::User { .. }));
        assert_eq!(history.len(), 2);
    }

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

    #[test]
    fn test_generation_params_from_preset_none() {
        let params = GenerationParams::from_preset(None);
        assert!(params.temperature.is_none());
        assert!(params.max_tokens.is_none());
    }

    #[test]
    fn test_generation_params_from_preset() {
        let preset = GenerationPreset::creative();
        let params = GenerationParams::from_preset(Some(&preset));
        assert_eq!(params.temperature, Some(preset.temperature));
        assert_eq!(params.top_p, Some(preset.top_p));
        // max_tokens is 0 for built-in Creative → should be None
        assert!(params.max_tokens.is_none());
        // frequency_penalty and presence_penalty are 0.0 → should be None
        assert!(params.frequency_penalty.is_none());
        assert!(params.presence_penalty.is_none());
    }

    #[test]
    fn test_generation_params_from_preset_with_values() {
        let mut preset = GenerationPreset::creative();
        preset.max_tokens = 2048;
        preset.frequency_penalty = 0.5;
        preset.presence_penalty = 0.3;
        preset.stop_sequences = vec!["\n".to_string(), "END".to_string()];
        let params = GenerationParams::from_preset(Some(&preset));
        assert_eq!(params.max_tokens, Some(2048));
        assert_eq!(params.frequency_penalty, Some(0.5));
        assert_eq!(params.presence_penalty, Some(0.3));
        assert_eq!(params.stop_sequences, Some(vec!["\n".to_string(), "END".to_string()]));
    }

    #[test]
    fn test_provider_supports_param_openai() {
        assert!(provider_supports_param(&ProviderType::OpenAI, "frequency_penalty"));
        assert!(provider_supports_param(&ProviderType::OpenAI, "presence_penalty"));
        assert!(provider_supports_param(&ProviderType::OpenAI, "top_p"));
        assert!(provider_supports_param(&ProviderType::OpenAI, "stop"));
    }

    #[test]
    fn test_provider_supports_param_anthropic() {
        assert!(!provider_supports_param(&ProviderType::Anthropic, "frequency_penalty"));
        assert!(!provider_supports_param(&ProviderType::Anthropic, "presence_penalty"));
        assert!(provider_supports_param(&ProviderType::Anthropic, "top_p"));
        assert!(provider_supports_param(&ProviderType::Anthropic, "stop"));
    }

    #[test]
    fn test_provider_supports_param_gemini() {
        assert!(!provider_supports_param(&ProviderType::Gemini, "frequency_penalty"));
        assert!(!provider_supports_param(&ProviderType::Gemini, "presence_penalty"));
        assert!(provider_supports_param(&ProviderType::Gemini, "top_p"));
    }

    #[test]
    fn test_provider_supports_param_cohere() {
        assert!(!provider_supports_param(&ProviderType::Cohere, "frequency_penalty"));
        assert!(!provider_supports_param(&ProviderType::Cohere, "presence_penalty"));
    }

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
        let anthropic_additional = AiService::build_additional_params(&ProviderType::Anthropic, &params);
        assert!(anthropic_additional.contains_key("top_p"));
        assert!(!anthropic_additional.contains_key("frequency_penalty"));
        assert!(!anthropic_additional.contains_key("presence_penalty"));
        assert!(anthropic_additional.contains_key("stop"));

        // temperature and max_tokens should NOT be in additional params (native AgentBuilder)
        assert!(!openai_additional.contains_key("temperature"));
        assert!(!openai_additional.contains_key("max_tokens"));
    }
}
