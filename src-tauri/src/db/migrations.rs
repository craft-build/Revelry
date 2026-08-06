use rusqlite_migration::{Migrations, M};

/// Returns the migration set for the Phase 2 schema as a single collapsed migration.
/// This produces the complete Phase 2 database schema in one step, rather than
/// replaying the incremental migration history from the gtk_ref implementation.
pub fn get_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // Single collapsed migration producing the complete Phase 2 schema
        M::up(
            "
            -- Providers table (for Phase 3 FK target)
            CREATE TABLE providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider_type TEXT NOT NULL,
                base_url TEXT,
                model_name TEXT NOT NULL DEFAULT '',
                context_window_size INTEGER,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- App settings key-value store
            CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Characters table (full V2 spec with all columns)
            CREATE TABLE characters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                personality TEXT NOT NULL DEFAULT '',
                scenario TEXT NOT NULL DEFAULT '',
                first_mes TEXT NOT NULL DEFAULT '',
                mes_example TEXT NOT NULL DEFAULT '',
                system_prompt TEXT NOT NULL DEFAULT '',
                post_history_instructions TEXT NOT NULL DEFAULT '',
                alternate_greetings TEXT NOT NULL DEFAULT '[]',
                tags TEXT NOT NULL DEFAULT '[]',
                creator TEXT NOT NULL DEFAULT 'Unknown',
                character_version TEXT NOT NULL DEFAULT '2.0',
                avatar_path TEXT,
                extensions TEXT NOT NULL DEFAULT 'null',
                creator_notes TEXT NOT NULL DEFAULT '',
                character_book TEXT NOT NULL DEFAULT 'null',
                default_persona_id TEXT,
                default_preset_id TEXT,
                lorebook_scan_depth INTEGER NOT NULL DEFAULT 2,
                lorebook_recursive_depth INTEGER NOT NULL DEFAULT 2,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- FTS5 virtual table for character search
            CREATE VIRTUAL TABLE characters_fts USING fts5(
                name, description, tags,
                content=characters,
                content_rowid=rowid
            );

            -- FTS5 sync triggers
            CREATE TRIGGER characters_ai AFTER INSERT ON characters BEGIN
                INSERT INTO characters_fts(rowid, name, description, tags)
                VALUES (new.rowid, new.name, new.description, new.tags);
            END;

            CREATE TRIGGER characters_ad AFTER DELETE ON characters BEGIN
                INSERT INTO characters_fts(characters_fts, rowid, name, description, tags)
                VALUES ('delete', old.rowid, old.name, old.description, old.tags);
            END;

            CREATE TRIGGER characters_au AFTER UPDATE ON characters BEGIN
                INSERT INTO characters_fts(characters_fts, rowid, name, description, tags)
                VALUES ('delete', old.rowid, old.name, old.description, old.tags);
                INSERT INTO characters_fts(rowid, name, description, tags)
                VALUES (new.rowid, new.name, new.description, new.tags);
            END;

            -- Conversations table (Phase 2: nullable provider_id, with title and preview)
            CREATE TABLE conversations (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
                provider_id TEXT REFERENCES providers(id) ON DELETE SET NULL,
                creator_persona_id TEXT,
                generation_preset_id TEXT,
                title TEXT NOT NULL DEFAULT '',
                last_message_preview TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);

            -- Messages table (Phase 2: no branch_id)
            CREATE TABLE messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                role TEXT NOT NULL CHECK(role IN ('user','assistant','system')),
                content TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX idx_messages_conv_order ON messages(conversation_id, sort_order);

            -- Personas table
            CREATE TABLE personas (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                appearance TEXT NOT NULL DEFAULT '',
                personality TEXT NOT NULL DEFAULT '',
                background TEXT NOT NULL DEFAULT '',
                role_description TEXT NOT NULL DEFAULT '',
                relationships TEXT NOT NULL DEFAULT '',
                speech_style TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Generation presets table
            CREATE TABLE generation_presets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                temperature REAL NOT NULL DEFAULT 0.8,
                top_p REAL NOT NULL DEFAULT 0.9,
                max_tokens INTEGER NOT NULL DEFAULT 0,
                frequency_penalty REAL NOT NULL DEFAULT 0.0,
                presence_penalty REAL NOT NULL DEFAULT 0.0,
                stop_sequences TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Lorebook entries table (unused until Phase 9)
            CREATE TABLE lorebook_entries (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
                name TEXT NOT NULL DEFAULT '',
                keywords TEXT NOT NULL DEFAULT '[]',
                secondary_terms TEXT NOT NULL DEFAULT '[]',
                content TEXT NOT NULL DEFAULT '',
                insertion_position TEXT NOT NULL DEFAULT 'after_char',
                depth_from_bottom INTEGER NOT NULL DEFAULT 4,
                enabled INTEGER NOT NULL DEFAULT 1,
                token_budget INTEGER,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX idx_lorebook_entries_character ON lorebook_entries(character_id, sort_order);

            -- Regex scripts table (unused until Phase 9)
            CREATE TABLE regex_scripts (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
                name TEXT NOT NULL DEFAULT '',
                find_pattern TEXT NOT NULL DEFAULT '',
                replace_pattern TEXT NOT NULL DEFAULT '',
                flags TEXT NOT NULL DEFAULT 'gi',
                sort_order INTEGER NOT NULL DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX idx_regex_scripts_character ON regex_scripts(character_id, sort_order);
            "
        )
        .down(
            "DROP INDEX IF EXISTS idx_regex_scripts_character;
             DROP TABLE IF EXISTS regex_scripts;
             DROP INDEX IF EXISTS idx_lorebook_entries_character;
             DROP TABLE IF EXISTS lorebook_entries;
             DROP TABLE IF EXISTS generation_presets;
             DROP TABLE IF EXISTS personas;
             DROP INDEX IF EXISTS idx_messages_conv_order;
             DROP TABLE IF EXISTS messages;
             DROP INDEX IF EXISTS idx_conversations_updated;
             DROP TABLE IF EXISTS conversations;
             DROP TRIGGER IF EXISTS characters_au;
             DROP TRIGGER IF EXISTS characters_ad;
             DROP TRIGGER IF EXISTS characters_ai;
             DROP TABLE IF EXISTS characters_fts;
             DROP TABLE IF EXISTS characters;
             DROP TABLE IF EXISTS app_settings;
             DROP TABLE IF EXISTS providers;"
        ),

        // Phase 3: standalone lorebooks + avatar_path on personas
        M::up(
            "
            -- Add avatar_path column to personas
            ALTER TABLE personas ADD COLUMN avatar_path TEXT;

            -- Drop old lorebook_entries table (was per-character, now per-lorebook)
            DROP INDEX IF EXISTS idx_lorebook_entries_character;
            DROP TABLE IF EXISTS lorebook_entries;

            -- Standalone lorebooks table
            CREATE TABLE lorebooks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Lorebook entries (now belong to a lorebook, not a character)
            CREATE TABLE lorebook_entries (
                id TEXT PRIMARY KEY,
                lorebook_id TEXT NOT NULL REFERENCES lorebooks(id) ON DELETE CASCADE,
                keys TEXT NOT NULL DEFAULT '[]',
                secondary_keys TEXT NOT NULL DEFAULT '[]',
                content TEXT NOT NULL DEFAULT '',
                position TEXT NOT NULL DEFAULT 'after_char',
                depth INTEGER NOT NULL DEFAULT 4,
                activation_state TEXT NOT NULL DEFAULT 'always',
                extensions TEXT NOT NULL DEFAULT 'null',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX idx_lorebook_entries_lorebook ON lorebook_entries(lorebook_id);

            -- Character-lorebook junction table (many-to-many)
            CREATE TABLE character_lorebooks (
                character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
                lorebook_id TEXT NOT NULL REFERENCES lorebooks(id) ON DELETE CASCADE,
                sort_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (character_id, lorebook_id)
            );
            "
        )
        .down(
            "DROP TABLE IF EXISTS character_lorebooks;
             DROP INDEX IF EXISTS idx_lorebook_entries_lorebook;
             DROP TABLE IF EXISTS lorebook_entries;
             DROP TABLE IF EXISTS lorebooks;"
        ),

        // Phase 8: message branching + FTS5 full-text search
        M::up(
            "
            -- Message branches table (conversation forking)
            CREATE TABLE message_branches (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                parent_message_id TEXT NOT NULL,
                label TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX idx_message_branches_parent ON message_branches(parent_message_id);
            CREATE INDEX idx_message_branches_conv ON message_branches(conversation_id);

            -- Add branch_id column to messages (nullable FK to message_branches)
            ALTER TABLE messages ADD COLUMN branch_id TEXT REFERENCES message_branches(id) ON DELETE CASCADE;
            CREATE INDEX idx_messages_conv_branch ON messages(conversation_id, branch_id);

            -- FTS5 virtual table for message content search
            CREATE VIRTUAL TABLE messages_fts USING fts5(
                content,
                content=messages,
                content_rowid=rowid
            );

            -- FTS5 sync triggers
            CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
            END;

            CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
            END;

            CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
                INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
            END;
            "
        )
        .down(
            "DROP TRIGGER IF EXISTS messages_au;
             DROP TRIGGER IF EXISTS messages_ad;
             DROP TRIGGER IF EXISTS messages_ai;
             DROP TABLE IF EXISTS messages_fts;
             DROP INDEX IF EXISTS idx_messages_conv_branch;
             DROP INDEX IF EXISTS idx_message_branches_conv;
             DROP INDEX IF EXISTS idx_message_branches_parent;
             DROP TABLE IF EXISTS message_branches;"
        ),

        // Phase 4: provider models + conversation model_name
        M::up(
            "
            -- Add models JSON column to providers (1:N provider-models)
            ALTER TABLE providers ADD COLUMN models TEXT NOT NULL DEFAULT '[]';

            -- Add default_params JSON column to providers (generation params)
            ALTER TABLE providers ADD COLUMN default_params TEXT;

            -- Add model_name to conversations (provider_id + model_name pair)
            ALTER TABLE conversations ADD COLUMN model_name TEXT;
            "
        )
        .down(
            "
            ALTER TABLE conversations DROP COLUMN model_name;
            ALTER TABLE providers DROP COLUMN default_params;
            ALTER TABLE providers DROP COLUMN models;
            "
        ),

        // Phase 9: lorebook_entries new columns + conversation_lorebooks junction table
        M::up(
            "
            -- Add token_budget column to lorebook_entries (nullable INTEGER)
            ALTER TABLE lorebook_entries ADD COLUMN token_budget INTEGER;

            -- Add sort_order column to lorebook_entries (default 0)
            ALTER TABLE lorebook_entries ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

            -- Add enabled column to lorebook_entries (default 1 = true)
            ALTER TABLE lorebook_entries ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;

            -- Create conversation_lorebooks junction table (mirrors character_lorebooks structure)
            CREATE TABLE conversation_lorebooks (
                conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                lorebook_id TEXT NOT NULL REFERENCES lorebooks(id) ON DELETE CASCADE,
                sort_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (conversation_id, lorebook_id)
            );
            CREATE INDEX idx_conversation_lorebooks_conv ON conversation_lorebooks(conversation_id, sort_order);
            "
        )
        .down(
            "
            DROP INDEX IF EXISTS idx_conversation_lorebooks_conv;
            DROP TABLE IF EXISTS conversation_lorebooks;
            ALTER TABLE lorebook_entries DROP COLUMN enabled;
            ALTER TABLE lorebook_entries DROP COLUMN sort_order;
            ALTER TABLE lorebook_entries DROP COLUMN token_budget;
            "
        ),

        // Phase 10: per-conversation max output token override
        M::up(
            "
            ALTER TABLE conversations ADD COLUMN override_max_tokens INTEGER;
            "
        )
        .down(
            "
            ALTER TABLE conversations DROP COLUMN override_max_tokens;
            "
        ),
    ])
}

/// Configures SQLite connection pragmas. Must be called before migrations
/// because PRAGMA journal_mode=WAL cannot be set inside a transaction.
pub fn configure_connection(conn: &rusqlite::Connection) -> Result<(), rusqlite_migration::Error> {
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| rusqlite_migration::Error::RusqliteError {
            query: "PRAGMA journal_mode=WAL".into(),
            err: e.into(),
        })?;
    // synchronous=NORMAL (the recommended companion to WAL) avoids an fsync
    // per commit. A process crash/close never loses committed WAL data; only a
    // power loss could drop the very last transaction. This keeps frequent
    // per-token writes (streamed-message persistence) cheap without risking
    // database corruption.
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| rusqlite_migration::Error::RusqliteError {
            query: "PRAGMA synchronous=NORMAL".into(),
            err: e,
        })?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| rusqlite_migration::Error::RusqliteError {
            query: "PRAGMA busy_timeout=5000".into(),
            err: e.into(),
        })?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| rusqlite_migration::Error::RusqliteError {
            query: "PRAGMA foreign_keys=ON".into(),
            err: e.into(),
        })?;
    Ok(())
}

