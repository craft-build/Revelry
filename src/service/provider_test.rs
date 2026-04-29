use crate::domain::provider::ProviderType;
use rig::client::CompletionClient;

/// Result of a provider connectivity test.
#[derive(Debug, Clone)]
pub struct ProviderTestResult {
    /// Whether the connection test succeeded.
    pub success: bool,
    /// Human-readable message describing the result.
    pub message: String,
    /// Number of models available (if the provider supports listing models).
    pub model_count: Option<usize>,
}

/// Tests connectivity to an AI provider by constructing a rig client
/// from the given configuration and attempting a lightweight API call.
///
/// For cloud providers with API keys: attempts a minimal prompt.
/// For local providers (Ollama): attempts to connect to the local server.
///
/// Has a 10-second timeout to prevent hanging on unreachable endpoints.
pub async fn test_provider(
    provider_type: &ProviderType,
    base_url: Option<&str>,
    api_key: Option<&str>,
) -> ProviderTestResult {
    match provider_type {
        ProviderType::OpenAI => test_openai(api_key).await,
        ProviderType::Anthropic => test_anthropic(api_key).await,
        ProviderType::Ollama => test_ollama(base_url).await,
        ProviderType::Gemini => test_gemini(api_key).await,
        ProviderType::Cohere => test_cohere(api_key).await,
        ProviderType::Mistral => test_mistral(api_key).await,
        ProviderType::Groq => test_groq(api_key).await,
        ProviderType::DeepSeek => test_deepseek(api_key).await,
        ProviderType::OpenRouter => test_openrouter(api_key).await,
        ProviderType::Xai => test_xai(api_key).await,
        ProviderType::Perplexity => test_perplexity(api_key).await,
        ProviderType::Together => test_together(api_key).await,
        ProviderType::AzureOpenAI => test_azure_openai(api_key, base_url).await,
        ProviderType::HuggingFace => test_huggingface(api_key).await,
        ProviderType::Custom => test_custom(api_key, base_url).await,
    }
}

/// Macro to generate provider test functions that follow the same pattern:
/// 1. Check required parameters
/// 2. Build client
/// 3. Send minimal prompt with timeout
macro_rules! define_provider_test {
    ($name:ident, $require_msg:expr, $client_expr:expr, $model:expr) => {
        async fn $name(api_key: Option<&str>) -> ProviderTestResult {
            let Some(key) = api_key else {
                return ProviderTestResult {
                    success: false,
                    message: $require_msg.to_string(),
                    model_count: None,
                };
            };

            let client = match $client_expr(key) {
                Ok(c) => c,
                Err(e) => {
                    return ProviderTestResult {
                        success: false,
                        message: format!("Client creation failed: {}", redact_error(&e.to_string())),
                        model_count: None,
                    };
                }
            };

            match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                async {
                    use rig::completion::Prompt;
                    let agent = client.agent($model).build();
                    agent.prompt("test").await
                },
            ).await {
                Ok(Ok(_)) => ProviderTestResult {
                    success: true,
                    message: "Connected successfully".to_string(),
                    model_count: None,
                },
                Ok(Err(e)) => ProviderTestResult {
                    success: false,
                    message: format!("Connection failed: {}", redact_error(&e.to_string())),
                    model_count: None,
                },
                Err(_) => ProviderTestResult {
                    success: false,
                    message: "Connection timed out after 10 seconds".to_string(),
                    model_count: None,
                },
            }
        }
    };
}

define_provider_test!(test_openai, "API key required for OpenAI",
    |key: &str| rig::providers::openai::Client::new(key), rig::providers::openai::GPT_4O_MINI);

define_provider_test!(test_anthropic, "API key required for Anthropic",
    |key: &str| rig::providers::anthropic::Client::new(key), rig::providers::anthropic::completion::CLAUDE_SONNET_4_6);

define_provider_test!(test_gemini, "API key required for Gemini",
    |key: &str| rig::providers::gemini::Client::new(key), rig::providers::gemini::completion::GEMINI_2_0_FLASH);

