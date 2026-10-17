//! Embedding service: resolves the configured embedder (a remote provider via
//! rig — including OpenAI-compatible endpoints like Synthetic — or a local
//! fastembed model), caches query vectors, and maintains the derived vector
//! index in SQLite. Everything here fails soft: callers treat any error as
//! "embeddings unavailable" and fall back to the no-embedding path.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::oneshot;

use crate::db::embeddings::EmbeddingRow;
use crate::db::DbCommand;
use crate::domain::embedding::EmbeddingConfig;
use crate::domain::provider::ProviderConfig;
use crate::service::vector::content_hash;

/// app_settings key holding the serialized `EmbeddingConfig`.
pub const EMBEDDING_CONFIG_KEY: &str = "embedding_config";

/// app_settings key for the cross-conversation character-memory toggle
/// ("true"/"false", default off).
pub const CROSS_MEMORY_SETTING_KEY: &str = "memory_cross_conversation";

/// Maximum cross-conversation echoes recalled per prompt.
pub const EXTERNAL_MEMORY_LIMIT: usize = 5;

/// Max characters of text fed to an embedder (rough cap on cost/latency).
pub const MAX_EMBED_TEXT_CHARS: usize = 8000;

/// Timeout for embedding round-trips on the interactive path (query embeds
/// during prompt assembly). Background maintenance has no timeout.
pub const EMBED_TIMEOUT_SECS: u64 = 3;

/// Query cache capacity per model; cleared wholesale when full.
const QUERY_CACHE_CAPACITY: usize = 64;

// ── Embedder ────────────────────────────────────────────────────────────────

/// A resolved embedding model behind a stable `model_key`.
pub enum Embedder {
    Remote(rig::DynModel<rig::operation::Embedding>),
    Local(rig::DynModel<rig::operation::Embedding>),
}

impl Embedder {
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, String> {
        match self {
            Embedder::Remote(model) => {
                let embedding = model.embed_text(text).await.map_err(|e| e.to_string())?;
                Ok(embedding.vec.into_iter().map(|v| v as f32).collect())
            }
            Embedder::Local(model) => {
                // fastembed inference blocks the calling thread (in-process
                // ONNX) — keep it off the async runtime workers.
                let model = Arc::new(model.clone());
                let text = text.to_string();
                let handle = tokio::task::spawn_blocking(move || {
                    futures::executor::block_on(async {
                        model.embed_text(&text).await.map_err(|e| e.to_string())
                    })
                })
                .await
                .map_err(|e| format!("local embedding task failed: {e}"))??;
                Ok(handle.vec.into_iter().map(|v| v as f32).collect())
            }
        }
    }
}

/// Local fastembed models offered in Settings. Keys are stable config strings.
pub fn local_model_choices() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "nomic-embed-text-v1.5",
            "Nomic embed v1.5 · 768 dims · ~100 MB",
        ),
        ("bge-small-en-v1.5", "BGE small EN v1.5 · 384 dims · ~35 MB"),
    ]
}

fn local_model_from_key(key: &str) -> Result<rig::fastembed::FastembedModel, String> {
    match key {
        "nomic-embed-text-v1.5" => Ok(rig::fastembed::FastembedModel::NomicEmbedTextV15),
        "bge-small-en-v1.5" => Ok(rig::fastembed::FastembedModel::BGESmallENV15),
        other => Err(format!("Unknown local embedding model: {other}")),
    }
}

/// Stable identity of an embedding space. Stored on every row; switching to a
/// different key invalidates the index until rebuilt.
pub fn model_key_for(
    config: &EmbeddingConfig,
    provider_type: Option<&crate::domain::provider::ProviderType>,
) -> String {
    match config {
        EmbeddingConfig::None => String::new(),
        EmbeddingConfig::Remote { model, .. } => {
            let slug = provider_type
                .and_then(|t| serde_json::to_value(t).ok())
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".to_string());
            format!("remote:{slug}:{model}")
        }
        EmbeddingConfig::Local { model } => format!("local:{model}"),
    }
}

// ── Runtime (managed Tauri state) ───────────────────────────────────────────

pub struct LoadedEmbedder {
    pub model_key: String,
    pub embedder: Embedder,
}

