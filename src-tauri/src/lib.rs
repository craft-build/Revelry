mod commands;
mod db;
mod domain;
mod secrets;

use db::DbWorker;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs::data_dir()
        .expect("Could not determine platform data directory")
        .join("com.revelry.desktop")
        .join("revelry.db");

    let db_worker = DbWorker::start(db_path.clone()).unwrap_or_else(|e| {
        eprintln!(
            "FATAL: Failed to start database worker at {:?}: {}",
            db_path, e
        );
        std::process::exit(1);
    });

    // Initialize keyring store (v4 requires explicit init before Entry::new())
    if let Err(e) = keyring::use_native_store(true) {
        tracing::warn!("Failed to initialize keyring: {}. Keyring features will be disabled.", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(db_worker)
        .invoke_handler(tauri::generate_handler![
            commands::characters::get_characters,
            commands::characters::save_character,
            commands::characters::delete_character,
            commands::characters::search_characters,
            commands::personas::get_personas,
            commands::personas::save_persona,
            commands::personas::delete_persona,
            commands::personas::search_personas,
            commands::conversations::get_conversations,
            commands::conversations::start_conversation,
            commands::conversations::delete_conversation,
            commands::conversations::search_conversations,
            commands::conversations::get_messages,
            commands::conversations::save_message,
            commands::conversations::update_conversation_persona,
            commands::presets::get_presets,
            commands::lorebooks::get_lorebooks,
            commands::lorebooks::get_lorebook,
            commands::lorebooks::save_lorebook,
            commands::lorebooks::delete_lorebook,
            commands::lorebooks::get_lorebook_entries,
            commands::lorebooks::save_lorebook_entry,
            commands::lorebooks::delete_lorebook_entry,
            commands::lorebooks::assign_lorebook_to_character,
            commands::lorebooks::remove_lorebook_from_character,
            commands::avatars::upload_avatar,
            commands::avatars::delete_avatar,
            commands::keywords::test_lorebook_keywords,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Release keyring store on shutdown (v4 requires explicit cleanup)
    keyring::release_store();
}
