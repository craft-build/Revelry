use serde::{Deserialize, Serialize};

/// User configuration for the embedding subsystem. Persisted as JSON under the
/// `embedding_config` app setting. Everything embedding-powered degrades to a
/// no-embedding path when this is `None` (or the source fails at runtime).
///
/// - `Remote` references an existing ProviderConfig (API key stays in the
///   keyring); OpenAI-compatible providers cover aggregators like Synthetic
///   (custom_openai_compatible + a model such as nomic-embed-text-v1.5).
/// - `Local` runs a fastembed model on-device (opt-in; the model downloads on
///   first use).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "source",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum EmbeddingConfig {
    #[default]
    None,
    Remote {
        provider_id: String,
        model: String,
    },
    Local {
        model: String,
    },
}

impl EmbeddingConfig {
    /// True when a source is selected (embeddings enabled).
    pub fn is_enabled(&self) -> bool {
        !matches!(self, EmbeddingConfig::None)
    }
}

/// One namespace's indexed vector count for the currently configured model.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingNamespaceCount {
    pub namespace: String,
    pub count: i64,
}

/// Runtime view of the embedding subsystem for Settings display.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingStatus {
    pub configured: bool,
    /// "none" | "remote" | "local"
    pub source: String,
    /// Stable identity of the embedding space, e.g.
    /// "remote:custom_openai_compatible:nomic-embed-text-v1.5" or
    /// "local:nomic-embed-text-v1.5". Empty when not configured.
    pub model_key: String,
    /// Rows indexed for the current model_key, grouped by namespace.
    pub counts: Vec<EmbeddingNamespaceCount>,
    /// Rows left over from a different model_key (stale until rebuilt).
    pub stale_count: i64,
}

/// Result of the Settings "test" round-trip.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingTestResult {
    pub dims: usize,
    pub latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serde_none() {
        let json = serde_json::to_string(&EmbeddingConfig::None).expect("ser failed");
        assert_eq!(json, r#"{"source":"none"}"#);
        let parsed: EmbeddingConfig = serde_json::from_str(&json).expect("de failed");
        assert_eq!(parsed, EmbeddingConfig::None);
    }

    #[test]
    fn test_config_serde_remote() {
        let config = EmbeddingConfig::Remote {
            provider_id: "prov-1".to_string(),
            model: "nomic-embed-text-v1.5".to_string(),
        };
        let json = serde_json::to_string(&config).expect("ser failed");
        assert!(json.contains(r#""source":"remote""#), "got: {json}");
        assert!(json.contains(r#""providerId":"prov-1""#), "got: {json}");
        assert!(
            json.contains(r#""model":"nomic-embed-text-v1.5""#),
            "got: {json}"
        );
        let parsed: EmbeddingConfig = serde_json::from_str(&json).expect("de failed");
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_config_serde_local() {
        let config = EmbeddingConfig::Local {
            model: "bge-small-en-v1.5".to_string(),
        };
        let json = serde_json::to_string(&config).expect("ser failed");
        assert!(json.contains(r#""source":"local""#), "got: {json}");
        let parsed: EmbeddingConfig = serde_json::from_str(&json).expect("de failed");
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_config_default_is_none() {
        assert_eq!(EmbeddingConfig::default(), EmbeddingConfig::None);
        assert!(!EmbeddingConfig::default().is_enabled());
        assert!(EmbeddingConfig::Local {
            model: "x".to_string()
        }
        .is_enabled());
    }
}
