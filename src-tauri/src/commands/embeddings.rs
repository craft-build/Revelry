use tauri::State;
use tokio::sync::oneshot;

use crate::ai::embeddings::{self, db_call, EmbeddingRuntime, EMBEDDING_CONFIG_KEY};
use crate::db::{DbCommand, DbWorker};
use crate::domain::embedding::{
    EmbeddingConfig, EmbeddingNamespaceCount, EmbeddingStatus, EmbeddingTestResult,
};

async fn load_config(db: &State<'_, DbWorker>) -> Result<EmbeddingConfig, String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::LoadAppSetting {
        key: EMBEDDING_CONFIG_KEY.to_string(),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    let raw = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {e}"))?
        .map_err(|e| e.to_string())?;

    match raw {
        Some(json) if !json.trim().is_empty() => {
            serde_json::from_str(&json).map_err(|e| format!("Invalid embedding config: {e}"))
        }
        _ => Ok(EmbeddingConfig::None),
    }
}

#[tauri::command]
pub async fn get_embedding_config(db: State<'_, DbWorker>) -> Result<EmbeddingConfig, String> {
    load_config(&db).await
}

/// Saves the embedding config. When the source's model key changes, the
/// runtime cache is invalidated and the index rebuilds in the background.
#[tauri::command]
pub async fn set_embedding_config(
    db: State<'_, DbWorker>,
    runtime: State<'_, EmbeddingRuntime>,
    config: EmbeddingConfig,
) -> Result<(), String> {
    // Validate remote references a usable provider before persisting.
    if let EmbeddingConfig::Remote { provider_id, model } = &config {
        if model.trim().is_empty() {
            return Err("Embedding model must not be empty".to_string());
        }
        let (reply_tx, reply_rx) = oneshot::channel();
        db.send(DbCommand::LoadProviders { reply: reply_tx })
            .map_err(|e| e.to_string())?;
        let providers = reply_rx
            .await
            .map_err(|e| format!("DB worker dropped: {e}"))?
            .map_err(|e| e.to_string())?;
        let provider = providers
            .into_iter()
            .find(|p| &p.id == provider_id)
            .ok_or_else(|| "Embedding provider not found".to_string())?;
        if !provider.enabled {
            return Err("Embedding provider is disabled".to_string());
        }
        if provider.provider_type.requires_api_key() {
            let pid = provider.id.clone();
            let has_key = tokio::task::spawn_blocking(move || {
                crate::secrets::KeyringStore::load_key(&pid).is_ok()
            })
            .await
            .map_err(|e| format!("Keyring task failed: {e}"))?;
            if !has_key {
                return Err("The selected provider has no API key".to_string());
            }
        }
    }

    if let EmbeddingConfig::Local { model } = &config {
        // Reject unknown local model keys up front.
        embeddings::local_model_choices()
            .iter()
            .find(|(key, _)| key == model)
            .ok_or_else(|| format!("Unknown local embedding model: {model}"))?;
    }

    embeddings::save_embedding_config(&db.sender(), &config).await?;
    runtime.invalidate();
    embeddings::spawn_rebuild_index(runtime.inner().clone(), db.sender());
    Ok(())
}

