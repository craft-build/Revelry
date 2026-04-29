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
