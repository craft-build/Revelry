pub mod characters;
pub mod conversations;
pub mod lorebook;
pub mod migrations;
pub mod models;
pub mod personas;
pub mod providers;

use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message};
use crate::domain::lorebook::LorebookEntry;
use crate::domain::persona::Persona;
use crate::domain::provider::ProviderConfig;
use relm4::{ComponentSender, Worker};
use rusqlite::Connection;
use std::path::PathBuf;

/// Input messages sent to the DbService Worker.
#[derive(Debug)]
pub enum DbInput {
    LoadProviders,
    SaveProvider(ProviderConfig),
    DeleteProvider(String),
    LoadAppSetting(String),
    SaveAppSetting { key: String, value: String },
    LoadCharacters,
    SaveCharacter(CharacterCardV2),
    DeleteCharacter(String),
    SearchCharacters(String),
    // Persona operations
    LoadPersonas,
    SavePersona(Persona),
    DeletePersona(String),
    SetGlobalDefaultPersona(Option<String>),
    // Conversation operations
    StartConversation { character_id: String, provider_id: String, creator_persona_id: Option<String> },
    LoadConversations,
    LoadMessages { conversation_id: String, branch_id: Option<String> },
    SaveMessage(Message),
    UpdateMessageContent { id: String, content: String },
    DeleteMessage(String),
    DeleteConversation(String),
    CountMessages(String),
    // Branch operations
    LoadBranches(String),
    CreateBranch(crate::domain::conversation::MessageBranch),
    // Persona-conversation operations
    UpdateConversationPersona { conversation_id: String, persona_id: Option<String> },
    // Lorebook operations
    LoadLorebookEntries(String),
    SaveLorebookEntry(LorebookEntry),
    DeleteLorebookEntry(String),
}

/// Output messages sent from the DbService Worker back to the UI.
#[derive(Debug)]
pub enum DbOutput {
    ProvidersLoaded(Vec<ProviderConfig>),
    ProviderSaved(ProviderConfig),
    ProviderDeleted(String),
    AppSettingLoaded(Option<String>),
    AppSettingSaved,
    CharactersLoaded(Vec<CharacterCardV2>),
    CharacterSaved(CharacterCardV2),
    CharacterDeleted(String),
    CharactersSearched(Vec<CharacterCardV2>),
    // Persona outputs
    PersonasLoaded(Vec<Persona>),
    PersonaSaved(Persona),
    PersonaDeleted(String),
    GlobalDefaultPersonaSet,
    // Conversation outputs
    ConversationStarted(Conversation),
    ConversationsLoaded(Vec<Conversation>),
    MessagesLoaded(Vec<Message>),
    MessageSaved(Message),
    MessageContentUpdated,
    MessageDeleted(String),
    ConversationDeleted(String),
    MessageCount(i32),
    ConversationPersonaUpdated(String),
    // Branch outputs
    BranchesLoaded(Vec<crate::domain::conversation::MessageBranch>),
    BranchCreated(crate::domain::conversation::MessageBranch),
    // Lorebook outputs
    LorebookEntriesLoaded(Vec<LorebookEntry>),
    LorebookEntrySaved(LorebookEntry),
    LorebookEntryDeleted(String),
    Error(String),
}

/// Database service Worker running on a dedicated background thread.
/// Owns the SQLite connection and processes all DB operations serially.
/// This prevents main-loop blocking per PITFALLS.md Pitfall 3.
pub struct DbService {
    conn: Connection, // !Send — stays on worker thread
}

impl Worker for DbService {
    type Init = PathBuf; // Database file path
    type Input = DbInput;
    type Output = DbOutput;

    fn init(db_path: Self::Init, _sender: ComponentSender<Self>) -> Self {
        let mut conn = Connection::open(&db_path)
            .unwrap_or_else(|e| panic!("Failed to open database at {:?}: {}", db_path, e));
        migrations::run_migrations(&mut conn)
            .unwrap_or_else(|e| panic!("Failed to run migrations: {}", e));
        tracing::info!("Database initialized at {:?}", db_path);
        DbService { conn }
    }

