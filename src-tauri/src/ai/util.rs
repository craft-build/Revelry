use crate::domain::provider::ProviderType;

use super::{AiService, GenerationParams};

/// Returns true if the given provider supports the named additional parameter.
/// Per D-02: silently skip unsupported params per-provider.
pub(crate) fn provider_supports_param(provider_type: &ProviderType, param_name: &str) -> bool {
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
pub(crate) fn classify_error(err: &str) -> String {
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

impl AiService {
    /// Build provider-specific additional_params JSON for non-native params (D-02).
    /// Only includes parameters the provider actually supports.
    pub(crate) fn build_additional_params(
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
