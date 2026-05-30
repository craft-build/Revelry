use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::ai::{AiService, GenerateCharacterChunk};
use crate::db::{DbCommand, DbWorker};

use crate::error::AppError;

/// Managed state holding the cancellation token for character generation.
/// Only one generation can be active at a time.
pub struct GenerateCancelState {
    pub token: Mutex<Option<CancellationToken>>,
}

#[tauri::command]
pub async fn generate_character(
    db: State<'_, DbWorker>,
    cancel_state: State<'_, GenerateCancelState>,
    channel: Channel<GenerateCharacterChunk>,
    idea: String,
    provider_id: String,
    model_name: String,
) -> Result<(), AppError> {
    // 1. Cancel any existing generation stream
    {
        let mut token_guard = cancel_state.token.lock()?;
        if let Some(existing) = token_guard.take() {
            existing.cancel();
        }
    }

    // 2. Load the provider
    let provider = {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        db.send(DbCommand::LoadProviders { reply: reply_tx })
            .map_err(|e| AppError::DbWorkerUnavailable(e.to_string()))?;
        let providers = reply_rx
            .await
            .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
            .map_err(|e| AppError::Database(e.to_string()))?;
        providers
            .into_iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| AppError::not_found("Provider not found"))?
    };

    // 3. Load the API key
    let provider_id_for_key = provider.id.clone();
    let provider_type = provider.provider_type.clone();
    let api_key_result = tokio::task::spawn_blocking(move || {
        if !provider_type.requires_api_key() {
            return Ok(String::new());
        }
        crate::secrets::KeyringStore::load_key(&provider_id_for_key)
    })
    .await
    .map_err(|e| AppError::keyring(format!("Keyring task failed: {}", e)))?;

    let api_key = match api_key_result {
        Ok(key) => key,
        Err(_) => {
            let _ = channel.send(GenerateCharacterChunk::Error {
                error: "No API key set for this provider. Add your key in Settings.".to_string(),
                error_type: "no_api_key".to_string(),
            });
            return Ok(());
        }
    };

    // 4. Override model_name with the user's selection
    let mut provider_config = provider.clone();
    if !model_name.is_empty() {
        provider_config.model_name = model_name;
    }

    // 5. Resolve generation params from provider defaults
    let params = AiService::from_provider_config(&provider_config);

    // 6. Start streaming
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = cancel_state.token.lock()?;
        *token_guard = Some(cancel_token.clone());
    }

    let provider_config = provider_config.clone();
    tokio::spawn(async move {
        AiService::stream_generate_character(
            &provider_config,
            &idea,
            &api_key,
            params,
            channel,
            cancel_token,
        );
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_character_generation(
    cancel_state: State<'_, GenerateCancelState>,
) -> Result<(), AppError> {
    let token = {
        let mut token_guard = cancel_state.token.lock()?;
        token_guard.take()
    };

    if let Some(token) = token {
        token.cancel();
    }

    Ok(())
}