    fn update(&mut self, msg: DbInput, sender: ComponentSender<Self>) {
        let result = match msg {
            DbInput::LoadProviders => {
                providers::load_providers(&self.conn)
                    .map(DbOutput::ProvidersLoaded)
            }
            DbInput::SaveProvider(p) => {
                // Check if provider exists → insert or update
                let existing = providers::load_provider(&self.conn, &p.id);
                match existing {
                    Ok(Some(_)) => {
                        providers::update_provider(&self.conn, &p)
                            .map(|_| DbOutput::ProviderSaved(p))
                    }
                    Ok(None) => {
                        providers::insert_provider(&self.conn, &p)
                            .map(|_| DbOutput::ProviderSaved(p))
                    }
                    Err(e) => Err(e),
                }
            }
            DbInput::DeleteProvider(id) => {
                providers::delete_provider(&self.conn, &id)
                    .map(|_| DbOutput::ProviderDeleted(id))
            }
            DbInput::LoadAppSetting(key) => {
                providers::load_app_setting(&self.conn, &key)
                    .map(DbOutput::AppSettingLoaded)
            }
            DbInput::SaveAppSetting { key, value } => {
                providers::save_app_setting(&self.conn, &key, &value)
                    .map(|_| DbOutput::AppSettingSaved)
            }
            DbInput::LoadCharacters => {
                characters::load_characters(&self.conn)
                    .map(DbOutput::CharactersLoaded)
            }
            DbInput::SaveCharacter(c) => {
                // Check if character exists → insert or update (upsert)
                let existing = characters::load_character(&self.conn, &c.id);
                match existing {
                    Ok(Some(_)) => {
                        characters::update_character(&self.conn, &c)
                            .map(|_| DbOutput::CharacterSaved(c))
                    }
                    Ok(None) => {
                        characters::insert_character(&self.conn, &c)
                            .map(|_| DbOutput::CharacterSaved(c))
                    }
                    Err(e) => Err(e),
                }
            }
            DbInput::DeleteCharacter(id) => {
                characters::delete_character(&self.conn, &id)
                    .map(|_| DbOutput::CharacterDeleted(id))
            }
            DbInput::SearchCharacters(query) => {
                characters::search_characters(&self.conn, &query)
                    .map(DbOutput::CharactersSearched)
            }
            DbInput::LoadPersonas => {
                personas::load_personas(&self.conn)
                    .map(DbOutput::PersonasLoaded)
            }
            DbInput::SavePersona(p) => {
                let existing = personas::load_persona(&self.conn, &p.id);
                match existing {
                    Ok(Some(_)) => {
                        personas::update_persona(&self.conn, &p)
                            .map(|_| DbOutput::PersonaSaved(p))
                    }
                    Ok(None) => {
                        personas::insert_persona(&self.conn, &p)
                            .map(|_| DbOutput::PersonaSaved(p))
                    }
                    Err(e) => Err(e),
                }
            }
            DbInput::DeletePersona(id) => {
                personas::delete_persona(&self.conn, &id)
                    .map(|_| DbOutput::PersonaDeleted(id))
            }
            DbInput::SetGlobalDefaultPersona(id) => {
                personas::set_global_default_persona_id(&self.conn, id.as_deref())
                    .map(|_| DbOutput::GlobalDefaultPersonaSet)
            }
            DbInput::StartConversation { character_id, provider_id, creator_persona_id } => {
                // Load the character first
                match characters::load_character(&self.conn, &character_id) {
                    Ok(Some(character)) => {
                        conversations::start_conversation(&self.conn, &character, &provider_id, creator_persona_id.as_deref())
                            .map(DbOutput::ConversationStarted)
                    }
                    Ok(None) => Err(anyhow::anyhow!("Character not found: {}", character_id)),
                    Err(e) => Err(e),
                }
            }
            DbInput::LoadConversations => {
                conversations::load_conversations(&self.conn)
                    .map(DbOutput::ConversationsLoaded)
            }
            DbInput::LoadMessages { conversation_id, branch_id } => {
                conversations::load_messages(&self.conn, &conversation_id, branch_id.as_deref())
                    .map(DbOutput::MessagesLoaded)
            }
            DbInput::SaveMessage(msg) => {
                conversations::save_message(&self.conn, &msg)
                    .map(|_| DbOutput::MessageSaved(msg))
            }
            DbInput::UpdateMessageContent { id, content } => {
                conversations::update_message_content(&self.conn, &id, &content)
                    .map(|_| DbOutput::MessageContentUpdated)
            }
            DbInput::DeleteMessage(id) => {
                conversations::delete_message(&self.conn, &id)
                    .map(|_| DbOutput::MessageDeleted(id))
            }
            DbInput::DeleteConversation(id) => {
                conversations::delete_conversation(&self.conn, &id)
                    .map(|_| DbOutput::ConversationDeleted(id))
            }
            DbInput::CountMessages(conversation_id) => {
                conversations::count_messages_in_conversation(&self.conn, &conversation_id)
                    .map(DbOutput::MessageCount)
            }
            DbInput::LoadBranches(parent_message_id) => {
                conversations::load_branches_for_message(&self.conn, &parent_message_id)
                    .map(DbOutput::BranchesLoaded)
            }
            DbInput::CreateBranch(branch) => {
                conversations::create_branch(&self.conn, &branch)
                    .map(|_| DbOutput::BranchCreated(branch))
            }
            DbInput::UpdateConversationPersona { conversation_id, persona_id } => {
                conversations::update_conversation_persona(&self.conn, &conversation_id, persona_id.as_deref())
                    .map(|_| DbOutput::ConversationPersonaUpdated(conversation_id))
            }
            DbInput::LoadLorebookEntries(character_id) => {
                lorebook::load_lorebook_entries(&self.conn, &character_id)
                    .map(DbOutput::LorebookEntriesLoaded)
            }
            DbInput::SaveLorebookEntry(e) => {
                let existing = lorebook::load_lorebook_entry(&self.conn, &e.id);
                match existing {
                    Ok(Some(_)) => {
                        lorebook::update_lorebook_entry(&self.conn, &e)
                            .map(|_| DbOutput::LorebookEntrySaved(e))
                    }
                    Ok(None) => {
                        lorebook::insert_lorebook_entry(&self.conn, &e)
                            .map(|_| DbOutput::LorebookEntrySaved(e))
                    }
                    Err(e) => Err(e),
                }
            }
            DbInput::DeleteLorebookEntry(id) => {
                lorebook::delete_lorebook_entry(&self.conn, &id)
                    .map(|_| DbOutput::LorebookEntryDeleted(id))
            }
        };

        match result {
            Ok(output) => {
                if let Err(e) = sender.output(output) {
                    tracing::error!("Failed to send DB output: {:?}", e);
                }
            }
            Err(e) => {
                tracing::error!("DB operation failed: {}", e);
                let _ = sender.output(DbOutput::Error(e.to_string()));
            }
        }
    }
}
