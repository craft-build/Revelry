use rig::agent::Agent;
use rig::client::{AgentClientExt, CompletionClient};

use super::GenerationParams;

/// Builds an OpenAI client, optionally with a custom base URL.
pub(crate) fn build_openai_client(
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
        rig::providers::openai::Client::builder()
            .api_key(api_key)
            .build()
            .map_err(|e| e.to_string())
    }
}

/// Shared rig agent builder: preamble + temperature/max_tokens/additional_params.
pub(crate) fn build_agent<C>(
    client: &C,
    model_name: &str,
    system_prompt: &str,
    params: &GenerationParams,
    additional: &serde_json::Map<String, serde_json::Value>,
) -> Agent
where
    C: AgentClientExt,
    <C as CompletionClient>::CompletionModel: 'static,
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
    builder.build()
}

/// Dispatches to the correct rig native provider client and runs `$body`
/// with the constructed client bound to `$client`. Defined once, shared by
/// chat streaming and character generation. Only one match arm runs, so the
/// body may move captured values directly (no clones needed).
macro_rules! with_native_client {
    ($provider:expr, $base_url:expr, $api_key:expr, $client:ident, $body:block) => {
        match $provider {
            ProviderType::OpenAI => {
                let $client = $crate::ai::providers::build_openai_client($api_key, $base_url)?;
                $body
            }
            ProviderType::Anthropic => {
                let $client = rig::providers::anthropic::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Ollama => {
                let url = $base_url.unwrap_or("http://localhost:11434");
                let $client = rig::providers::ollama::Client::builder()
                    .api_key(String::new())
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Gemini => {
                let $client = rig::providers::gemini::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Cohere => {
                let $client = rig::providers::cohere::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Mistral => {
                let $client = rig::providers::mistral::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Groq => {
                let $client = rig::providers::groq::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::DeepSeek => {
                let $client = rig::providers::deepseek::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::OpenRouter => {
                let $client = rig::providers::openrouter::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Xai => {
                let $client = rig::providers::xai::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Perplexity => {
                let $client = rig::providers::perplexity::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::Together => {
                let $client = rig::providers::together::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::AzureOpenAI => {
                let url = $base_url.ok_or("Base URL required for Azure OpenAI")?;
                let $client = rig::providers::openai::Client::builder()
                    .api_key($api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?
                    .completions_api();
                $body
            }
            ProviderType::HuggingFace => {
                let $client = rig::providers::huggingface::Client::builder()
                    .api_key($api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                $body
            }
            ProviderType::CustomOpenAICompatible => {
                let url = $base_url.ok_or("Base URL required for custom provider")?;
                let $client = rig::providers::openai::Client::builder()
                    .api_key($api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?
                    .completions_api();
                $body
            }
        }
    };
}

pub(crate) use with_native_client;
