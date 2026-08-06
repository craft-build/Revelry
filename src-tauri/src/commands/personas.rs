use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::Persona;

#[tauri::command]
pub async fn get_personas(db: State<'_, DbWorker>) -> Result<Vec<Persona>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadPersonas { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_persona(db: State<'_, DbWorker>, persona: Persona) -> Result<Persona, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SavePersona {
        persona,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_persona(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeletePersona {
        id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
