//! AI service: streaming chat, character generation, embeddings, model listing.

mod chat;
pub(crate) mod embeddings;
mod generate;
mod model_list;
mod providers;
pub(crate) mod summary;
mod types;
mod util;

use crate::domain::provider::ProviderConfig;

pub use chat::PostStreamHook;
pub use types::{
    BudgetInfo, GenerateCharacterChunk, GeneratedCharacterData, GenerationParams, MemoryInfo,
    MessagePersistence, StreamChunk,
};

pub struct AiService;

impl AiService {
    /// Resolves GenerationParams from a ProviderConfig's default_params.
    /// Uses the `From<domain::provider::GenerationParams>` impl to convert
    /// the domain type (i64 max_tokens) to the AI type (u64 max_tokens).
    pub fn from_provider_config(provider: &ProviderConfig) -> GenerationParams {
        match &provider.default_params {
            Some(params) => params.clone().into(),
            None => GenerationParams::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::provider::ProviderType;

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
            ..Default::default()
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
            ..Default::default()
        });
        let params = AiService::from_provider_config(&provider);
        // max_tokens of 0 should map to None (unset)
        assert!(params.max_tokens.is_none());
    }
}
