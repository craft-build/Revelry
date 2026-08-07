use crate::domain::provider::{ProviderModelEntry, ProviderType};

use rig::client::ModelListingClient;
use rig::model::ModelList;

use super::providers::build_openai_client;
use super::AiService;

impl AiService {
    /// Fetch available models from a provider's Models API. Uses rig's native
    /// `list_models()` for the 7 supported providers; falls back to an OpenAI
    /// client at the provider's base URL for the OpenAI-compatible family.
    /// Azure OpenAI is unsupported (deployment-based auth).
    pub async fn list_provider_models(
        provider_type: &ProviderType,
        base_url: Option<&str>,
        api_key: &str,
    ) -> Result<Vec<ProviderModelEntry>, String> {
        let list: ModelList = match provider_type {
            ProviderType::OpenAI => {
                let client = build_openai_client(api_key, base_url)?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Anthropic => {
                let client = rig::providers::anthropic::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Ollama => {
                let url = base_url.unwrap_or("http://localhost:11434");
                let client = rig::providers::ollama::Client::builder()
                    .api_key(String::new())
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Gemini => {
                let client = rig::providers::gemini::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Mistral => {
                let client = rig::providers::mistral::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::DeepSeek => {
                let client = rig::providers::deepseek::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::OpenRouter => {
                let client = rig::providers::openrouter::Client::builder()
                    .api_key(api_key)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            // OpenAI-compatible fallback: these providers expose an
            // OpenAI-shaped `/models` endpoint even though rig does not
            // natively list models for them.
            ProviderType::Groq => {
                let url = base_url.unwrap_or("https://api.groq.com/openai/v1");
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Together => {
                let url = base_url.unwrap_or("https://api.together.xyz/v1");
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Xai => {
                let url = base_url.unwrap_or("https://api.x.ai/v1");
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Perplexity => {
                let url = base_url.unwrap_or("https://api.perplexity.ai");
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::HuggingFace => {
                let url = base_url.unwrap_or("https://api-inference.huggingface.co/v1");
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::Cohere => {
                let url = base_url.unwrap_or("https://api.cohere.com/v1");
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::CustomOpenAICompatible => {
                let url = base_url.ok_or(
                    "Base URL is required to detect models for a custom OpenAI-compatible provider.",
                )?;
                let client = rig::providers::openai::Client::builder()
                    .api_key(api_key)
                    .base_url(url)
                    .build()
                    .map_err(|e| e.to_string())?;
                client.list_models().await.map_err(|e| e.to_string())?
            }
            ProviderType::AzureOpenAI => {
                return Err(
                    "Model detection is not supported for Azure OpenAI. Azure uses deployment-based \
                     model naming and a different authentication scheme — add models manually."
                        .to_string(),
                );
            }
        };

        Ok(Self::model_list_to_entries(list))
    }

    /// Convert a rig `ModelList` into sorted, deduped `ProviderModelEntry` values.
    /// `id` becomes `name`; `context_length` (u32) becomes `context_window_size` (i64).
    fn model_list_to_entries(list: rig::model::ModelList) -> Vec<ProviderModelEntry> {
        let mut entries: Vec<ProviderModelEntry> = list
            .into_iter()
            .map(|m| ProviderModelEntry {
                name: m.id,
                context_window_size: m.context_length.map(|c| c as i64),
                enabled: true,
            })
            .collect();
        entries.sort_by_key(|a| a.name.to_lowercase());
        entries.dedup_by(|a, b| a.name == b.name);
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_list_to_entries_basic() {
        use rig::model::{Model, ModelList};
        let list = ModelList::new(vec![
            Model::from_id("gpt-4o-mini"),
            Model::from_id("gpt-4o"),
        ]);
        let entries = AiService::model_list_to_entries(list);
        assert_eq!(entries.len(), 2);
        // Sorted alphabetically by name (id)
        assert_eq!(entries[0].name, "gpt-4o");
        assert_eq!(entries[1].name, "gpt-4o-mini");
        assert!(entries.iter().all(|e| e.enabled));
        assert!(entries.iter().all(|e| e.context_window_size.is_none()));
    }

    #[test]
    fn test_model_list_to_entries_with_context_length() {
        use rig::model::{Model, ModelList};
        let mut model = Model::from_id("claude-3");
        model.context_length = Some(200000);
        let list = ModelList::new(vec![model]);
        let entries = AiService::model_list_to_entries(list);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].context_window_size, Some(200000));
    }

    #[test]
    fn test_model_list_to_entries_dedup_and_case_insensitive_sort() {
        use rig::model::{Model, ModelList};
        let list = ModelList::new(vec![
            Model::from_id("zeta"),
            Model::from_id("Alpha"),
            Model::from_id("zeta"), // duplicate
        ]);
        let entries = AiService::model_list_to_entries(list);
        // Deduped to 2 entries
        assert_eq!(entries.len(), 2);
        // Case-insensitive sort: Alpha < zeta
        assert_eq!(entries[0].name, "Alpha");
        assert_eq!(entries[1].name, "zeta");
    }
}
