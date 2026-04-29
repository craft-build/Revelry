pub mod migrations;
pub mod models;
pub mod providers;

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
}

/// Output messages sent from the DbService Worker back to the UI.
#[derive(Debug)]
pub enum DbOutput {
    ProvidersLoaded(Vec<ProviderConfig>),
    ProviderSaved(ProviderConfig),
    ProviderDeleted(String),
    AppSettingLoaded(Option<String>),
    AppSettingSaved,
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
