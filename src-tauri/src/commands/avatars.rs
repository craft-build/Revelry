use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Supported avatar image extensions.
const AVATAR_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];

/// Uploads an avatar image to the app's data directory.
/// Copies the source file to `{app_data}/avatars/{id}.{ext}` and returns the relative path.
#[tauri::command]
pub async fn upload_avatar(
    app: AppHandle,
    id: String,
    source_path: String,
) -> Result<String, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err(format!("Source file does not exist: {}", source_path));
    }

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    if !AVATAR_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!(
            "Unsupported avatar format: {}. Supported: {:?}",
            ext, AVATAR_EXTENSIONS
        ));
    }

    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let avatars_dir = app_data.join("avatars");
    std::fs::create_dir_all(&avatars_dir)
        .map_err(|e| format!("Failed to create avatars dir: {}", e))?;

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

    std::fs::copy(&source, &dest_path).map_err(|e| format!("Failed to copy avatar: {}", e))?;

    // Return relative path for storage in DB
    Ok(format!("avatars/{}", dest_filename))
}

/// Deletes an avatar file from the app's data directory.
#[tauri::command]
pub async fn delete_avatar(app: AppHandle, path: String) -> Result<(), String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let full_path = app_data.join(&path);

    // Security: ensure the path is within the app data directory
    if !full_path.starts_with(&app_data) {
        return Err("Avatar path is outside app data directory".to_string());
    }

    if full_path.exists() {
        std::fs::remove_file(&full_path).map_err(|e| format!("Failed to delete avatar: {}", e))?;
    }

    Ok(())
}