struct RuntimeInner {
    /// Cached resolved embedder for the current config.
    loaded: Option<Arc<LoadedEmbedder>>,
    /// Loaded local fastembed handle keyed by the model string; clones share
    /// the ONNX session so the (blocking, downloading) load happens once.
    local_handle: Option<(String, Arc<rig::fastembed::Fastembed>)>,
    /// (text, model_key) → vector. Small LRU-ish cache cleared when full.
    query_cache: HashMap<(String, String), Vec<f32>>,
}

/// Shared embedding state: cached embedder, cached query vectors, and a
/// single-flight guard for index rebuilds. Cheap to clone (Arc inside).
#[derive(Clone)]
pub struct EmbeddingRuntime {
    inner: Arc<Mutex<RuntimeInner>>,
    rebuild_running: Arc<AtomicBool>,
}

impl Default for EmbeddingRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeInner {
                loaded: None,
                local_handle: None,
                query_cache: HashMap::new(),
            })),
            rebuild_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Drops cached embedder + query vectors (config changed).
    pub fn invalidate(&self) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).loaded = None;
    }

    /// Resolves the current config into a cached embedder. Errors mean
    /// "embeddings unavailable" and must never be fatal to the caller.
    pub async fn resolve(
        &self,
        db: &std::sync::mpsc::Sender<DbCommand>,
    ) -> Result<Arc<LoadedEmbedder>, String> {
        if let Some(loaded) = self.cached() {
            return Ok(loaded);
        }

        let config = load_embedding_config(db).await?;
        let loaded = self.build(&config, db).await?;
        let loaded = Arc::new(loaded);
        {
            let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            guard.loaded = Some(loaded.clone());
        }
        Ok(loaded)
    }

    fn cached(&self) -> Option<Arc<LoadedEmbedder>> {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .loaded
            .clone()
    }

    async fn build(
        &self,
        config: &EmbeddingConfig,
        db: &std::sync::mpsc::Sender<DbCommand>,
    ) -> Result<LoadedEmbedder, String> {
        match config {
            EmbeddingConfig::None => Err("Embeddings are not configured".to_string()),
            EmbeddingConfig::Remote { provider_id, model } => {
                if model.trim().is_empty() {
                    return Err("Embedding model is not set".to_string());
                }
                let provider = load_provider(db, provider_id)
                    .await?
                    .ok_or_else(|| "Embedding provider not found".to_string())?;
                if !provider.enabled {
                    return Err("Embedding provider is disabled".to_string());
                }
                let api_key = load_provider_key(&provider).await?;
                let embedder = build_remote_embedder(&provider, model, &api_key)?;
                Ok(LoadedEmbedder {
                    model_key: model_key_for(config, Some(&provider.provider_type)),
                    embedder,
                })
            }
            EmbeddingConfig::Local { model } => {
                let fast_model = local_model_from_key(model)?;
                let handle = self.local_handle(&fast_model, model).await?;
                // A cheap, fresh wire each resolution; the transport (handle)
                // is the expensive part and stays cached above.
                let model = handle
                    .embedding(&fast_model, None)
                    .map_err(|e| e.to_string())?;
                Ok(LoadedEmbedder {
                    model_key: model_key_for(config, None),
                    embedder: Embedder::Local(model.into()),
                })
            }
        }
    }

    async fn local_handle(
        &self,
        fast_model: &rig::fastembed::FastembedModel,
        model_key: &str,
    ) -> Result<Arc<rig::fastembed::Fastembed>, String> {
        if let Some((key, handle)) = self
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .local_handle
            .clone()
        {
            if key == model_key {
                return Ok(handle);
            }
        }

        // First use downloads the model — keep the blocking load off the
        // async runtime and surface failures as plain strings.
        let fast_model = fast_model.clone();
        let handle = tokio::task::spawn_blocking(move || {
            rig::fastembed::Fastembed::load(&fast_model).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| format!("local model load task failed: {e}"))??;
        let handle = Arc::new(handle);

        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .local_handle = Some((model_key.to_string(), handle.clone()));
        Ok(handle)
    }

    /// Embeds a query text with a small cache. This is the interactive path
    /// (lorebook scan window, memory recall query) — errors fail open.
    pub async fn embed_query(
        &self,
        db: &std::sync::mpsc::Sender<DbCommand>,
        text: &str,
    ) -> Result<Vec<f32>, String> {
        let loaded = self.resolve(db).await?;
        let text: String = text.chars().take(MAX_EMBED_TEXT_CHARS).collect();
        let cache_key = (text.clone(), loaded.model_key.clone());

        if let Some(vector) = self
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .query_cache
            .get(&cache_key)
            .cloned()
        {
            return Ok(vector);
        }

        let vector = loaded.embedder.embed_text(&text).await?;

        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if guard.query_cache.len() >= QUERY_CACHE_CAPACITY {
            guard.query_cache.clear();
        }
        guard.query_cache.insert(cache_key, vector.clone());
        Ok(vector)
    }

    /// True while a rebuild task is in flight (used to single-flight).
    pub fn rebuild_in_progress(&self) -> bool {
        self.rebuild_running.load(Ordering::Relaxed)
    }
}

