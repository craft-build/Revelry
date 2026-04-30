use crate::domain::character::CharacterCardV2;
use crate::domain::provider::{ProviderConfig, ProviderType};
use chrono::NaiveDateTime;
use rusqlite::Row;

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
        created_at,
        updated_at,
    })
}