#[tauri::command]
pub async fn get_embedding_status(db: State<'_, DbWorker>) -> Result<EmbeddingStatus, String> {
    let config = load_config(&db).await?;

    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::CountEmbeddings { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    let counts = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {e}"))?
        .map_err(|e| e.to_string())?;

    // The current model key needs the provider type for remote configs.
    let model_key = match &config {
        EmbeddingConfig::Remote { provider_id, .. } => {
            let (reply_tx, reply_rx) = oneshot::channel();
            db.send(DbCommand::LoadProviders { reply: reply_tx })
                .map_err(|e| e.to_string())?;
            let providers = reply_rx
                .await
                .map_err(|e| format!("DB worker dropped: {e}"))?
                .map_err(|e| e.to_string())?;
            providers
                .into_iter()
                .find(|p| &p.id == provider_id)
                .map(|p| embeddings::model_key_for(&config, Some(&p.provider_type)))
                .unwrap_or_default()
        }
        _ => embeddings::model_key_for(&config, None),
    };

    let mut namespace_counts: Vec<EmbeddingNamespaceCount> = Vec::new();
    let mut stale_count = 0i64;
    for (row_model, namespace, count) in counts {
        if row_model == model_key {
            if let Some(existing) = namespace_counts
                .iter_mut()
                .find(|c| c.namespace == namespace)
            {
                existing.count += count;
            } else {
                namespace_counts.push(EmbeddingNamespaceCount { namespace, count });
            }
        } else {
            stale_count += count;
        }
    }

    let source = match &config {
        EmbeddingConfig::None => "none",
        EmbeddingConfig::Remote { .. } => "remote",
        EmbeddingConfig::Local { .. } => "local",
    }
    .to_string();

    Ok(EmbeddingStatus {
        configured: config.is_enabled(),
        source,
        model_key,
        counts: namespace_counts,
        stale_count,
    })
}

/// Round-trips a short text through the configured embedder. First local use
/// loads (and possibly downloads) the model, so this may take a while.
#[tauri::command]
pub async fn test_embedding(
    db: State<'_, DbWorker>,
    runtime: State<'_, EmbeddingRuntime>,
    text: Option<String>,
) -> Result<EmbeddingTestResult, String> {
    let text = text.unwrap_or_else(|| "The fox runs through the night.".to_string());
    let sender = db.sender();
    let runtime = (*runtime).clone();

    let started = std::time::Instant::now();
    let loaded = runtime.resolve(&sender).await?;
    let vector = loaded.embedder.embed_text(&text).await?;
    if vector.is_empty() {
        return Err("The embedding provider returned an empty vector".to_string());
    }

    Ok(EmbeddingTestResult {
        dims: vector.len(),
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

/// Manually triggers a full index rebuild for the current config.
#[tauri::command]
pub async fn rebuild_embedding_index(
    db: State<'_, DbWorker>,
    runtime: State<'_, EmbeddingRuntime>,
) -> Result<(), String> {
    if runtime.rebuild_in_progress() {
        return Ok(()); // Already running.
    }
    embeddings::spawn_rebuild_index(runtime.inner().clone(), db.sender());
    Ok(())
}

/// Excludes (or re-includes) one conversation from cross-conversation
/// memory injection — the durable form of "start fresh".
#[tauri::command]
pub async fn set_conversation_memory_enabled(
    db: State<'_, DbWorker>,
    conversation_id: String,
    enabled: bool,
) -> Result<(), String> {
    let key = embeddings::memory_disabled_key(&conversation_id);
    if enabled {
        // Absent = enabled; storing "false" keeps it simple and explicit.
        embeddings::write_app_setting(&db.sender(), &key, "false").await
    } else {
        embeddings::write_app_setting(&db.sender(), &key, "true").await
    }
}

/// Sets or clears a character's cross-conversation memory cutoff.
/// `forgotten: Some(true)` records "now" — the character recalls nothing
/// from before this moment; `Some(false)`/`None` restores full recall.
#[tauri::command]
pub async fn set_character_memory_forgotten(
    db: State<'_, DbWorker>,
    character_id: String,
    forgotten: Option<bool>,
) -> Result<(), String> {
    let key = embeddings::memory_forgotten_key(&character_id);
    match forgotten.unwrap_or(false) {
        true => {
            let now = jiff::Timestamp::now().to_string();
            embeddings::write_app_setting(&db.sender(), &key, &now).await
        }
        false => embeddings::write_app_setting(&db.sender(), &key, "").await,
    }
}

/// Returns one conversation's cross-conversation memory participation
/// (true unless explicitly disabled by "start fresh").
#[tauri::command]
pub async fn get_conversation_memory_enabled(
    db: State<'_, DbWorker>,
    conversation_id: String,
) -> Result<bool, String> {
    let key = embeddings::memory_disabled_key(&conversation_id);
    let value = embeddings::read_app_setting(&db.sender(), &key).await?;
    Ok(value.as_deref() != Some("true"))
}

/// A character with an active cross-conversation memory cutoff.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterMemoryForgotten {
    pub character_id: String,
    /// RFC 3339 timestamp — nothing before it is recalled.
    pub forgotten_at: String,
}

/// Lists every character with an active memory cutoff (Library badges).
#[tauri::command]
pub async fn list_character_memory_forgotten(
    db: State<'_, DbWorker>,
) -> Result<Vec<CharacterMemoryForgotten>, String> {
    let prefix = embeddings::memory_forgotten_key("");
    let prefix = &prefix[..prefix.len() - 1]; // trailing ':' only
    let rows = db_call(&db.sender(), |reply| DbCommand::LoadAppSettingsByPrefix {
        prefix: prefix.to_string(),
        reply,
    })
    .await
    .map_err(|e| e.to_string())?;

    let key_len = prefix.len() + 1;
    Ok(rows
        .into_iter()
        .filter_map(|(key, value)| {
            let forgotten_at = value.trim().to_string();
            if forgotten_at.is_empty() {
                return None; // Cleared cutoff.
            }
            Some(CharacterMemoryForgotten {
                character_id: key[key_len..].to_string(),
                forgotten_at,
            })
        })
        .collect())
}