define_provider_test!(test_cohere, "API key required for Cohere",
    |key: &str| rig::providers::cohere::Client::new(key), "command-r-plus");

define_provider_test!(test_mistral, "API key required for Mistral",
    |key: &str| rig::providers::mistral::Client::new(key), "mistral-large-latest");

define_provider_test!(test_groq, "API key required for Groq",
    |key: &str| rig::providers::groq::Client::new(key), "llama-3.3-70b-versatile");

define_provider_test!(test_deepseek, "API key required for DeepSeek",
    |key: &str| rig::providers::deepseek::Client::new(key), "deepseek-chat");

define_provider_test!(test_openrouter, "API key required for OpenRouter",
    |key: &str| rig::providers::openrouter::Client::new(key), "openai/gpt-4o-mini");

define_provider_test!(test_xai, "API key required for xAI",
    |key: &str| rig::providers::xai::Client::new(key), "grok-3-mini");

define_provider_test!(test_perplexity, "API key required for Perplexity",
    |key: &str| rig::providers::perplexity::Client::new(key), "sonar-pro");

define_provider_test!(test_together, "API key required for Together AI",
    |key: &str| rig::providers::together::Client::new(key), "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo");

define_provider_test!(test_huggingface, "API key required for HuggingFace",
    |key: &str| rig::providers::huggingface::Client::new(key), "meta-llama/Llama-3.1-8B-Instruct");

/// Test Ollama connectivity (local, no API key required).
async fn test_ollama(base_url: Option<&str>) -> ProviderTestResult {
    let url = base_url.unwrap_or("http://localhost:11434");

    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        async {
            use rig::completion::Prompt;
            let client = rig::providers::ollama::Client::builder()
                .api_key(String::new())
                .base_url(url)
                .build()
                .map_err(|e| rig::completion::CompletionError::ProviderError(e.to_string()))?;
            let agent = client.agent("llama3.2").build();
            agent.prompt("test").await
        },
    ).await {
        Ok(Ok(_)) => ProviderTestResult {
            success: true,
            message: "Connected successfully".to_string(),
            model_count: None,
        },
        Ok(Err(e)) => {
            let err_str = e.to_string();
            if err_str.contains("not found") || err_str.contains("404") {
                ProviderTestResult {
                    success: true,
                    message: "Connected to Ollama server".to_string(),
                    model_count: None,
                }
            } else {
                ProviderTestResult {
                    success: false,
                    message: format!("Connection failed: {}", redact_error(&err_str)),
                    model_count: None,
                }
            }
        }
        Err(_) => ProviderTestResult {
            success: false,
            message: "Connection timed out after 10 seconds. Is Ollama running?".to_string(),
            model_count: None,
        },
    }
}

/// Test Azure OpenAI connectivity (requires both API key and deployment URL).
async fn test_azure_openai(api_key: Option<&str>, base_url: Option<&str>) -> ProviderTestResult {
    let (Some(key), Some(url)) = (api_key, base_url) else {
        return ProviderTestResult {
            success: false,
            message: "API key and base URL required for Azure OpenAI".to_string(),
            model_count: None,
        };
    };

    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        async {
            use rig::completion::Prompt;
            let client = rig::providers::openai::Client::builder()
                .api_key(key)
                .base_url(url)
                .build()
                .map_err(|e| rig::completion::CompletionError::ProviderError(e.to_string()))?;
            let agent = client.agent("gpt-4o-mini").build();
            agent.prompt("test").await
        },
    ).await {
        Ok(Ok(_)) => ProviderTestResult {
            success: true,
            message: "Connected successfully".to_string(),
            model_count: None,
        },
        Ok(Err(e)) => ProviderTestResult {
            success: false,
            message: format!("Connection failed: {}", redact_error(&e.to_string())),
            model_count: None,
        },
        Err(_) => ProviderTestResult {
            success: false,
            message: "Connection timed out after 10 seconds".to_string(),
            model_count: None,
        },
    }
}

