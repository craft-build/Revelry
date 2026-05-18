use crate::domain::conversation::Message;
use crate::domain::provider::ProviderConfig;
use std::sync::Mutex;
use tiktoken_rs::CoreBPE;

/// Default context window size used when no provider override or model match exists.
/// Set to 8192 as a conservative default (matches Llama 3 base context).
pub const DEFAULT_CONTEXT_WINDOW: i64 = 8192;

/// Service for counting tokens using the tiktoken-rs cl100k_base BPE tokenizer.
///
/// Per D-03: Uses a single tokenizer (cl100k_base) for all providers.
/// Applies a 10% safety margin on all token counts (round up: count + count / 10).
/// The tokenizer is lazily initialized on first use and cached behind a Mutex.
pub struct TokenCountService {
    bpe: Mutex<Option<CoreBPE>>,
}

impl TokenCountService {
    /// Creates a new TokenCountService with an uninitialized tokenizer.
    /// The tokenizer will be lazily initialized on first call to count_tokens.
    pub fn new() -> Self {
        Self {
            bpe: Mutex::new(None),
        }
    }

    /// Counts tokens in the given text using cl100k_base BPE tokenization.
    /// Applies a 10% safety margin (round up: count + count / 10).
    /// Per D-03, a single tokenizer is used for all providers.
    pub fn count_tokens(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        let guard = self.get_bpe();
        let count = count_with_guard(&guard, text);
        // Apply 10% safety margin (round up)
        count + count / 10
    }

    /// Resolves the context window size for a provider using a 4-tier fallback:
    /// 1. If provider.context_window_size is Some(n) and n > 0, return n (provider-level override)
    /// 2. Per-model entry override from provider.models matching the active model name
    /// 3. Look up active model name in built-in defaults table
    /// 4. Fall back to DEFAULT_CONTEXT_WINDOW
    /// Per D-02.
    pub fn resolve_context_window_size(
        &self,
        provider: &ProviderConfig,
        active_model_name: &str,
    ) -> i64 {
        // Tier 1: Provider-level user override
        if let Some(size) = provider.context_window_size {
            if size > 0 {
                return size;
            }
        }

        // Tier 2: Per-model entry override
        if let Some(entry) = provider.models.iter().find(|m| m.name == active_model_name) {
            if let Some(size) = entry.context_window_size {
                if size > 0 {
                    return size;
                }
            }
        }

        // Tier 3: Built-in model defaults
        let defaults = built_in_model_context_windows();
        for (pattern, size) in defaults {
            if active_model_name.contains(pattern) {
                return size;
            }
        }

        // Tier 4: Default fallback
        DEFAULT_CONTEXT_WINDOW
    }

    /// Counts the total tokens used by a preamble and a list of messages.
    /// Each message has ~4 tokens of role overhead plus its content tokens.
    pub fn count_message_tokens(&self, preamble: &str, messages: &[Message]) -> usize {
        let preamble_tokens = self.count_tokens(preamble);
        let message_tokens: usize = messages
            .iter()
            .map(|msg| {
                // Role overhead (~4 tokens) + content tokens
                4 + self.count_tokens(&msg.content)
            })
            .sum();
        preamble_tokens + message_tokens
    }

    /// Lazily initializes and returns a reference to the cached BPE tokenizer.
    fn get_bpe(&self) -> std::sync::MutexGuard<'_, Option<CoreBPE>> {
        let mut guard = self
            .bpe
            .lock()
            .expect("TokenCountService bpe mutex poisoned");
        if guard.is_none() {
            *guard = Some(
                tiktoken_rs::cl100k_base().expect("Failed to initialize cl100k_base tokenizer"),
            );
        }
        guard
    }
}

/// Helper to count tokens using a MutexGuard<Option<CoreBPE>>.
/// The guard is always Some after initialization, so we unwrap the Option.
fn count_with_guard(guard: &std::sync::MutexGuard<'_, Option<CoreBPE>>, text: &str) -> usize {
    guard
        .as_ref()
        .expect("BPE should be initialized")
        .count_with_special_tokens(text)
}

