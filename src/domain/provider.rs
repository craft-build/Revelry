use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported AI provider types.
/// Each variant maps to a rig-core provider client.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Ollama,
    Gemini,
    Cohere,
    Mistral,
    Groq,
    DeepSeek,
    OpenRouter,
    Xai,
    Perplexity,
    Together,
    AzureOpenAI,
    HuggingFace,
    Custom,
}

impl ProviderType {
    /// Returns true for providers that require an API key (cloud providers and custom).
    /// Returns false for Ollama (local provider).
    pub fn requires_api_key(&self) -> bool {
        match self {
            ProviderType::Ollama => false,
            _ => true, // All cloud providers and Custom require an API key
        }
    }

    /// Returns the default base URL for this provider type.
    /// Returns None for types that have no sensible default (Custom).
    pub fn default_base_url(&self) -> Option<&'static str> {
        match self {
            ProviderType::OpenAI => Some("https://api.openai.com/v1"),
            ProviderType::Anthropic => Some("https://api.anthropic.com/v1"),
            ProviderType::Ollama => Some("http://localhost:11434"),
            ProviderType::Gemini => Some("https://generativelanguage.googleapis.com/v1beta"),
            ProviderType::Cohere => Some("https://api.cohere.ai/v1"),
            ProviderType::Mistral => Some("https://api.mistral.ai/v1"),
            ProviderType::Groq => Some("https://api.groq.com/openai/v1"),
            ProviderType::DeepSeek => Some("https://api.deepseek.com/v1"),
            ProviderType::OpenRouter => Some("https://openrouter.ai/api/v1"),
            ProviderType::Xai => Some("https://api.x.ai/v1"),
            ProviderType::Perplexity => Some("https://api.perplexity.ai"),
            ProviderType::Together => Some("https://api.together.xyz/v1"),
            ProviderType::AzureOpenAI => None, // Requires deployment-specific URL
            ProviderType::HuggingFace => Some("https://api-inference.huggingface.co"),
            ProviderType::Custom => None, // No default for custom endpoints
        }
    }
}

