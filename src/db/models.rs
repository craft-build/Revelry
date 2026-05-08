use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message, MessageBranch, MessageRole};
use crate::domain::lorebook::{InsertionPosition, LorebookEntry};
use crate::domain::persona::Persona;
use crate::domain::provider::{ProviderConfig, ProviderType};
use chrono::NaiveDateTime;
use rusqlite::Row;
use std::str::FromStr;

/// Converts a database row into a ProviderConfig.
/// Timestamps are stored as TEXT and parsed from the database.
pub fn row_to_provider(row: &Row) -> rusqlite::Result<ProviderConfig> {
    let provider_type_str: String = row.get("provider_type")?;
    let provider_type: ProviderType = serde_json::from_str(&provider_type_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;
    let updated_at = NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    Ok(ProviderConfig {
        id: row.get("id")?,
        name: row.get("name")?,
        provider_type,
        base_url: row.get("base_url")?,
        model_name: row.get("model_name")?,
        context_window_size: row.get("context_window_size")?,
        enabled: row.get::<_, i32>("enabled")? != 0,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a CharacterCardV2.
/// JSON fields (alternate_greetings, tags, extensions) are deserialized from TEXT.
/// Timestamps are stored as TEXT and parsed with the standard format.
pub fn row_to_character(row: &Row) -> rusqlite::Result<CharacterCardV2> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;
    let updated_at = NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let alt_greetings_str: String = row.get("alternate_greetings")?;
    let alternate_greetings: Vec<String> = serde_json::from_str(&alt_greetings_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let extensions_str: String = row.get("extensions")?;
    let extensions: serde_json::Value = serde_json::from_str(&extensions_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let character_book_str: String = row.get("character_book")?;
    let character_book: serde_json::Value = serde_json::from_str(&character_book_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

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
        lorebook_scan_depth: row.get("lorebook_scan_depth")?,
        lorebook_recursive_depth: row.get("lorebook_recursive_depth")?,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a Persona.
/// Tags are deserialized from a JSON array stored as TEXT.
/// Timestamps are stored as TEXT and parsed with the standard format.
pub fn row_to_persona(row: &Row) -> rusqlite::Result<Persona> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;
    let updated_at = NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

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
        created_at,
        updated_at,
    })
}

/// Converts a database row into a Conversation.
/// Timestamps are stored as TEXT and parsed from the database.
pub fn row_to_conversation(row: &Row) -> rusqlite::Result<Conversation> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;
    let updated_at = NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    Ok(Conversation {
        id: row.get("id")?,
        character_id: row.get("character_id")?,
        provider_id: row.get("provider_id")?,
        creator_persona_id: row.get("creator_persona_id")?,
        created_at,
        updated_at,
    })
}

/// Converts a database row into a Message.
/// Timestamps are stored as TEXT and parsed from the database.
pub fn row_to_message(row: &Row) -> rusqlite::Result<Message> {
    let timestamp_str: String = row.get("timestamp")?;
    let timestamp = NaiveDateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

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
        branch_id: row.get("branch_id")?,
        role,
        content: row.get("content")?,
        timestamp,
        sort_order: row.get("sort_order")?,
    })
}

/// Converts a database row into a MessageBranch.
/// Timestamps are stored as TEXT and parsed from the database.
pub fn row_to_message_branch(row: &Row) -> rusqlite::Result<MessageBranch> {
    let created_at_str: String = row.get("created_at")?;
    let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    Ok(MessageBranch {
        id: row.get("id")?,
        conversation_id: row.get("conversation_id")?,
        parent_message_id: row.get("parent_message_id")?,
        label: row.get("label")?,
        created_at,
    })
}

/// Converts a database row into a LorebookEntry.
/// Keywords and secondary_terms are deserialized from JSON arrays stored as TEXT.
/// InsertionPosition is stored as TEXT and parsed via FromStr.
/// Timestamps are stored as TEXT and parsed with the standard format.
pub fn row_to_lorebook_entry(row: &Row) -> rusqlite::Result<LorebookEntry> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;
    let updated_at = NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let keywords_str: String = row.get("keywords")?;
    let keywords: Vec<String> = serde_json::from_str(&keywords_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let secondary_terms_str: String = row.get("secondary_terms")?;
    let secondary_terms: Vec<String> = serde_json::from_str(&secondary_terms_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))?;

    let insertion_position_str: String = row.get("insertion_position")?;
    let insertion_position: InsertionPosition = InsertionPosition::from_str(&insertion_position_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        ))?;

    Ok(LorebookEntry {
        id: row.get("id")?,
        character_id: row.get("character_id")?,
        name: row.get("name")?,
        keywords,
        secondary_terms,
        content: row.get("content")?,
        insertion_position,
        depth_from_bottom: row.get("depth_from_bottom")?,
        enabled: row.get::<_, i32>("enabled")? != 0,
        token_budget: row.get("token_budget")?,
        sort_order: row.get("sort_order")?,
        created_at,
        updated_at,
    })
}