impl Default for TokenCountService {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the built-in model context window sizes for known models.
/// Uses model name substring matching for flexibility.
/// Per D-02, this is the second tier in the 3-tier fallback.
fn built_in_model_context_windows() -> Vec<(&'static str, i64)> {
    vec![
        // OpenAI models
        ("gpt-4o", 128000),
        ("gpt-4-turbo", 128000),
        // Anthropic models
        ("claude-sonnet-4", 200000),
        ("claude-3-5-sonnet", 200000),
        // Local models
        ("llama3.2", 8192),
        ("llama-3.3-70b", 128000),
        // Other providers
        ("deepseek-chat", 65536),
        ("mistral-large", 128000),
        ("gemini-2.0-flash", 1048576),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::{Message as DomainMessage, MessageRole};
    use crate::domain::provider::{ProviderConfig, ProviderType};
    use chrono::Utc;

    fn make_test_message(role: MessageRole, content: &str) -> DomainMessage {
        DomainMessage {
            id: "test-id".to_string(),
            conversation_id: "conv-1".to_string(),
            role,
            content: content.to_string(),
            sort_order: 0,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_count_tokens_positive() {
        let service = TokenCountService::new();
        let count = service.count_tokens("hello world");
        // cl100k_base: "hello world" = 2 tokens, with 10% margin: 2 + 0 = 2
        assert!(
            count > 0,
            "Token count should be positive for non-empty text"
        );
        assert_eq!(count, 2, "hello world should be 2 tokens with 10% margin");
    }

    #[test]
    fn test_count_tokens_empty() {
        let service = TokenCountService::new();
        let count = service.count_tokens("");
        assert_eq!(count, 0, "Empty string should have 0 tokens");
    }

    #[test]
    fn test_default_context_window() {
        assert_eq!(DEFAULT_CONTEXT_WINDOW, 8192);
    }

    #[test]
    fn test_resolve_context_window_user_override() {
        let service = TokenCountService::new();
        let mut provider = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        provider.context_window_size = Some(50000);
        let size = service.resolve_context_window_size(&provider, &provider.model_name);
        assert_eq!(size, 50000, "Should return user override when present");
    }

    #[test]
    fn test_resolve_context_window_builtin_default() {
        let service = TokenCountService::new();
        let provider = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        // ProviderType::OpenAI defaults to model_name "gpt-4o-mini"
        // "gpt-4o-mini" contains "gpt-4o" → should match 128000
        let size = service.resolve_context_window_size(&provider, &provider.model_name);
        assert_eq!(
            size, 128000,
            "Should return built-in default for known model"
        );
    }

    #[test]
    fn test_resolve_context_window_unknown_model() {
        let service = TokenCountService::new();
        let mut provider =
            ProviderConfig::new("Test".to_string(), ProviderType::CustomOpenAICompatible);
        provider.model_name = "some-unknown-model-v1".to_string();
        let size = service.resolve_context_window_size(&provider, &provider.model_name);
        assert_eq!(
            size, DEFAULT_CONTEXT_WINDOW,
            "Should fall back to DEFAULT for unknown model"
        );
    }

    #[test]
    fn test_builtin_model_table_entries() {
        let defaults = built_in_model_context_windows();
        // Check that key entries exist
        let gpt4o = defaults.iter().find(|(p, _)| *p == "gpt-4o");
        assert!(gpt4o.is_some(), "Should have gpt-4o entry");
        assert_eq!(gpt4o.unwrap().1, 128000);

        let claude = defaults.iter().find(|(p, _)| *p == "claude-sonnet-4");
        assert!(claude.is_some(), "Should have claude-sonnet-4 entry");
        assert_eq!(claude.unwrap().1, 200000);

        let llama = defaults.iter().find(|(p, _)| *p == "llama3.2");
        assert!(llama.is_some(), "Should have llama3.2 entry");
        assert_eq!(llama.unwrap().1, 8192);
    }

    #[test]
    fn test_provider_config_has_context_window_size() {
        let provider = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        // context_window_size should be None by default (new field)
        assert!(
            provider.context_window_size.is_none(),
            "context_window_size should default to None"
        );
    }
}