/// Configuration for an AI provider stored in the database.
/// API keys are NOT stored here — they go in the OS keychain via the keyring crate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl ProviderConfig {
    /// Creates a new provider configuration with a generated UUID v4 id.
    /// Sets the base_url from the provider type's default.
    /// Sets enabled to true and timestamps to the current time.
    pub fn new(name: String, provider_type: ProviderType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            base_url: provider_type.default_base_url().map(|s| s.to_string()),
            provider_type,
            enabled: true,
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_serde_roundtrip() {
        let provider = ProviderType::OpenAI;
        let json = serde_json::to_string(&provider).expect("serialize failed");
        let deserialized: ProviderType =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(provider, deserialized);
    }

    #[test]
    fn test_provider_type_serde_all_variants() {
        let variants = [
            ProviderType::OpenAI,
            ProviderType::Anthropic,
            ProviderType::Ollama,
            ProviderType::Gemini,
            ProviderType::Cohere,
            ProviderType::Mistral,
            ProviderType::Groq,
            ProviderType::DeepSeek,
            ProviderType::OpenRouter,
            ProviderType::Xai,
            ProviderType::Perplexity,
            ProviderType::Together,
            ProviderType::AzureOpenAI,
            ProviderType::HuggingFace,
            ProviderType::Custom,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize failed");
            let deserialized: ProviderType =
                serde_json::from_str(&json).expect("deserialize failed");
            assert_eq!(variant, &deserialized, "serde roundtrip failed for {:?}", variant);
        }
    }

    #[test]
    fn test_requires_api_key_cloud_providers() {
        // All cloud providers should require an API key
        assert!(ProviderType::OpenAI.requires_api_key(), "OpenAI should require API key");
        assert!(ProviderType::Anthropic.requires_api_key(), "Anthropic should require API key");
        assert!(ProviderType::Gemini.requires_api_key(), "Gemini should require API key");
        assert!(ProviderType::Cohere.requires_api_key(), "Cohere should require API key");
        assert!(ProviderType::Mistral.requires_api_key(), "Mistral should require API key");
        assert!(ProviderType::Groq.requires_api_key(), "Groq should require API key");
        assert!(ProviderType::DeepSeek.requires_api_key(), "DeepSeek should require API key");
        assert!(ProviderType::OpenRouter.requires_api_key(), "OpenRouter should require API key");
        assert!(ProviderType::Xai.requires_api_key(), "Xai should require API key");
        assert!(ProviderType::Perplexity.requires_api_key(), "Perplexity should require API key");
        assert!(ProviderType::Together.requires_api_key(), "Together should require API key");
        assert!(ProviderType::AzureOpenAI.requires_api_key(), "AzureOpenAI should require API key");
        assert!(ProviderType::HuggingFace.requires_api_key(), "HuggingFace should require API key");
        assert!(ProviderType::Custom.requires_api_key(), "Custom should require API key");
    }

    #[test]
    fn test_requires_api_key_ollama() {
        // Ollama (local) should NOT require an API key
        assert!(!ProviderType::Ollama.requires_api_key(), "Ollama should NOT require API key");
    }

    #[test]
    fn test_default_base_url_openai() {
        assert_eq!(
            ProviderType::OpenAI.default_base_url(),
            Some("https://api.openai.com/v1")
        );
    }

    #[test]
    fn test_default_base_url_anthropic() {
        assert_eq!(
            ProviderType::Anthropic.default_base_url(),
            Some("https://api.anthropic.com/v1")
        );
    }

    #[test]
    fn test_default_base_url_ollama() {
        assert_eq!(
            ProviderType::Ollama.default_base_url(),
            Some("http://localhost:11434")
        );
    }

    #[test]
    fn test_default_base_url_gemini() {
        assert_eq!(
            ProviderType::Gemini.default_base_url(),
            Some("https://generativelanguage.googleapis.com/v1beta")
        );
    }

    #[test]
    fn test_default_base_url_cohere() {
        assert_eq!(
            ProviderType::Cohere.default_base_url(),
            Some("https://api.cohere.ai/v1")
        );
    }

    #[test]
    fn test_default_base_url_mistral() {
        assert_eq!(
            ProviderType::Mistral.default_base_url(),
            Some("https://api.mistral.ai/v1")
        );
    }

    #[test]
    fn test_default_base_url_groq() {
        assert_eq!(
            ProviderType::Groq.default_base_url(),
            Some("https://api.groq.com/openai/v1")
        );
    }

    #[test]
    fn test_default_base_url_deepseek() {
        assert_eq!(
            ProviderType::DeepSeek.default_base_url(),
            Some("https://api.deepseek.com/v1")
        );
    }

    #[test]
    fn test_default_base_url_openrouter() {
        assert_eq!(
            ProviderType::OpenRouter.default_base_url(),
            Some("https://openrouter.ai/api/v1")
        );
    }

    #[test]
    fn test_default_base_url_xai() {
        assert_eq!(
            ProviderType::Xai.default_base_url(),
            Some("https://api.x.ai/v1")
        );
    }

    #[test]
    fn test_default_base_url_perplexity() {
        assert_eq!(
            ProviderType::Perplexity.default_base_url(),
            Some("https://api.perplexity.ai")
        );
    }

    #[test]
    fn test_default_base_url_together() {
        assert_eq!(
            ProviderType::Together.default_base_url(),
            Some("https://api.together.xyz/v1")
        );
    }

    #[test]
    fn test_default_base_url_custom() {
        // Custom providers have no default URL
        assert_eq!(ProviderType::Custom.default_base_url(), None);
    }

    #[test]
    fn test_provider_config_new_generates_uuid() {
        let config = ProviderConfig::new("My Provider".to_string(), ProviderType::OpenAI);
        // The id should be a valid UUID v4 (non-empty, parseable by uuid crate)
        assert!(!config.id.is_empty(), "ProviderConfig::new should generate a non-empty UUID");
        let parsed = Uuid::parse_str(&config.id);
        assert!(parsed.is_ok(), "ProviderConfig id should be a valid UUID, got: {}", config.id);
    }

    #[test]
    fn test_provider_config_new_sets_enabled() {
        let config = ProviderConfig::new("My Provider".to_string(), ProviderType::OpenAI);
        assert!(config.enabled, "New provider should be enabled by default");
    }

    #[test]
    fn test_provider_config_new_sets_base_url_from_type() {
        let config = ProviderConfig::new("My Provider".to_string(), ProviderType::OpenAI);
        assert_eq!(
            config.base_url,
            Some("https://api.openai.com/v1".to_string()),
            "ProviderConfig::new should set base_url from provider type default"
        );
    }

    #[test]
    fn test_provider_config_new_ollama_no_api_key_needed() {
        let config = ProviderConfig::new("Local Ollama".to_string(), ProviderType::Ollama);
        assert_eq!(
            config.base_url,
            Some("http://localhost:11434".to_string()),
            "Ollama config should default to localhost"
        );
    }

    #[test]
    fn test_provider_config_new_custom_no_default_url() {
        let config = ProviderConfig::new("My Custom".to_string(), ProviderType::Custom);
        assert_eq!(
            config.base_url,
            None,
            "Custom provider should have no default base_url"
        );
    }

    #[test]
    fn test_provider_config_new_sets_timestamps() {
        let before = chrono::Local::now().naive_local();
        let config = ProviderConfig::new("My Provider".to_string(), ProviderType::OpenAI);
        let after = chrono::Local::now().naive_local();
        assert!(config.created_at >= before && config.created_at <= after,
            "created_at should be approximately now");
        assert!(config.updated_at >= before && config.updated_at <= after,
            "updated_at should be approximately now");
    }

    #[test]
    fn test_provider_config_serde_roundtrip() {
        let config = ProviderConfig::new("Test Provider".to_string(), ProviderType::Anthropic);
        let json = serde_json::to_string(&config).expect("serialize failed");
        let deserialized: ProviderConfig =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(config.id, deserialized.id);
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.provider_type, deserialized.provider_type);
        assert_eq!(config.enabled, deserialized.enabled);
    }
}
