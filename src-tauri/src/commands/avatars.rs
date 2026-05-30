use crate::error::AppError;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const AVATAR_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

/// Uploads an avatar image to the app's data directory.
/// Copies the source file to `{app_data}/avatars/{id}.{ext}` and returns the absolute path.
/// The absolute path is suitable for Tauri's `convertFileSrc()` on the frontend.
#[tauri::command]
pub async fn upload_avatar(
    app: AppHandle,
    id: String,
    source_path: String,
) -> Result<String, AppError> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err(AppError::Validation(format!(
            "Source file does not exist: {}",
            source_path
        )));
    }

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    if !AVATAR_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::Validation(format!(
            "Unsupported avatar format: {}. Supported: {:?}",
            ext, AVATAR_EXTENSIONS
        )));
    }

    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::other(e.to_string()))?;

    let avatars_dir = app_data.join("avatars");
    std::fs::create_dir_all(&avatars_dir)?;

    let dest_filename = format!("{}.{}", id, ext);
    let dest_path = avatars_dir.join(&dest_filename);

    // Remove old avatar files for this id (different extensions)
    if let Ok(entries) = std::fs::read_dir(&avatars_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(&format!("{}.", id)) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    std::fs::copy(&source, &dest_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to copy avatar: {}", e),
        ))
    })?;

    // Return absolute path for storage in DB — Tauri's convertFileSrc() needs absolute paths
    dest_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::validation("Failed to convert avatar path to string"))
}

/// Deletes an avatar file from the app's data directory.
/// Accepts both absolute paths (from upload_avatar) and legacy relative paths.
#[tauri::command]
pub async fn delete_avatar(app: AppHandle, path: String) -> Result<(), AppError> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::other(e.to_string()))?;

    let full_path = if PathBuf::from(&path).is_absolute() {
        PathBuf::from(&path)
    } else {
        app_data.join(&path)
    };

    // Security: ensure the path is within the app data directory
    if !full_path.starts_with(&app_data) {
        return Err(AppError::PathTraversal(full_path));
    }

    if full_path.exists() {
        std::fs::remove_file(&full_path)?;
    }

    Ok(())
}
