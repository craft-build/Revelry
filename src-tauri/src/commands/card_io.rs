use crate::domain::character::CharacterCardV2;
use crate::service::card_io;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Import a character card from a file (PNG with embedded chara tEXt chunk or JSON).
/// Returns the parsed CharacterCardV2 with a new UUID and normalized fields.
#[tauri::command]
pub async fn import_card(
    app: AppHandle,
    file_path: String,
) -> Result<CharacterCardV2, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (character, avatar_bytes) = if ext == "png" {
        let png_data = std::fs::read(&path)
            .map_err(|e| format!("Failed to read PNG file: {}", e))?;
        card_io::import_from_png(&png_data)
            .map_err(|e| format!("{}", e))?
    } else if ext == "json" {
        let json_str = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read JSON file: {}", e))?;
        let character = card_io::import_from_json(&json_str)
            .map_err(|e| format!("{}", e))?;
        (character, None)
    } else {
        return Err(format!(
            "Unsupported file format: .{}. Supported formats: .png, .json",
            ext
        ));
    };

    // Save avatar if present (PNG import provides the image as avatar)
    let mut character = character;
    if let Some(avatar_data) = avatar_bytes {
        let app_data = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        let avatars_dir = app_data.join("avatars");
        std::fs::create_dir_all(&avatars_dir)
            .map_err(|e| format!("Failed to create avatars dir: {}", e))?;

        let avatar_path = avatars_dir.join(format!("{}.png", character.id));
        std::fs::write(&avatar_path, &avatar_data)
            .map_err(|e| format!("Failed to save avatar: {}", e))?;

        character.avatar_path = Some(
            avatar_path
                .to_str()
                .ok_or("Failed to convert avatar path to string")?
                .to_string(),
        );
    }

    Ok(character)
}

/// Export a character card as a PNG file with embedded chara tEXt chunk.
/// Reads the character's avatar (if any) and generates the PNG at the specified save path.
#[tauri::command]
pub async fn export_card(
    app: AppHandle,
    character_id: String,
    save_path: String,
) -> Result<(), String> {
    // Load the character from the database via DbWorker
    let db = app.state::<crate::db::DbWorker>();
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(crate::db::DbCommand::LoadCharacters { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    let characters = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;

    let character = characters
        .into_iter()
        .find(|c| c.id == character_id)
        .ok_or_else(|| format!("Character not found: {}", character_id))?;

    // Read avatar bytes if available
    let avatar_bytes = if let Some(ref avatar_path) = character.avatar_path {
        let path = PathBuf::from(avatar_path);
        if path.exists() {
            Some(
                std::fs::read(&path)
                    .map_err(|e| format!("Failed to read avatar: {}", e))?,
            )
        } else {
            None
        }
    } else {
        None
    };

    // Generate PNG with embedded card data
    let png_data = card_io::export_to_png(&character, avatar_bytes.as_deref())
        .map_err(|e| format!("Failed to export card: {}", e))?;

    // Write to the save path
    std::fs::write(&save_path, &png_data)
        .map_err(|e| format!("Failed to write PNG: {}", e))?;

    Ok(())
}
