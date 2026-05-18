use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported AI provider types.
/// Each variant maps to a rig-core provider client.
/// Note: We use explicit rename attributes instead of rename_all = "snake_case"
/// because serde's snake_case conversion splits "OpenAI" into "open_a_i"
/// instead of the expected "openai".
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "cohere")]
    Cohere,
    #[serde(rename = "mistral")]
    Mistral,
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "xai")]
    Xai,
    #[serde(rename = "perplexity")]
    Perplexity,
    #[serde(rename = "together")]
    Together,
    #[serde(rename = "azure_openai")]
    AzureOpenAI,
    #[serde(rename = "huggingface")]
    HuggingFace,
    #[serde(rename = "custom_openai_compatible")]
    CustomOpenAICompatible,
}

impl ProviderType {
    /// Returns true for providers that require an API key (cloud providers and custom).
    /// Returns false for Ollama (local provider).
    pub fn requires_api_key(&self) -> bool {
        match self {
            ProviderType::Ollama => false,
            _ => true,
        }
    }

    /// Returns the default base URL for this provider type.
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
            ProviderType::AzureOpenAI => None,
            ProviderType::HuggingFace => Some("https://api-inference.huggingface.co"),
            ProviderType::CustomOpenAICompatible => None,
        }
    }

    /// Returns a sensible default model name for this provider type.
    pub fn default_model_name(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "gpt-4o-mini",
            ProviderType::Anthropic => "claude-sonnet-4-20250514",
            ProviderType::Ollama => "llama3.2",
            ProviderType::Gemini => "gemini-2.0-flash",
            ProviderType::Cohere => "command-r-plus",
            ProviderType::Mistral => "mistral-large-latest",
            ProviderType::Groq => "llama-3.3-70b-versatile",
            ProviderType::DeepSeek => "deepseek-chat",
            ProviderType::OpenRouter => "openai/gpt-4o-mini",
            ProviderType::Xai => "grok-3-mini",
            ProviderType::Perplexity => "sonar-pro",
            ProviderType::Together => "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
            ProviderType::AzureOpenAI => "gpt-4o-mini",
            ProviderType::HuggingFace => "meta-llama/Llama-3.1-8B-Instruct",
            ProviderType::CustomOpenAICompatible => "default",
        }
    }
}

/// A single model entry within a provider's models list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderModelEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window_size: Option<i64>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Default generation parameters for a provider.
/// These are shared across all models under the provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}

/// Configuration for an AI provider stored in the database.
/// API keys are NOT stored here — they go in the OS keychain via KeyringStore.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "provider_type")]
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    /// The primary/default model name (kept for backward compat and quick reference).
    pub model_name: String,
    /// JSON array of model entries stored in the `models` TEXT column.
    #[serde(default = "Vec::new")]
    pub models: Vec<ProviderModelEntry>,
    /// User override for context window size.
    #[serde(default)]
    pub context_window_size: Option<i64>,
    pub enabled: bool,
    /// Default generation parameters stored as JSON.
    #[serde(default)]
    pub default_params: Option<GenerationParams>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProviderConfig {
    /// Creates a new provider configuration with a generated UUID v4 id.
    /// Sets the base_url from the provider type's default.
    /// Auto-populates models vec with one default ProviderModelEntry.
    /// Sets enabled to true and timestamps to the current time.
    pub fn new(name: String, provider_type: ProviderType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            base_url: provider_type.default_base_url().map(|s| s.to_string()),
            model_name: provider_type.default_model_name().to_string(),
            models: vec![ProviderModelEntry {
                name: provider_type.default_model_name().to_string(),
                context_window_size: None,
                enabled: true,
            }],
            context_window_size: None,
            provider_type,
            enabled: true,
            default_params: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_snake_case_serde() {
        // Verify snake_case serde produces "openai" not "OpenAI"
        let pt = ProviderType::OpenAI;
        let json = serde_json::to_string(&pt).expect("serialize failed");
        assert_eq!(
            json, "\"openai\"",
            "ProviderType::OpenAI should serialize as \"openai\""
        );

        // Verify round-trip
        let deserialized: ProviderType = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(pt, deserialized);
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
            ProviderType::CustomOpenAICompatible,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize failed");
            let deserialized: ProviderType =
                serde_json::from_str(&json).expect("deserialize failed");
            assert_eq!(
                variant, &deserialized,
                "serde roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn test_requires_api_key_cloud_providers() {
        assert!(ProviderType::OpenAI.requires_api_key());
        assert!(ProviderType::Anthropic.requires_api_key());
        assert!(ProviderType::CustomOpenAICompatible.requires_api_key());
        assert!(
            !ProviderType::Ollama.requires_api_key(),
            "Ollama should NOT require API key"
        );
    }

    #[test]
    fn test_provider_config_new() {
        let config = ProviderConfig::new("My Provider".to_string(), ProviderType::OpenAI);
        assert!(!config.id.is_empty());
        assert!(Uuid::parse_str(&config.id).is_ok());
        assert!(config.enabled);
        assert_eq!(
            config.base_url,
            Some("https://api.openai.com/v1".to_string())
        );
        assert_eq!(config.model_name, "gpt-4o-mini");
        assert_eq!(
            config.models.len(),
            1,
            "new provider should have 1 default model"
        );
        assert_eq!(config.models[0].name, "gpt-4o-mini");
        assert!(config.models[0].enabled);
        assert!(config.default_params.is_none());
    }

    #[test]
    fn test_provider_config_serde_roundtrip() {
        let config = ProviderConfig::new("Test Provider".to_string(), ProviderType::Anthropic);
        let json = serde_json::to_string(&config).expect("serialize failed");
        let deserialized: ProviderConfig = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(config.id, deserialized.id);
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.provider_type, deserialized.provider_type);
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.models.len(), deserialized.models.len());
    }
}
