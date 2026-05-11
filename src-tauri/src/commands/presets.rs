use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::GenerationPreset;

#[tauri::command]
pub async fn get_presets(db: State<'_, DbWorker>) -> Result<Vec<GenerationPreset>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadPresets { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
