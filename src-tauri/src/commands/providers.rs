use serde::{Deserialize, Serialize};
use tauri::State;

use crate::ai::AiService;
use crate::db::{DbCommand, DbWorker};
use crate::domain::provider::{ProviderConfig, ProviderModelEntry, ProviderType};

/// Provider configuration with key status.
/// Returned by get_providers so the frontend knows if a key is set
/// without ever receiving the actual key value (security invariant).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderWithKeyStatus {
    #[serde(flatten)]
    pub provider: ProviderConfig,
    pub has_api_key: bool,
}

#[tauri::command]
pub async fn get_providers(db: State<'_, DbWorker>) -> Result<Vec<ProviderWithKeyStatus>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadProviders { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    let providers = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;

    // Batch-check has_api_key for each provider via spawn_blocking
    let mut result = Vec::with_capacity(providers.len());
    for provider in providers {
        let pid = provider.id.clone();
        let has_key = tokio::task::spawn_blocking(move || {
            crate::secrets::KeyringStore::load_key(&pid).is_ok()
        })
        .await
        .unwrap_or(false);

        result.push(ProviderWithKeyStatus {
            provider,
            has_api_key: has_key,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn save_provider(
    db: State<'_, DbWorker>,
    provider: ProviderConfig,
) -> Result<ProviderConfig, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveProvider {
        provider,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_provider(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteProvider {
        id: id.clone(),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;

    // Best-effort keyring cleanup (Pitfall 5 — don't orphan keychain entries)
    let keyring_id = id;
    let _ =
        tokio::task::spawn_blocking(move || crate::secrets::KeyringStore::delete_key(&keyring_id))
            .await;

    Ok(())
}

#[tauri::command]
pub async fn save_api_key(provider_id: String, api_key: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        crate::secrets::KeyringStore::save_key(&provider_id, &api_key)
    })
    .await
    .map_err(|e| format!("Keyring task failed: {}", e))?
    .map_err(|e| format!("Failed to save API key: {}", e))
}

#[tauri::command]
pub async fn has_api_key(provider_id: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        match crate::secrets::KeyringStore::load_key(&provider_id) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })
    .await
    .map_err(|e| format!("Keyring check failed: {}", e))?
}

#[tauri::command]
pub async fn delete_api_key(provider_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || crate::secrets::KeyringStore::delete_key(&provider_id))
        .await
        .map_err(|e| format!("Keyring task failed: {}", e))?
        .map_err(|e| format!("Failed to delete API key: {}", e))
}

#[tauri::command]
pub async fn check_keyring_available() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| Ok(crate::secrets::KeyringStore::is_available()))
        .await
        .map_err(|e| format!("Keyring check failed: {}", e))?
}

#[tauri::command]
pub async fn update_conversation_provider(
    db: State<'_, DbWorker>,
    id: String,
    provider_id: Option<String>,
    model_name: Option<String>,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::UpdateConversationProvider {
        conversation_id: id,
        provider_id,
        model_name,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

/// Detect available models from a provider's Models API.
///
/// Resolves the API key by preferring a passed `api_key` (create flow, where
/// the key isn't in the keyring yet) and falling back to the keyring via
/// `provider_id` (edit flow). Ollama is allowed with an empty key.
///
/// Returns a sorted, deduped list of `ProviderModelEntry` values.
#[tauri::command]
pub async fn list_provider_models(
    provider_type: ProviderType,
    base_url: Option<String>,
    provider_id: Option<String>,
    api_key: Option<String>,
) -> Result<Vec<ProviderModelEntry>, String> {
    // Resolve the API key: prefer an explicitly passed key (create mode),
    // otherwise load from the keyring (edit mode). Ollama needs no key.
    let key = if provider_type == ProviderType::Ollama {
        String::new()
    } else if let Some(k) = api_key {
        if k.trim().is_empty() {
            return Err(
                "An API key is required to detect models. Add a key and try again.".to_string(),
            );
        }
        k
    } else if let Some(pid) = provider_id {
        let pid_clone = pid.clone();
        tokio::task::spawn_blocking(move || crate::secrets::KeyringStore::load_key(&pid_clone))
            .await
            .map_err(|e| format!("Keyring task failed: {}", e))?
            .map_err(|e| {
                format!(
                    "No API key found for this provider. Add a key and try again. ({})",
                    e
                )
            })?
    } else {
        return Err(
            "An API key is required to detect models. Add a key and try again.".to_string(),
        );
    };

    AiService::list_provider_models(&provider_type, base_url.as_deref(), &key).await
}
