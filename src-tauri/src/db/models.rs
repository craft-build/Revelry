use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message, MessageBranch, MessageRole};
use crate::domain::generation::GenerationPreset;
use crate::domain::lorebook::{Lorebook, LorebookActivation, LorebookEntry, LorebookPosition};
use crate::domain::persona::Persona;
use crate::domain::provider::{GenerationParams, ProviderConfig, ProviderModelEntry, ProviderType};
use crate::domain::regex_script::RegexScript;
use chrono::{DateTime, Utc};
use rusqlite::Row;
use std::str::FromStr;

/// Parses a RFC 3339 timestamp string from the database into DateTime<Utc>.
fn parse_timestamp(s: &str) -> rusqlite::Result<DateTime<Utc>> {
    s.parse::<DateTime<Utc>>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })
}

/// Converts a database row into a CharacterCardV2.
/// JSON fields (alternate_greetings, tags, extensions, character_book) are deserialized from TEXT.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_character(row: &Row) -> rusqlite::Result<CharacterCardV2> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    let alt_greetings_str: String = row.get("alternate_greetings")?;
    let alternate_greetings: Vec<String> =
        serde_json::from_str(&alt_greetings_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;

    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let extensions_str: String = row.get("extensions")?;
    let extensions: serde_json::Value = serde_json::from_str(&extensions_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let character_book_str: String = row.get("character_book")?;
    let character_book: serde_json::Value =
        serde_json::from_str(&character_book_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;

    Ok(CharacterCardV2 {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        personality: row.get("personality")?,
        scenario: row.get("scenario")?,
        first_mes: row.get("first_mes")?,
        mes_example: row.get("mes_example")?,
        system_prompt: row.get("system_prompt")?,
        post_history_instructions: row.get("post_history_instructions")?,
        alternate_greetings,
        tags,
        creator: row.get("creator")?,
        character_version: row.get("character_version")?,
        avatar_path: row.get("avatar_path")?,
        extensions,
        creator_notes: row.get("creator_notes")?,
        character_book,
        default_persona_id: row.get("default_persona_id")?,
        default_preset_id: row.get("default_preset_id")?,
        lorebook_scan_depth: row.get("lorebook_scan_depth")?,
        lorebook_recursive_depth: row.get("lorebook_recursive_depth")?,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a Persona.
/// Tags are deserialized from a JSON array stored as TEXT.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_persona(row: &Row) -> rusqlite::Result<Persona> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(Persona {
        id: row.get("id")?,
        name: row.get("name")?,
        appearance: row.get("appearance")?,
        personality: row.get("personality")?,
        background: row.get("background")?,
        role_description: row.get("role_description")?,
        relationships: row.get("relationships")?,
        speech_style: row.get("speech_style")?,
        tags,
        avatar_path: row.get("avatar_path")?,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a Conversation.
/// Phase 2 includes title, last_message_preview, and generation_preset_id.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_conversation(row: &Row) -> rusqlite::Result<Conversation> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    Ok(Conversation {
        id: row.get("id")?,
        character_id: row.get("character_id")?,
        provider_id: row.get("provider_id")?,
        model_name: row.get("model_name")?,
        creator_persona_id: row.get("creator_persona_id")?,
        generation_preset_id: row.get("generation_preset_id")?,
        override_max_tokens: row.get("override_max_tokens")?,
        title: row.get("title")?,
        last_message_preview: row.get("last_message_preview")?,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a Message.
/// Phase 8: includes branch_id. Role is parsed via MessageRole::from_str().
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_message(row: &Row) -> rusqlite::Result<Message> {
    let timestamp_str: String = row.get("timestamp")?;
    let timestamp = parse_timestamp(&timestamp_str)?;

    let role_str: String = row.get("role")?;
    let role = match MessageRole::from_str(&role_str) {
        Ok(r) => r,
        Err(e) => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            ));
        }
    };

    Ok(Message {
        id: row.get("id")?,
        conversation_id: row.get("conversation_id")?,
        role,
        content: row.get("content")?,
        sort_order: row.get("sort_order")?,
        timestamp,
        branch_id: row.get("branch_id")?,
    })
}

/// Converts a database row into a MessageBranch.
pub fn row_to_message_branch(row: &Row) -> rusqlite::Result<MessageBranch> {
    let created_at_str: String = row.get("created_at")?;
    let created_at = parse_timestamp(&created_at_str)?;

    Ok(MessageBranch {
        id: row.get("id")?,
        conversation_id: row.get("conversation_id")?,
        parent_message_id: row.get("parent_message_id")?,
        label: row.get("label")?,
        created_at,
    })
}

/// Converts a database row into a ProviderConfig.
/// JSON fields (provider_type, models, default_params) are deserialized from TEXT.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_provider(row: &Row) -> rusqlite::Result<ProviderConfig> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    // ProviderType from JSON TEXT column
    let provider_type_str: String = row.get("provider_type")?;
    let provider_type: ProviderType = serde_json::from_str(&provider_type_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    // models JSON TEXT column
    let models_str: String = row.get("models")?;
    let models: Vec<ProviderModelEntry> = serde_json::from_str(&models_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    // default_params JSON TEXT column (nullable)
    let default_params: Option<GenerationParams> = {
        let default_params_str: Option<String> = row.get("default_params")?;
        match default_params_str {
            Some(s) if s != "null" => serde_json::from_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
            _ => None,
        }
    };

    Ok(ProviderConfig {
        id: row.get("id")?,
        name: row.get("name")?,
        provider_type,
        base_url: row.get("base_url")?,
        model_name: row.get("model_name")?,
        models,
        context_window_size: row.get("context_window_size")?,
        enabled: row.get::<_, i32>("enabled")? != 0,
        default_params,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a GenerationPreset.
/// stop_sequences is deserialized from a JSON array stored as TEXT.
/// is_builtin is stored as INTEGER and converted to bool via != 0.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_generation_preset(row: &Row) -> rusqlite::Result<GenerationPreset> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    let stop_sequences_str: String = row.get("stop_sequences")?;
    let stop_sequences: Vec<String> = serde_json::from_str(&stop_sequences_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(GenerationPreset {
        id: row.get("id")?,
        name: row.get("name")?,
        is_builtin: row.get::<_, i32>("is_builtin")? != 0,
        temperature: row.get("temperature")?,
        top_p: row.get("top_p")?,
        max_tokens: row.get("max_tokens")?,
        frequency_penalty: row.get("frequency_penalty")?,
        presence_penalty: row.get("presence_penalty")?,
        stop_sequences,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a Lorebook.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_lorebook(row: &Row) -> rusqlite::Result<Lorebook> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    Ok(Lorebook {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a LorebookEntry.
/// JSON fields (keys, secondary_keys, extensions) are deserialized from TEXT.
/// Position and activation_state are parsed from TEXT via FromStr.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_lorebook_entry(row: &Row) -> rusqlite::Result<LorebookEntry> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    let keys_str: String = row.get("keys")?;
    let keys: Vec<String> = serde_json::from_str(&keys_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let secondary_keys_str: String = row.get("secondary_keys")?;
    let secondary_keys: Vec<String> = serde_json::from_str(&secondary_keys_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let position_str: String = row.get("position")?;
    let position = match LorebookPosition::from_str(&position_str) {
        Ok(p) => p,
        Err(e) => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            ));
        }
    };

    let activation_str: String = row.get("activation_state")?;
    let activation_state = match LorebookActivation::from_str(&activation_str) {
        Ok(a) => a,
        Err(e) => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            ));
        }
    };

    let extensions_str: String = row.get("extensions")?;
    let extensions: serde_json::Value = serde_json::from_str(&extensions_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(LorebookEntry {
        id: row.get("id")?,
        lorebook_id: row.get("lorebook_id")?,
        keys,
        secondary_keys,
        content: row.get("content")?,
        position,
        depth: row.get("depth")?,
        activation_state,
        token_budget: row.get::<_, Option<i64>>("token_budget")?,
        sort_order: row.get("sort_order")?,
        enabled: row.get::<_, i32>("enabled")? != 0,
        extensions,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a RegexScript.
/// `enabled` is stored as INTEGER (1/0) and converted to bool.
/// Timestamps are stored as RFC 3339 TEXT and parsed into DateTime<Utc>.
pub fn row_to_regex_script(row: &Row) -> rusqlite::Result<RegexScript> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    Ok(RegexScript {
        id: row.get("id")?,
        character_id: row.get("character_id")?,
        name: row.get("name")?,
        find_pattern: row.get("find_pattern")?,
        replace_pattern: row.get("replace_pattern")?,
        flags: row.get("flags")?,
        sort_order: row.get("sort_order")?,
        enabled: row.get::<_, i32>("enabled")? != 0,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    #[test]
    fn test_row_to_character() {
        let conn = setup_test_db();
        let character = crate::domain::character::CharacterCardV2::new("Mapper Test".to_string());
        crate::db::characters::insert_character(&conn, &character).expect("Insert should succeed");

        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, personality, scenario, first_mes, mes_example,
                 system_prompt, post_history_instructions, alternate_greetings, tags, creator,
                 character_version, avatar_path, extensions, creator_notes, character_book,
                 default_persona_id, default_preset_id, lorebook_scan_depth, lorebook_recursive_depth,
                 created_at, updated_at FROM characters WHERE id = ?1",
            )
            .expect("prepare failed");
        let loaded: CharacterCardV2 = stmt
            .query_row([&character.id], |row| row_to_character(row))
            .expect("query failed");

        assert_eq!(loaded.id, character.id);
        assert_eq!(loaded.name, "Mapper Test");
        assert!(loaded.alternate_greetings.is_empty());
        assert!(loaded.tags.is_empty());
        assert_eq!(loaded.extensions, serde_json::json!({}));
        assert_eq!(loaded.character_book, serde_json::json!({}));
    }

    #[test]
    fn test_row_to_persona() {
        let conn = setup_test_db();
        let persona = crate::domain::persona::Persona::new(
            "Mapper Persona".to_string(),
            "Friendly".to_string(),
        );
        crate::db::personas::insert_persona(&conn, &persona).expect("Insert should succeed");

        let mut stmt = conn
            .prepare(
                "SELECT id, name, appearance, personality, background, role_description,
                 relationships, speech_style, tags, avatar_path, created_at, updated_at
                 FROM personas WHERE id = ?1",
            )
            .expect("prepare failed");
        let loaded: Persona = stmt
            .query_row([&persona.id], |row| row_to_persona(row))
            .expect("query failed");

        assert_eq!(loaded.id, persona.id);
        assert_eq!(loaded.name, "Mapper Persona");
        assert_eq!(loaded.personality, "Friendly");
        assert!(loaded.tags.is_empty());
        assert!(loaded.avatar_path.is_none());
    }

    #[test]
    fn test_row_to_generation_preset() {
        let conn = setup_test_db();
        crate::db::generation::seed_builtin_presets(&conn).expect("Seed should succeed");

        let mut stmt = conn
            .prepare(
                "SELECT id, name, is_builtin, temperature, top_p, max_tokens,
                 frequency_penalty, presence_penalty, stop_sequences, created_at, updated_at
                 FROM generation_presets WHERE id = 'builtin-creative'",
            )
            .expect("prepare failed");
        let loaded: GenerationPreset = stmt
            .query_row([], |row| row_to_generation_preset(row))
            .expect("query failed");

        assert_eq!(loaded.id, "builtin-creative");
        assert_eq!(loaded.name, "Creative");
        assert!(loaded.is_builtin);
        assert_eq!(loaded.temperature, 1.2);
    }
}
