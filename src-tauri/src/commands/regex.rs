use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::regex_script::RegexScript;

#[tauri::command]
pub async fn get_regex_scripts(
    db: State<'_, DbWorker>,
    character_id: String,
) -> Result<Vec<RegexScript>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadRegexScripts {
        character_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_regex_script(
    db: State<'_, DbWorker>,
    script: RegexScript,
) -> Result<RegexScript, String> {
    // Validate pattern before saving (REGX-05)
    crate::service::regex::validate_regex_flags(&script.flags).map_err(|e| e.to_string())?;
    crate::service::regex::validate_regex_pattern(&script.find_pattern, &script.flags)
        .map_err(|e| e.to_string())?;

    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveRegexScript {
        script,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_regex_script(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteRegexScript { id, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_single_regex(
    pattern: String,
    replace_pattern: String,
    flags: String,
    sample_text: String,
) -> Result<serde_json::Value, String> {
    // Validate pattern
    crate::service::regex::validate_regex_flags(&flags).map_err(|e| e.to_string())?;
    let re = crate::service::regex::validate_regex_pattern(&pattern, &flags)
        .map_err(|e| e.to_string())?;

    let is_global = flags.contains('g');
    if is_global {
        let result = re
            .replace_all(&sample_text, replace_pattern.as_str())
            .to_string();
        let match_count = re.find_iter(&sample_text).count();
        Ok(serde_json::json!({
            "output": result,
            "match_count": match_count,
        }))
    } else {
        let result = re
            .replace(&sample_text, replace_pattern.as_str())
            .to_string();
        let match_count = if re.is_match(&sample_text) { 1 } else { 0 };
        Ok(serde_json::json!({
            "output": result,
            "match_count": match_count,
        }))
    }
}

#[tauri::command]
pub async fn validate_regex_pattern(
    pattern: String,
    flags: String,
) -> Result<(), String> {
    crate::service::regex::validate_regex_flags(&flags).map_err(|e| e.to_string())?;
    crate::service::regex::validate_regex_pattern(&pattern, &flags).map_err(|e| e.to_string())?;
    Ok(())
}