/// Runs all pending migrations on the given connection.
/// Configures the connection pragmas first (WAL, busy_timeout, foreign_keys).
pub fn run_migrations(conn: &mut rusqlite::Connection) -> Result<(), rusqlite_migration::Error> {
    configure_connection(conn)?;
    get_migrations().to_latest(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    #[test]
    fn test_migrations_run_successfully() {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        let result = run_migrations(&mut conn);
        assert!(result.is_ok(), "Migrations should run without error");
    }

    #[test]
    fn test_providers_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='providers')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "providers table should exist after migration");
    }

    #[test]
    fn test_app_settings_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='app_settings')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "app_settings table should exist after migration");
    }

    #[test]
    fn test_characters_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='characters')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "characters table should exist after migration");
    }

    #[test]
    fn test_characters_fts_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='characters_fts')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "characters_fts table should exist after migration");
    }

    #[test]
    fn test_conversations_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='conversations')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "conversations table should exist after migration");
    }

    #[test]
    fn test_messages_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='messages')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "messages table should exist after migration");
    }

    #[test]
    fn test_personas_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='personas')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "personas table should exist after migration");
    }

    #[test]
    fn test_generation_presets_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='generation_presets')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(
            result,
            "generation_presets table should exist after migration"
        );
    }

    #[test]
    fn test_providers_table_columns() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM providers")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.contains(&"id".to_string()),
            "providers should have 'id' column"
        );
        assert!(
            columns.contains(&"name".to_string()),
            "providers should have 'name' column"
        );
        assert!(
            columns.contains(&"provider_type".to_string()),
            "providers should have 'provider_type' column"
        );
        assert!(
            columns.contains(&"base_url".to_string()),
            "providers should have 'base_url' column"
        );
        assert!(
            columns.contains(&"model_name".to_string()),
            "providers should have 'model_name' column"
        );
        assert!(
            columns.contains(&"context_window_size".to_string()),
            "providers should have 'context_window_size' column"
        );
        assert!(
            columns.contains(&"enabled".to_string()),
            "providers should have 'enabled' column"
        );
        assert!(
            columns.contains(&"created_at".to_string()),
            "providers should have 'created_at' column"
        );
        assert!(
            columns.contains(&"updated_at".to_string()),
            "providers should have 'updated_at' column"
        );
    }

    #[test]
    fn test_characters_table_has_all_columns() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM characters")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let expected = [
            "id",
            "name",
            "description",
            "personality",
            "scenario",
            "first_mes",
            "mes_example",
            "system_prompt",
            "post_history_instructions",
            "alternate_greetings",
            "tags",
            "creator",
            "character_version",
            "avatar_path",
            "extensions",
            "creator_notes",
            "character_book",
            "default_persona_id",
            "default_preset_id",
            "lorebook_scan_depth",
            "lorebook_recursive_depth",
            "created_at",
            "updated_at",
        ];
        for col in &expected {
            assert!(
                columns.contains(&col.to_string()),
                "characters should have '{}' column",
                col
            );
        }
    }

    #[test]
    fn test_conversations_has_title_and_preview() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM conversations")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.contains(&"title".to_string()),
            "conversations should have 'title' column"
        );
        assert!(
            columns.contains(&"last_message_preview".to_string()),
            "conversations should have 'last_message_preview' column"
        );
        assert!(
            columns.contains(&"provider_id".to_string()),
            "conversations should have 'provider_id' column"
        );
        assert!(
            columns.contains(&"generation_preset_id".to_string()),
            "conversations should have 'generation_preset_id' column"
        );
    }

    #[test]
    fn test_messages_has_branch_id() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM messages")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.contains(&"branch_id".to_string()),
            "messages should have 'branch_id' column (Phase 8)"
        );
        let expected = [
            "id",
            "conversation_id",
            "role",
            "content",
            "sort_order",
            "timestamp",
            "branch_id",
        ];
        for col in &expected {
            assert!(
                columns.contains(&col.to_string()),
                "messages should have '{}' column",
                col
            );
        }
    }

    #[test]
    fn test_message_branches_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='message_branches')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(
            result,
            "message_branches table should exist after Phase 8 migration"
        );
    }

    #[test]
    fn test_messages_fts_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='messages_fts')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(
            result,
            "messages_fts virtual table should exist after Phase 8 migration"
        );
    }

    #[test]
    fn test_generation_presets_name_unique() {
        let conn = setup_test_db();
        // Try inserting duplicate names — should fail
        conn.execute(
            "INSERT INTO generation_presets (id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty, presence_penalty, stop_sequences, created_at, updated_at) VALUES ('id1', 'Test', 0, 0.8, 0.9, 0, 0.0, 0.0, '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).expect("first insert should succeed");
        let result = conn.execute(
            "INSERT INTO generation_presets (id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty, presence_penalty, stop_sequences, created_at, updated_at) VALUES ('id2', 'Test', 0, 0.8, 0.9, 0, 0.0, 0.0, '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        );
        assert!(
            result.is_err(),
            "duplicate preset name should fail due to UNIQUE constraint"
        );
    }

    #[test]
    fn test_no_api_key_column_in_providers() {
        // Security check: providers table must NOT have an api_key column
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM providers")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            !columns
                .iter()
                .any(|c| c.contains("api_key") || c.contains("key")),
            "SECURITY: providers table must NOT contain api_key or key column"
        );
    }

    #[test]
    fn test_busy_timeout_set() {
        let conn = setup_test_db();
        let busy_timeout: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("Failed to query busy_timeout");
        assert_eq!(busy_timeout, 5000, "busy_timeout should be 5000ms");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = setup_test_db();
        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("Failed to query foreign_keys");
        assert_eq!(foreign_keys, 1, "foreign_keys should be enabled");
    }

    // ── Phase 3 migration tests ───────────────────────────────────────────

    #[test]
    fn test_lorebooks_table_exists() {
        let conn = setup_test_db();
        let result: i64 = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='lorebooks')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert_eq!(
            result, 1,
            "lorebooks table should exist after Phase 3 migration"
        );
    }

    #[test]
    fn test_lorebook_entries_table_exists() {
        let conn = setup_test_db();
        let result: i64 = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='lorebook_entries')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert_eq!(
            result, 1,
            "lorebook_entries table should exist after Phase 3 migration"
        );
    }

    #[test]
    fn test_character_lorebooks_table_exists() {
        let conn = setup_test_db();
        let result: i64 = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='character_lorebooks')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert_eq!(
            result, 1,
            "character_lorebooks table should exist after Phase 3 migration"
        );
    }

    #[test]
    fn test_personas_has_avatar_path() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM personas")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "avatar_path"),
            "personas table should have avatar_path column after Phase 3 migration"
        );
    }

    #[test]
    fn test_lorebook_entries_has_lorebook_id() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM lorebook_entries")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "lorebook_id"),
            "lorebook_entries table should have lorebook_id column (not character_id)"
        );
        assert!(
            !columns.iter().any(|c| c == "character_id"),
            "lorebook_entries table should NOT have character_id column after Phase 3 migration"
        );
    }

    #[test]
    fn test_lorebook_entries_columns() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM lorebook_entries")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let expected = vec![
            "id",
            "lorebook_id",
            "keys",
            "secondary_keys",
            "content",
            "position",
            "depth",
            "activation_state",
            "extensions",
            "created_at",
            "updated_at",
        ];
        for col in &expected {
            assert!(
                columns.iter().any(|c| c == col),
                "lorebook_entries should have '{}' column",
                col
            );
        }
    }

    // ── Phase 4 migration tests ───────────────────────────────────────────

    #[test]
    fn test_providers_has_models_column() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM providers")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "models"),
            "providers table should have 'models' column after Phase 4 migration"
        );
    }

    #[test]
    fn test_providers_has_default_params_column() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM providers")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "default_params"),
            "providers table should have 'default_params' column after Phase 4 migration"
        );
    }

    #[test]
    fn test_conversations_has_model_name() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM conversations")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "model_name"),
            "conversations table should have 'model_name' column after Phase 4 migration"
        );
    }

    // ── Phase 9 migration tests ───────────────────────────────────────────

    #[test]
    fn test_lorebook_entries_has_token_budget() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM lorebook_entries")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "token_budget"),
            "lorebook_entries should have 'token_budget' column after Phase 9 migration"
        );
    }

    #[test]
    fn test_lorebook_entries_has_sort_order() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM lorebook_entries")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "sort_order"),
            "lorebook_entries should have 'sort_order' column after Phase 9 migration"
        );
    }

    #[test]
    fn test_lorebook_entries_has_enabled() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM lorebook_entries")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "enabled"),
            "lorebook_entries should have 'enabled' column after Phase 9 migration"
        );
    }

    #[test]
    fn test_conversation_lorebooks_table_exists() {
        let conn = setup_test_db();
        let result: i64 = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='conversation_lorebooks')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert_eq!(
            result, 1,
            "conversation_lorebooks table should exist after Phase 9 migration"
        );
    }

    #[test]
    fn test_conversation_lorebooks_columns() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM conversation_lorebooks")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let expected = vec!["conversation_id", "lorebook_id", "sort_order"];
        for col in &expected {
            assert!(
                columns.iter().any(|c| c == col),
                "conversation_lorebooks should have '{}' column",
                col
            );
        }
    }
}
