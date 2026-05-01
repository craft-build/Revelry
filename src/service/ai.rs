use crate::domain::conversation::MessageRole;
use crate::domain::provider::{ProviderConfig, ProviderType};

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
    /// Builds a streaming request for the given provider and messages.
    /// Returns a stream of tokens that can be consumed incrementally.
    ///
    /// This is a placeholder for the full rig integration that will be
    /// implemented in Phase 3 Plan 02 when the ChatPanel component is built.
    pub fn stream_chat(
        _provider: &ProviderConfig,
        _system_prompt: &str,
        _user_message: &str,
        _history: &[(MessageRole, String)],
    ) -> impl futures::Stream<Item = StreamChunk> {
        // Placeholder implementation - returns an empty stream
        // Full implementation will use rig-core in Plan 02
        futures::stream::iter(vec![])
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
}
