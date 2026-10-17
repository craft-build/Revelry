mod ai;
mod commands;
mod db;
mod domain;
mod error;
mod secrets;
mod service;

use db::DbWorker;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = match dirs::data_dir() {
        Some(d) => d,
        None => {
            eprintln!("FATAL: Could not determine platform data directory");
            std::process::exit(1);
        }
    };
    let db_path = data_dir.join("com.revelry.desktop").join("revelry.db");

    let db_worker = DbWorker::start(db_path.clone()).unwrap_or_else(|e| {
        eprintln!(
            "FATAL: Failed to start database worker at {:?}: {}",
            db_path, e
        );
        std::process::exit(1);
    });

    // Keyring 4.1+ initializes lazily on first Entry::new(); no explicit
    // store init/release is needed. Keyring availability is checked per-call
    // via KeyringStore::is_available().

    let mut builder = tauri::Builder::default();
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_webdriver_automation::init());
    }
    builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(db_worker)
        .manage(commands::chat::CancelState {
            token: std::sync::Mutex::new(None),
        })
        .manage(commands::generate::GenerateCancelState {
            token: std::sync::Mutex::new(None),
        })
        .manage(ai::embeddings::EmbeddingRuntime::new())
        .invoke_handler(tauri::generate_handler![
            commands::characters::get_characters,
            commands::characters::save_character,
            commands::characters::delete_character,
            commands::personas::get_personas,
            commands::personas::save_persona,
            commands::personas::delete_persona,
            commands::conversations::get_conversations,
            commands::conversations::start_conversation,
            commands::conversations::delete_conversation,
            commands::conversations::rename_conversation,
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
            commands::lorebooks::import_lorebook,
            commands::avatars::upload_avatar,
            commands::avatars::delete_avatar,
            commands::keywords::test_lorebook_keywords,
            commands::providers::get_providers,
            commands::providers::save_provider,
            commands::providers::delete_provider,
            commands::providers::save_api_key,
            commands::providers::delete_api_key,
            commands::providers::check_keyring_available,
            commands::providers::update_conversation_provider,
            commands::providers::list_provider_models,
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
            commands::embeddings::get_embedding_config,
            commands::embeddings::set_embedding_config,
            commands::embeddings::get_embedding_status,
            commands::embeddings::test_embedding,
            commands::embeddings::rebuild_embedding_index,
            commands::keywords::test_semantic_activation,
            commands::generate::generate_character,
            commands::generate::stop_character_generation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
