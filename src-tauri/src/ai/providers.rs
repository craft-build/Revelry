use rig::agent::{Agent, AgentBuilder};

use super::GenerationParams;

/// Builds an OpenAI client, optionally with a custom base URL.
pub(crate) fn build_openai_client(
    api_key: &str,
    base_url: Option<&str>,
) -> rig::providers::openai::OpenAI {
    let config = rig::providers::openai::OpenAIConfig::new(api_key);
    let config = match base_url {
        Some(url) => config.with_base_url(url),
        None => config,
    };
    config.client()
}

/// Shared rig agent builder: preamble + temperature/max_tokens/additional_params.
pub(crate) fn build_agent(
    model: impl Into<rig::DynModel<rig::operation::Completion>>,
    system_prompt: &str,
    params: &GenerationParams,
    additional: &serde_json::Map<String, serde_json::Value>,
) -> Agent {
    let mut builder = AgentBuilder::new(model).preamble(system_prompt);
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
/// with the completion model bound to `$model`. Defined once, shared by
/// chat streaming and character generation. Only one match arm runs, so the
/// body may move captured values directly (no clones needed).
macro_rules! with_native_client {
    ($provider:expr, $base_url:expr, $api_key:expr, $model_name:expr, $model:ident, $body:block) => {
        match $provider {
            ProviderType::OpenAI => {
                let client = $crate::ai::providers::build_openai_client($api_key, $base_url);
                // Chat Completions, not the Responses API: rig 0.44's
                // `completion()` routes OpenAI to /v1/responses, which
                // rejects the penalty/stop params presets set and excludes
                // non-Responses models and chat-only proxies.
                let $model = client.chat($model_name);
                $body
            }
            ProviderType::Anthropic => {
                let client = rig::providers::anthropic::Anthropic::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Ollama => {
                let url = $base_url.unwrap_or("http://localhost:11434");
                let client = rig::providers::ollama::OllamaConfig::new()
                    .with_base_url(url)
                    .client();
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Gemini => {
                let client = rig::providers::gemini::Gemini::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Cohere => {
                let client = rig::providers::cohere::Cohere::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Mistral => {
                let client = rig::providers::mistral::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Groq => {
                let client = rig::providers::groq::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::DeepSeek => {
                let client = rig::providers::deepseek::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::OpenRouter => {
                let client = rig::providers::openrouter::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Xai => {
                let client = rig::providers::xai::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Perplexity => {
                let client = rig::providers::perplexity::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::Together => {
                let client = rig::providers::together::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::AzureOpenAI => {
                let url = $base_url.ok_or("Base URL required for Azure OpenAI")?;
                let client = $crate::ai::providers::build_openai_client($api_key, Some(url));
                let $model = client.chat($model_name);
                $body
            }
            ProviderType::HuggingFace => {
                let client = rig::providers::huggingface::new($api_key);
                let $model = client.completion($model_name);
                $body
            }
            ProviderType::CustomOpenAICompatible => {
                let url = $base_url.ok_or("Base URL required for custom provider")?;
                let client = $crate::ai::providers::build_openai_client($api_key, Some(url));
                let $model = client.chat($model_name);
                $body
            }
        }
    };
}

pub(crate) use with_native_client;