// ── Remote embedder construction (per provider type) ────────────────────────

/// Builds a rig embedding model from a provider config. Providers without a
/// rig embedding client are rejected with a clear message; the OpenAI client
/// (with custom base URL) covers the whole OpenAI-compatible family.
fn build_remote_embedder(
    provider: &ProviderConfig,
    model: &str,
    api_key: &str,
) -> Result<Embedder, String> {
    use crate::domain::provider::ProviderType;
    let base_url = provider.base_url.as_deref();
    match provider.provider_type {
        ProviderType::OpenAI => {
            let client = crate::ai::providers::build_openai_client(api_key, base_url);
            Ok(Embedder::Remote(
                client.embedding(model.to_string(), None).into(),
            ))
        }
        ProviderType::CustomOpenAICompatible | ProviderType::AzureOpenAI => {
            let url = base_url.ok_or("Base URL required for this provider type")?;
            let client = crate::ai::providers::build_openai_client(api_key, Some(url));
            Ok(Embedder::Remote(
                client.embedding(model.to_string(), None).into(),
            ))
        }
        ProviderType::Ollama => {
            let url = base_url.unwrap_or("http://localhost:11434");
            let client = rig::providers::ollama::OllamaConfig::new()
                .with_base_url(url)
                .client();
            Ok(Embedder::Remote(
                client.embedding(model.to_string(), None).into(),
            ))
        }
        ProviderType::Gemini => {
            let client = rig::providers::gemini::Gemini::new(api_key);
            Ok(Embedder::Remote(
                client.embedding(model.to_string(), None).into(),
            ))
        }
        ProviderType::Cohere => {
            let client = rig::providers::cohere::Cohere::new(api_key);
            Ok(Embedder::Remote(
                client.embedding(model.to_string(), None).into(),
            ))
        }
        // These providers route their /models and (where present) /embeddings
        // endpoints through the OpenAI-compatible shape at their base URL.
        ProviderType::Groq
        | ProviderType::Together
        | ProviderType::Xai
        | ProviderType::Perplexity
        | ProviderType::HuggingFace
        | ProviderType::DeepSeek
        | ProviderType::OpenRouter => {
            let url = base_url
                .or_else(|| provider.provider_type.default_base_url())
                .ok_or("Base URL required for this provider type")?;
            let client = crate::ai::providers::build_openai_client(api_key, Some(url));
            Ok(Embedder::Remote(
                client.embedding(model.to_string(), None).into(),
            ))
        }
        ProviderType::Anthropic | ProviderType::Mistral => Err(format!(
            "{} does not expose an embedding API",
            serde_json::to_value(&provider.provider_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "This provider".to_string())
        )),
    }
}

// ── DB helpers ──────────────────────────────────────────────────────────────

/// Runs a DbCommand and awaits its typed reply. Both channel failure and the
/// inner anyhow error surface as plain strings.
pub(crate) async fn db_call<T, F>(
    db: &std::sync::mpsc::Sender<DbCommand>,
    make: F,
) -> anyhow::Result<T>
where
    F: FnOnce(oneshot::Sender<anyhow::Result<T>>) -> DbCommand,
    T: Send + 'static,
{
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(make(reply_tx)).map_err(|e| anyhow::anyhow!(e))?;
    match reply_rx.await {
        Ok(inner) => inner,
        Err(e) => anyhow::bail!("DB worker dropped reply: {e}"),
    }
}

pub async fn load_embedding_config(
    db: &std::sync::mpsc::Sender<DbCommand>,
) -> Result<EmbeddingConfig, String> {
    let raw = db_call(db, |reply| DbCommand::LoadAppSetting {
        key: EMBEDDING_CONFIG_KEY.to_string(),
        reply,
    })
    .await
    .map_err(|e| e.to_string())?;

    match raw {
        Some(json) if !json.trim().is_empty() => {
            serde_json::from_str(&json).map_err(|e| format!("Invalid embedding config: {e}"))
        }
        _ => Ok(EmbeddingConfig::None),
    }
}

