pub mod characters;
pub mod conversations;
pub mod generation;
pub mod lorebook;
pub mod migrations;
pub mod models;
pub mod personas;
pub mod providers;
pub mod regex_scripts;

use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message};
use crate::domain::generation::GenerationPreset;
use crate::domain::lorebook::{Lorebook, LorebookEntry};
use crate::domain::persona::Persona;
use crate::domain::provider::ProviderConfig;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::mpsc;
use tokio::sync::oneshot;

/// Commands sent to the DbWorker background thread.
/// Each variant carries a `reply` oneshot channel so callers can await results.
#[derive(Debug)]
pub enum DbCommand {
    // Character operations
    LoadCharacters {
        reply: oneshot::Sender<anyhow::Result<Vec<CharacterCardV2>>>,
    },
    SaveCharacter {
        character: CharacterCardV2,
        reply: oneshot::Sender<anyhow::Result<CharacterCardV2>>,
    },
    DeleteCharacter {
        id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    SearchCharacters {
        query: String,
        reply: oneshot::Sender<anyhow::Result<Vec<CharacterCardV2>>>,
    },

    // Persona operations
    LoadPersonas {
        reply: oneshot::Sender<anyhow::Result<Vec<Persona>>>,
    },
    SavePersona {
        persona: Persona,
        reply: oneshot::Sender<anyhow::Result<Persona>>,
    },
    DeletePersona {
        id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    SearchPersonas {
        query: String,
        reply: oneshot::Sender<anyhow::Result<Vec<Persona>>>,
    },

    // Conversation operations
    LoadConversations {
        reply: oneshot::Sender<anyhow::Result<Vec<Conversation>>>,
    },
    StartConversation {
        character_id: String,
        reply: oneshot::Sender<anyhow::Result<Conversation>>,
    },
    DeleteConversation {
        id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    SearchConversations {
        query: String,
        reply: oneshot::Sender<anyhow::Result<Vec<Conversation>>>,
    },
    UpdateConversationPersona {
        conversation_id: String,
        persona_id: Option<String>,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    UpdateConversationProvider {
        conversation_id: String,
        provider_id: Option<String>,
        model_name: Option<String>,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },

    // Message operations
    LoadMessages {
        conversation_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<Message>>>,
    },
    SaveMessage {
        message: Message,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },

    // Preset operations
    LoadPresets {
        reply: oneshot::Sender<anyhow::Result<Vec<GenerationPreset>>>,
    },

    // Lorebook operations
    LoadLorebooks {
        reply: oneshot::Sender<anyhow::Result<Vec<Lorebook>>>,
    },
    LoadLorebook {
        id: String,
        reply: oneshot::Sender<anyhow::Result<Option<Lorebook>>>,
    },
    SaveLorebook {
        lorebook: Lorebook,
        reply: oneshot::Sender<anyhow::Result<Lorebook>>,
    },
    DeleteLorebook {
        id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },

    // Lorebook entry operations
    LoadLorebookEntries {
        lorebook_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<LorebookEntry>>>,
    },
    SaveLorebookEntry {
        entry: LorebookEntry,
        reply: oneshot::Sender<anyhow::Result<LorebookEntry>>,
    },
    DeleteLorebookEntry {
        id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },

    // Character-lorebook junction
    AssignLorebookToCharacter {
        character_id: String,
        lorebook_id: String,
        sort_order: i32,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    RemoveLorebookFromCharacter {
        character_id: String,
        lorebook_id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    LoadCharacterLorebooks {
        character_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<Lorebook>>>,
    },

    // Provider operations
    LoadProviders {
        reply: oneshot::Sender<anyhow::Result<Vec<ProviderConfig>>>,
    },
    SaveProvider {
        provider: ProviderConfig,
        reply: oneshot::Sender<anyhow::Result<ProviderConfig>>,
    },
    DeleteProvider {
        id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
}

/// Database worker running on a dedicated background thread.
/// Owns the SQLite connection and processes all DB operations serially.
/// This prevents main-thread blocking per PITFALLS.md Pitfall 3.
pub struct DbWorker {
    sender: mpsc::Sender<DbCommand>,
}

impl DbWorker {
    /// Starts the DbWorker background thread.
    /// Creates the database directory if it doesn't exist, opens the connection,
    /// runs migrations, and seeds built-in presets.
    pub fn start(db_path: PathBuf) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel::<DbCommand>();

        std::thread::spawn(move || {
            // Create parent directory if needed
            if let Some(parent) = db_path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        tracing::error!("Failed to create database directory {:?}: {}", parent, e);
                        return;
                    }
                }
            }

            // Open database connection
            let mut conn = match Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to open database at {:?}: {}", db_path, e);
                    return;
                }
            };

            // Run migrations
            if let Err(e) = migrations::run_migrations(&mut conn) {
                tracing::error!("Failed to run migrations: {}", e);
                return;
            }

            // Seed built-in presets (idempotent)
            if let Err(e) = generation::seed_builtin_presets(&conn) {
                tracing::error!("Failed to seed built-in presets: {}", e);
                return;
            }

            tracing::info!("Database initialized at {:?}", db_path);

            // Process commands serially on the background thread
            while let Ok(cmd) = receiver.recv() {
                DbWorker::handle_command(&conn, cmd);
            }

            tracing::info!("DbWorker thread shutting down");
        });

        Ok(Self { sender })
    }

    /// Sends a command to the background thread.
    pub fn send(&self, cmd: DbCommand) -> Result<(), String> {
        self.sender
            .send(cmd)
            .map_err(|e| format!("Failed to send DB command: {}", e))
    }

    /// Handles a single DbCommand on the background thread.
    /// Calls real CRUD functions from the db sub-modules and sends
    /// the result back through the oneshot reply channel.
    fn handle_command(conn: &Connection, cmd: DbCommand) {
        match cmd {
            DbCommand::LoadCharacters { reply } => {
                let result = characters::load_characters(conn);
                let _ = reply.send(result);
            }
            DbCommand::SaveCharacter { character, reply } => {
                let result = if characters::load_character(conn, &character.id)
                    .unwrap_or(None)
                    .is_some()
                {
                    characters::update_character(conn, &character).map(|_| character.clone())
                } else {
                    characters::insert_character(conn, &character).map(|_| character.clone())
                };
                let _ = reply.send(result);
            }
            DbCommand::DeleteCharacter { id, reply } => {
                let result = characters::delete_character(conn, &id);
                let _ = reply.send(result);
            }
            DbCommand::SearchCharacters { query, reply } => {
                let result = characters::search_characters(conn, &query);
                let _ = reply.send(result);
            }
            DbCommand::LoadPersonas { reply } => {
                let result = personas::load_personas(conn);
                let _ = reply.send(result);
            }
            DbCommand::SavePersona { persona, reply } => {
                let result = if personas::load_persona(conn, &persona.id)
                    .unwrap_or(None)
                    .is_some()
                {
                    personas::update_persona(conn, &persona).map(|_| persona.clone())
                } else {
                    personas::insert_persona(conn, &persona).map(|_| persona.clone())
                };
                let _ = reply.send(result);
            }
            DbCommand::DeletePersona { id, reply } => {
                let result = personas::delete_persona(conn, &id);
                let _ = reply.send(result);
            }
            DbCommand::SearchPersonas { query, reply } => {
                let result = personas::search_personas(conn, &query);
                let _ = reply.send(result);
            }
            DbCommand::LoadConversations { reply } => {
                let result = conversations::load_conversations(conn);
                let _ = reply.send(result);
            }
            DbCommand::StartConversation {
                character_id,
                reply,
            } => {
                let result = conversations::start_conversation(conn, &character_id);
                let _ = reply.send(result);
            }
            DbCommand::DeleteConversation { id, reply } => {
                let result = conversations::delete_conversation(conn, &id);
                let _ = reply.send(result);
            }
            DbCommand::SearchConversations { query, reply } => {
                let result = conversations::search_conversations(conn, &query);
                let _ = reply.send(result);
            }
            DbCommand::UpdateConversationPersona {
                conversation_id,
                persona_id,
                reply,
            } => {
                let result = conversations::update_conversation_persona(
                    conn,
                    &conversation_id,
                    persona_id.as_deref(),
                );
                let _ = reply.send(result);
            }
            DbCommand::UpdateConversationProvider {
                conversation_id,
                provider_id,
                model_name,
                reply,
            } => {
                let result = conversations::update_conversation_provider(
                    conn,
                    &conversation_id,
                    provider_id.as_deref(),
                    model_name.as_deref(),
                );
                let _ = reply.send(result);
            }
            DbCommand::LoadMessages {
                conversation_id,
                reply,
            } => {
                let result = conversations::load_messages(conn, &conversation_id);
                let _ = reply.send(result);
            }
            DbCommand::SaveMessage { message, reply } => {
                let result = conversations::save_message(conn, &message);
                let _ = reply.send(result);
            }
            DbCommand::LoadPresets { reply } => {
                let result = generation::load_presets(conn);
                let _ = reply.send(result);
            }

            // Lorebook operations
            DbCommand::LoadLorebooks { reply } => {
                let result = lorebook::load_lorebooks(conn);
                let _ = reply.send(result);
            }
            DbCommand::LoadLorebook { id, reply } => {
                let result = lorebook::load_lorebook(conn, &id);
                let _ = reply.send(result);
            }
            DbCommand::SaveLorebook { lorebook, reply } => {
                let result = if lorebook::load_lorebook(conn, &lorebook.id)
                    .unwrap_or(None)
                    .is_some()
                {
                    lorebook::update_lorebook(conn, &lorebook).map(|_| lorebook.clone())
                } else {
                    lorebook::insert_lorebook(conn, &lorebook).map(|_| lorebook.clone())
                };
                let _ = reply.send(result);
            }
            DbCommand::DeleteLorebook { id, reply } => {
                let result = lorebook::delete_lorebook(conn, &id);
                let _ = reply.send(result);
            }

            // Lorebook entry operations
            DbCommand::LoadLorebookEntries { lorebook_id, reply } => {
                let result = lorebook::load_lorebook_entries(conn, &lorebook_id);
                let _ = reply.send(result);
            }
            DbCommand::SaveLorebookEntry { entry, reply } => {
                let result = if lorebook::load_lorebook_entries(conn, &entry.lorebook_id)
                    .unwrap_or_default()
                    .iter()
                    .any(|e| e.id == entry.id)
                {
                    lorebook::update_lorebook_entry(conn, &entry).map(|_| entry.clone())
                } else {
                    lorebook::insert_lorebook_entry(conn, &entry).map(|_| entry.clone())
                };
                let _ = reply.send(result);
            }
            DbCommand::DeleteLorebookEntry { id, reply } => {
                let result = lorebook::delete_lorebook_entry(conn, &id);
                let _ = reply.send(result);
            }

            // Character-lorebook junction
            DbCommand::AssignLorebookToCharacter {
                character_id,
                lorebook_id,
                sort_order,
                reply,
            } => {
                let result = lorebook::assign_lorebook_to_character(
                    conn,
                    &character_id,
                    &lorebook_id,
                    sort_order,
                );
                let _ = reply.send(result);
            }
            DbCommand::RemoveLorebookFromCharacter {
                character_id,
                lorebook_id,
                reply,
            } => {
                let result =
                    lorebook::remove_lorebook_from_character(conn, &character_id, &lorebook_id);
                let _ = reply.send(result);
            }
            DbCommand::LoadCharacterLorebooks {
                character_id,
                reply,
            } => {
                let result = lorebook::load_lorebooks_for_character(conn, &character_id);
                let _ = reply.send(result);
            }

            // Provider operations
            DbCommand::LoadProviders { reply } => {
                let result = providers::load_providers(conn);
                let _ = reply.send(result);
            }
            DbCommand::SaveProvider { provider, reply } => {
                let result = if providers::load_provider(conn, &provider.id)
                    .unwrap_or(None)
                    .is_some()
                {
                    providers::update_provider(conn, &provider).map(|_| provider.clone())
                } else {
                    providers::insert_provider(conn, &provider).map(|_| provider.clone())
                };
                let _ = reply.send(result);
            }
            DbCommand::DeleteProvider { id, reply } => {
                let result = providers::delete_provider(conn, &id);
                let _ = reply.send(result);
            }
        }
    }
}

// Safety: DbWorker only exposes `send()` which is safe to call from any thread.
// The Connection stays on the background thread and is never moved across threads.
// The mpsc::Sender is already Send + Sync.
unsafe impl Send for DbWorker {}
unsafe impl Sync for DbWorker {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Helper: create oneshot pair, send command, block until reply arrives.
    /// Uses `blocking_recv()` which works outside a tokio runtime.
    fn send_and_recv<F, R>(worker: &DbWorker, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(oneshot::Sender<anyhow::Result<R>>) -> DbCommand,
        R: Send + 'static,
    {
        let (reply_tx, reply_rx) = oneshot::channel();
        worker.send(f(reply_tx)).expect("Send should succeed");
        reply_rx
            .blocking_recv()
            .expect("Reply channel should not be dropped")
    }

    #[test]
    fn test_db_worker_starts_and_handles_send() {
        let tmp_dir = std::env::temp_dir().join("revelry_test_db_worker");
        let db_path = tmp_dir.join("test.db");

        // Clean up from previous test runs
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let worker = DbWorker::start(db_path.clone()).expect("DbWorker should start");

        // Send a LoadCharacters command and get the reply
        let characters = send_and_recv(&worker, |reply| DbCommand::LoadCharacters { reply })
            .expect("LoadCharacters should succeed");
        assert!(characters.is_empty(), "Should have no characters initially");

        // Send a LoadPresets command and get the reply
        let presets = send_and_recv(&worker, |reply| DbCommand::LoadPresets { reply })
            .expect("LoadPresets should succeed");
        assert!(
            presets.len() >= 3,
            "Should have at least 3 built-in presets"
        );

        drop(worker);

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_db_worker_creates_directory() {
        let tmp_dir = std::env::temp_dir().join("revelry_test_db_dir_creation");
        let nested = tmp_dir.join("nested").join("deep");
        let db_path = nested.join("test.db");

        // Clean up from previous test runs
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let worker = DbWorker::start(db_path.clone()).expect("DbWorker should start");
        drop(worker);

        // Give the thread a moment to create the directory
        std::thread::sleep(Duration::from_millis(100));

        assert!(nested.exists(), "DbWorker should create parent directories");

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_db_worker_crud_cycle() {
        let tmp_dir = std::env::temp_dir().join("revelry_test_db_worker_crud");
        let db_path = tmp_dir.join("test.db");

        // Clean up from previous test runs
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let worker = DbWorker::start(db_path.clone()).expect("DbWorker should start");

        // Create a character and save it
        let character =
            crate::domain::character::CharacterCardV2::new("Integration Test".to_string());
        let saved = send_and_recv(&worker, |reply| DbCommand::SaveCharacter {
            character: character.clone(),
            reply,
        })
        .expect("SaveCharacter should succeed");
        assert_eq!(saved.id, character.id);

        // Load characters
        let characters = send_and_recv(&worker, |reply| DbCommand::LoadCharacters { reply })
            .expect("LoadCharacters should succeed");
        assert_eq!(characters.len(), 1, "Should have 1 character");

        // Search characters
        let results = send_and_recv(&worker, |reply| DbCommand::SearchCharacters {
            query: "Integration".to_string(),
            reply,
        })
        .expect("SearchCharacters should succeed");
        assert_eq!(results.len(), 1, "Should find 1 character");

        // Delete the character
        send_and_recv(&worker, |reply| DbCommand::DeleteCharacter {
            id: character.id.clone(),
            reply,
        })
        .expect("DeleteCharacter should succeed");

        // Verify deletion
        let characters = send_and_recv(&worker, |reply| DbCommand::LoadCharacters { reply })
            .expect("LoadCharacters should succeed");
        assert!(characters.is_empty(), "Character should be deleted");

        // Test persona CRUD
        let persona =
            crate::domain::persona::Persona::new("Test Persona".to_string(), "Brave".to_string());
        let saved_persona = send_and_recv(&worker, |reply| DbCommand::SavePersona {
            persona: persona.clone(),
            reply,
        })
        .expect("SavePersona should succeed");
        assert_eq!(saved_persona.id, persona.id);

        let search_results = send_and_recv(&worker, |reply| DbCommand::SearchPersonas {
            query: "Brave".to_string(),
            reply,
        })
        .expect("SearchPersonas should succeed");
        assert_eq!(search_results.len(), 1, "Should find 1 persona");

        send_and_recv(&worker, |reply| DbCommand::DeletePersona {
            id: persona.id.clone(),
            reply,
        })
        .expect("DeletePersona should succeed");

        drop(worker);

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
