use crate::db::{DbCommand, DbWorker};
use tauri::State;
use tokio::sync::oneshot;

#[tauri::command]
pub async fn get_app_setting(
    key: String,
    db: State<'_, DbWorker>,
) -> Result<Option<String>, String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::LoadAppSetting {
        key,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_app_setting(
    key: String,
    value: String,
    db: State<'_, DbWorker>,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::SaveAppSetting {
        key,
        value,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
