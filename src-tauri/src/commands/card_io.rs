use crate::domain::character::CharacterCardV2;
use crate::error::AppError;
use crate::service::card_io;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Import a character card from a file (PNG with embedded chara tEXt chunk or JSON).
/// Returns the parsed CharacterCardV2 with a new UUID and normalized fields.
#[tauri::command]
pub async fn import_card(app: AppHandle, file_path: String) -> Result<CharacterCardV2, AppError> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(AppError::Validation(format!(
            "File does not exist: {}",
            file_path
        )));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (character, avatar_bytes) = if ext == "png" {
        let png_data = std::fs::read(&path)?;
        card_io::import_from_png(&png_data)?
    } else if ext == "json" {
        let json_str = std::fs::read_to_string(&path)?;
        let character = card_io::import_from_json(&json_str)?;
        (character, None)
    } else {
        return Err(AppError::Validation(format!(
            "Unsupported file format: .{}. Supported formats: .png, .json",
            ext
        )));
    };

    // Save avatar if present (PNG import provides the image as avatar)
    let mut character = character;
    if let Some(avatar_data) = avatar_bytes {
        let app_data = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::other(e.to_string()))?;

        let avatars_dir = app_data.join("avatars");
        std::fs::create_dir_all(&avatars_dir)?;

        let avatar_path = avatars_dir.join(format!("{}.png", character.id));
        std::fs::write(&avatar_path, &avatar_data)?;

        character.avatar_path = Some(
            avatar_path
                .to_str()
                .ok_or_else(|| AppError::validation("Failed to convert avatar path to string"))?
                .to_string(),
        );
    }

    Ok(character)
}

/// Export a character card as a PNG file with embedded chara tEXt chunk.
/// Reads the character's avatar (if any) and generates the PNG at the specified save path.
///
/// Security: Validates that the save path has a `.png` extension and resolves
/// any `..` or symlink components via canonicalization of the parent directory.
#[tauri::command]
pub async fn export_card(
    app: AppHandle,
    character_id: String,
    save_path: String,
) -> Result<(), AppError> {
    let save_path = PathBuf::from(&save_path);

    // Security: validate file extension to prevent writing arbitrary file types
    match save_path.extension().and_then(|e| e.to_str()) {
        Some("png") => {}
        Some(other) => {
            return Err(AppError::Validation(format!(
                "Invalid file extension: .{}. Only .png is allowed for card export",
                other
            )));
        }
        None => {
            return Err(AppError::Validation(
                "Save path must have a .png extension".to_string(),
            ));
        }
    }

    // Security: canonicalize parent directory to resolve `..` and symlinks
    let parent = save_path
        .parent()
        .ok_or_else(|| AppError::validation("Save path has no parent directory"))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| AppError::Validation(format!("Invalid save directory: {}", e)))?;
    let safe_save_path = canonical_parent.join(save_path.file_name().unwrap_or_default());

    // Reject paths that resolve to sensitive system directories
    let path_str = canonical_parent.to_string_lossy();
    if path_str.starts_with("/etc")
        || path_str.starts_with("/usr")
        || path_str.starts_with("/System")
    {
        return Err(AppError::PathTraversal(canonical_parent));
    }

    // Load the character from the database via DbWorker
    let db = app.state::<crate::db::DbWorker>();
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(crate::db::DbCommand::LoadCharacters { reply: reply_tx })
        .map_err(AppError::DbWorkerUnavailable)?;
    let characters = reply_rx
        .await
        .map_err(|e| AppError::DbWorkerUnavailable(format!("DB worker dropped: {}", e)))?
        .map_err(|e| AppError::Database(e.to_string()))?;

    let character = characters
        .into_iter()
        .find(|c| c.id == character_id)
        .ok_or_else(|| AppError::not_found(format!("Character not found: {}", character_id)))?;

    // Read avatar bytes if available
    let avatar_bytes = if let Some(ref avatar_path) = character.avatar_path {
        let path = PathBuf::from(avatar_path);
        if path.exists() {
            Some(std::fs::read(&path)?)
        } else {
            None
        }
    } else {
        None
    };

    // Generate PNG with embedded card data
    let png_data = card_io::export_to_png(&character, avatar_bytes.as_deref())?;

    // Write to the validated save path
    std::fs::write(&safe_save_path, &png_data)?;

    Ok(())
}