pub async fn save_embedding_config(
    db: &std::sync::mpsc::Sender<DbCommand>,
    config: &EmbeddingConfig,
) -> Result<(), String> {
    let json = serde_json::to_string(config).map_err(|e| e.to_string())?;
    db_call(db, |reply| DbCommand::SaveAppSetting {
        key: EMBEDDING_CONFIG_KEY.to_string(),
        value: json,
        reply,
    })
    .await
    .map_err(|e| e.to_string())
}

async fn load_provider(
    db: &std::sync::mpsc::Sender<DbCommand>,
    provider_id: &str,
) -> Result<Option<ProviderConfig>, String> {
    let providers = db_call(db, |reply| DbCommand::LoadProviders { reply })
        .await
        .map_err(|e| e.to_string())?;
    Ok(providers.into_iter().find(|p| p.id == provider_id))
}

async fn load_provider_key(provider: &ProviderConfig) -> Result<String, String> {
    if !provider.provider_type.requires_api_key() {
        return Ok(String::new());
    }
    let provider_id = provider.id.clone();
    tokio::task::spawn_blocking(move || crate::secrets::KeyringStore::load_key(&provider_id))
        .await
        .map_err(|e| format!("Keyring task failed: {e}"))?
        .map_err(|e| format!("No API key for the embedding provider: {e}"))
}

// ── Index maintenance (write reconciliation) ────────────────────────────────

