mod ai;
mod commands;
mod db;
mod domain;
mod secrets;
mod service;

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
        .manage(commands::chat::CancelState {
            token: std::sync::Mutex::new(None),
        })
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
            commands::conversations::delete_message,
            commands::conversations::update_conversation_persona,
            commands::conversations::fork_conversation,
            commands::conversations::get_branches_for_message,
            commands::conversations::get_branches_for_conversation,
            commands::conversations::delete_branch,
            commands::conversations::search_messages,
            commands::presets::get_presets,
            commands::presets::save_preset,
            commands::presets::delete_preset,
            commands::conversations::update_conversation_generation,
            commands::lorebooks::get_lorebooks,
            commands::lorebooks::get_lorebook,
            commands::lorebooks::save_lorebook,
            commands::lorebooks::delete_lorebook,
            commands::lorebooks::get_lorebook_entries,
            commands::lorebooks::save_lorebook_entry,
            commands::lorebooks::delete_lorebook_entry,
            commands::lorebooks::assign_lorebook_to_character,
            commands::lorebooks::remove_lorebook_from_character,
            commands::lorebooks::get_character_lorebooks,
            commands::lorebooks::assign_lorebook_to_conversation,
            commands::lorebooks::remove_lorebook_from_conversation,
            commands::lorebooks::get_conversation_lorebooks,
            commands::avatars::upload_avatar,
            commands::avatars::delete_avatar,
            commands::keywords::test_lorebook_keywords,
            commands::providers::get_providers,
            commands::providers::save_provider,
            commands::providers::delete_provider,
            commands::providers::save_api_key,
            commands::providers::has_api_key,
            commands::providers::delete_api_key,
            commands::providers::check_keyring_available,
            commands::providers::update_conversation_provider,
            commands::chat::start_chat,
            commands::chat::stop_generation,
            commands::chat::regenerate,
            commands::chat::edit_and_resend,
            commands::card_io::import_card,
            commands::card_io::export_card,
            commands::regex::get_regex_scripts,
            commands::regex::save_regex_script,
            commands::regex::delete_regex_script,
            commands::regex::apply_single_regex,
            commands::regex::validate_regex_pattern,
            commands::app_settings::get_app_setting,
            commands::app_settings::set_app_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Release keyring store on shutdown (v4 requires explicit cleanup)
    keyring::release_store();
}