/// Test custom OpenAI-compatible endpoint connectivity.
async fn test_custom(api_key: Option<&str>, base_url: Option<&str>) -> ProviderTestResult {
    let Some(url) = base_url else {
        return ProviderTestResult {
            success: false,
            message: "Base URL required for custom provider".to_string(),
            model_count: None,
        };
    };

    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        async {
            use rig::completion::Prompt;
            let key = api_key.unwrap_or("");
            let client = rig::providers::openai::Client::builder()
                .api_key(key)
                .base_url(url)
                .build()
                .map_err(|e| rig::completion::CompletionError::ProviderError(e.to_string()))?;
            let agent = client.agent("default").build();
            agent.prompt("test").await
        },
    ).await {
        Ok(Ok(_)) => ProviderTestResult {
            success: true,
            message: "Connected successfully".to_string(),
            model_count: None,
        },
        Ok(Err(e)) => ProviderTestResult {
            success: false,
            message: format!("Connection failed: {}", redact_error(&e.to_string())),
            model_count: None,
        },
        Err(_) => ProviderTestResult {
            success: false,
            message: "Connection timed out after 10 seconds".to_string(),
            model_count: None,
        },
    }
}

/// Redacts potential API keys from error messages to prevent
/// them from appearing in logs or UI (T-01-09).
fn redact_error(error: &str) -> String {
    let mut redacted = error.to_string();

    // Redact Bearer tokens
    if let Some(start) = redacted.find("Bearer ") {
        let end = start + 7 + 8;
        if end < redacted.len() {
            redacted = format!("{}...[REDACTED]{}", &redacted[..end], &redacted[end.min(redacted.len())..]);
        }
    }

    // Redact sk- prefixed keys
    if let Some(pos) = redacted.find("sk-") {
        let prefix_end = pos + 6;
        if prefix_end < redacted.len() {
            redacted = format!("{}...[REDACTED]{}", &redacted[..prefix_end], &redacted[prefix_end.min(redacted.len())..]);
        }
    }

    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_test_result_success() {
        let result = ProviderTestResult {
            success: true,
            message: "Connected successfully".to_string(),
            model_count: Some(42),
        };
        assert!(result.success);
        assert_eq!(result.model_count, Some(42));
    }

    #[test]
    fn test_provider_test_result_failure() {
        let result = ProviderTestResult {
            success: false,
            message: "Connection failed: invalid API key".to_string(),
            model_count: None,
        };
        assert!(!result.success);
        assert!(result.model_count.is_none());
    }

    #[test]
    fn test_redact_error_bearer() {
        let error = "Unauthorized: Bearer sk-abc123def456ghi789";
        let redacted = redact_error(error);
        assert!(!redacted.contains("sk-abc123def456ghi789"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_error_sk_prefix() {
        let error = "Invalid key: sk-ant-1234567890abcdef";
        let redacted = redact_error(error);
        assert!(!redacted.contains("sk-ant-1234567890abcdef"));
    }

    #[test]
    fn test_redact_error_no_key() {
        let error = "Connection timed out after 10 seconds";
        let redacted = redact_error(error);
        assert_eq!(error, redacted);
    }

    #[test]
    fn test_test_provider_missing_api_key() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let result = rt.block_on(test_provider(&ProviderType::OpenAI, None, None));
        assert!(!result.success);
        assert!(result.message.contains("API key required"));
    }

    #[test]
    fn test_test_provider_ollama_no_key_needed() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let result = rt.block_on(test_provider(&ProviderType::Ollama, None, None));
        assert!(!result.message.contains("API key required"));
    }

    #[test]
    fn test_test_provider_custom_no_url() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let result = rt.block_on(test_provider(&ProviderType::Custom, None, None));
        assert!(!result.success);
        assert!(result.message.contains("Base URL required"));
    }

    #[test]
    fn test_test_provider_azure_needs_both() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let result = rt.block_on(test_provider(&ProviderType::AzureOpenAI, None, None));
        assert!(!result.success);
        assert!(result.message.contains("API key and base URL required"));
    }
}