/// Embeds one source text into the index, skipping when the stored
/// content_hash already matches (edits re-embed, no-op saves don't).
pub async fn embed_source(
    runtime: &EmbeddingRuntime,
    db: &std::sync::mpsc::Sender<DbCommand>,
    namespace: &str,
    ref_id: &str,
    text: &str,
) -> Result<(), String> {
    let text: String = text.chars().take(MAX_EMBED_TEXT_CHARS).collect();
    if text.trim().is_empty() {
        return Ok(());
    }

    let loaded = runtime.resolve(db).await?;
    let hash = content_hash(&text);

    let existing = db_call(db, |reply| DbCommand::GetEmbedding {
        namespace: namespace.to_string(),
        ref_id: ref_id.to_string(),
        reply,
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Some(record) = existing {
        if record.model_key == loaded.model_key && record.content_hash == hash {
            return Ok(()); // Unchanged — nothing to do.
        }
    }

    let vector = loaded.embedder.embed_text(&text).await?;
    let row = EmbeddingRow::new(namespace, ref_id, &loaded.model_key, &vector, &hash);
    db_call(db, |reply| DbCommand::UpsertEmbedding {
        row: Box::new(row),
        reply,
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Fire-and-forget message embed (called at message finalize/save).
pub fn spawn_embed_message(
    runtime: EmbeddingRuntime,
    db: std::sync::mpsc::Sender<DbCommand>,
    conversation_id: String,
    message_id: String,
    content: String,
) {
    if content.trim().is_empty() {
        return;
    }
    tokio::spawn(async move {
        let namespace = format!("messages:{conversation_id}");
        if let Err(e) = embed_source(&runtime, &db, &namespace, &message_id, &content).await {
            tracing::debug!("message embed skipped: {e}");
        }
    });
}

/// Fire-and-forget lorebook entry embed (called at entry save).
pub fn spawn_embed_lorebook_entry(
    runtime: EmbeddingRuntime,
    db: std::sync::mpsc::Sender<DbCommand>,
    entry: crate::domain::lorebook::LorebookEntry,
) {
    if entry.content.trim().is_empty() {
        return;
    }
    tokio::spawn(async move {
        let namespace = format!("lorebook:{}", entry.lorebook_id);
        if let Err(e) = embed_source(&runtime, &db, &namespace, &entry.id, &entry.content).await {
            tracing::debug!("lorebook entry embed skipped: {e}");
        }
    });
}

/// Fire-and-forget vector delete (message / entry delete).
pub fn spawn_delete_embedding(db: std::sync::mpsc::Sender<DbCommand>, ref_id: String) {
    tokio::spawn(async move {
        if let Err(e) = db_call(&db, |reply| DbCommand::DeleteEmbeddingByRef {
            ref_id,
            reply,
        })
        .await
        {
            tracing::warn!("embedding delete failed: {e}");
        }
    });
}

/// Rebuilds the whole index for the current config. Single-flighted; when the
/// config is `None` the index is simply cleared.
pub fn spawn_rebuild_index(runtime: EmbeddingRuntime, db: std::sync::mpsc::Sender<DbCommand>) {
    if runtime.rebuild_running.swap(true, Ordering::Relaxed) {
        return; // One rebuild at a time.
    }
    tokio::spawn(async move {
        let result = rebuild_index(&runtime, &db).await;
        runtime.rebuild_running.store(false, Ordering::Relaxed);
        if let Err(e) = result {
            tracing::warn!("embedding index rebuild failed: {e}");
        }
    });
}

async fn rebuild_index(
    runtime: &EmbeddingRuntime,
    db: &std::sync::mpsc::Sender<DbCommand>,
) -> Result<(), String> {
    let config = load_embedding_config(db).await?;

    // Collect existing model keys so stale rows can be dropped at the end.
    let counts = db_call(db, |reply| DbCommand::CountEmbeddings { reply })
        .await
        .map_err(|e| e.to_string())?;

    let target_key = match &config {
        EmbeddingConfig::None => {
            // Disabled: the derived index is worthless — clear it.
            for (model_key, _, _) in &counts {
                db_call(db, |reply| DbCommand::DeleteEmbeddingsByModel {
                    model_key: model_key.clone(),
                    reply,
                })
                .await
                .map_err(|e| e.to_string())?;
            }
            tracing::info!("embedding index cleared (embeddings disabled)");
            return Ok(());
        }
        _ => runtime.resolve(db).await?.model_key.clone(),
    };

    // Lorebook entries.
    let lorebooks = db_call(db, |reply| DbCommand::LoadLorebooks { reply })
        .await
        .map_err(|e| e.to_string())?;
    let lorebook_ids: Vec<String> = lorebooks.iter().map(|lb| lb.id.clone()).collect();
    let entries = db_call(db, |reply| DbCommand::LoadLorebookEntriesForLorebooks {
        lorebook_ids: lorebook_ids.clone(),
        reply,
    })
    .await
    .map_err(|e| e.to_string())?;
    for entry in &entries {
        if entry.content.trim().is_empty() {
            continue;
        }
        let namespace = format!("lorebook:{}", entry.lorebook_id);
        embed_source(runtime, db, &namespace, &entry.id, &entry.content).await?;
    }

    // Messages (root branch of every conversation).
    let conversations = db_call(db, |reply| DbCommand::LoadConversations { reply })
        .await
        .map_err(|e| e.to_string())?;
    for conversation in &conversations {
        let messages = db_call(db, |reply| DbCommand::LoadMessages {
            conversation_id: conversation.id.clone(),
            branch_id: None,
            reply,
        })
        .await
        .map_err(|e| e.to_string())?;
        let namespace = format!("messages:{}", conversation.id);
        for message in &messages {
            if message.content.trim().is_empty() {
                continue;
            }
            embed_source(runtime, db, &namespace, &message.id, &message.content).await?;
        }
    }

    // Drop rows from previous models.
    for (model_key, _, _) in &counts {
        if model_key != &target_key {
            db_call(db, |reply| DbCommand::DeleteEmbeddingsByModel {
                model_key: model_key.clone(),
                reply,
            })
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    tracing::info!("embedding index rebuilt for {target_key}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_model_choices_have_keys() {
        assert!(local_model_choices().iter().all(|(key, _)| !key.is_empty()));
    }

    #[test]
    fn test_local_model_from_key() {
        assert!(local_model_from_key("nomic-embed-text-v1.5").is_ok());
        assert!(local_model_from_key("bge-small-en-v1.5").is_ok());
        assert!(local_model_from_key("nope").is_err());
    }

    #[test]
    fn test_model_key_for() {
        use crate::domain::provider::ProviderType;
        assert_eq!(model_key_for(&EmbeddingConfig::None, None), "");

        let remote = EmbeddingConfig::Remote {
            provider_id: "p1".to_string(),
            model: "nomic-embed-text-v1.5".to_string(),
        };
        let key = model_key_for(&remote, Some(&ProviderType::CustomOpenAICompatible));
        assert_eq!(key, "remote:custom_openai_compatible:nomic-embed-text-v1.5");

        let local = EmbeddingConfig::Local {
            model: "bge-small-en-v1.5".to_string(),
        };
        assert_eq!(model_key_for(&local, None), "local:bge-small-en-v1.5");
    }
}
