use crate::db::models::row_to_provider;
use crate::domain::provider::ProviderConfig;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Inserts a new provider into the database.
/// JSON fields (provider_type, models, default_params) are serialized to TEXT.
/// Timestamps are stored as RFC 3339 TEXT.
pub fn insert_provider(conn: &Connection, provider: &ProviderConfig) -> Result<()> {
    let provider_type_str = serde_json::to_string(&provider.provider_type)?;
    let models_str = serde_json::to_string(&provider.models)?;
    let default_params_str = match &provider.default_params {
        Some(p) => serde_json::to_string(p)?,
        None => "null".to_string(),
    };

    conn.prepare_cached(
        "INSERT INTO providers (id, name, provider_type, base_url, model_name, models,
         context_window_size, default_params, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?
    .execute(params![
        provider.id,
        provider.name,
        provider_type_str,
        provider.base_url,
        provider.model_name,
        models_str,
        provider.context_window_size,
        default_params_str,
        provider.enabled as i32,
        provider.created_at.to_rfc3339(),
        provider.updated_at.to_rfc3339(),
    ])?;
    Ok(())
}

/// Loads all providers, ordered by updated_at descending (most recently modified first).
pub fn load_providers(conn: &Connection) -> Result<Vec<ProviderConfig>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, provider_type, base_url, model_name, models,
         context_window_size, default_params, enabled, created_at, updated_at
         FROM providers ORDER BY updated_at DESC",
    )?;
    let providers = stmt.query_map([], |row| row_to_provider(row))?;
    let mut result = Vec::new();
    for provider in providers {
        result.push(provider?);
    }
    Ok(result)
}

/// Loads a single provider by ID.
pub fn load_provider(conn: &Connection, id: &str) -> Result<Option<ProviderConfig>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, provider_type, base_url, model_name, models,
         context_window_size, default_params, enabled, created_at, updated_at
         FROM providers WHERE id = ?1",
    )?;
    match stmt.query_row(params![id], |row| row_to_provider(row)) {
        Ok(provider) => Ok(Some(provider)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Updates an existing provider in the database.
/// JSON fields (provider_type, models, default_params) are serialized to TEXT.
/// Timestamps are stored as RFC 3339 TEXT.
pub fn update_provider(conn: &Connection, provider: &ProviderConfig) -> Result<()> {
    let provider_type_str = serde_json::to_string(&provider.provider_type)?;
    let models_str = serde_json::to_string(&provider.models)?;
    let default_params_str = match &provider.default_params {
        Some(p) => serde_json::to_string(p)?,
        None => "null".to_string(),
    };

    conn.prepare_cached(
        "UPDATE providers SET name = ?1, provider_type = ?2, base_url = ?3, model_name = ?4,
         models = ?5, context_window_size = ?6, default_params = ?7, enabled = ?8,
         updated_at = ?9 WHERE id = ?10",
    )?
    .execute(params![
        provider.name,
        provider_type_str,
        provider.base_url,
        provider.model_name,
        models_str,
        provider.context_window_size,
        default_params_str,
        provider.enabled as i32,
        provider.updated_at.to_rfc3339(),
        provider.id,
    ])?;
    Ok(())
}

/// Deletes a provider from the database.
pub fn delete_provider(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM providers WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::domain::provider::ProviderType;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    #[test]
    fn test_insert_and_load_provider() {
        let conn = setup_test_db();
        let provider = ProviderConfig::new("Test OpenAI".to_string(), ProviderType::OpenAI);

        insert_provider(&conn, &provider).expect("Insert should succeed");

        let loaded = load_providers(&conn).expect("Load should succeed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, provider.id);
        assert_eq!(loaded[0].name, "Test OpenAI");
        assert_eq!(loaded[0].provider_type, ProviderType::OpenAI);
        assert_eq!(loaded[0].model_name, "gpt-4o-mini");
        assert_eq!(loaded[0].models.len(), 1);
    }

    #[test]
    fn test_load_provider_by_id() {
        let conn = setup_test_db();
        let provider = ProviderConfig::new("Test Anthropic".to_string(), ProviderType::Anthropic);
        insert_provider(&conn, &provider).expect("Insert should succeed");

        let loaded = load_provider(&conn, &provider.id).expect("Load by ID should succeed");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "Test Anthropic");
    }

    #[test]
    fn test_load_provider_not_found() {
        let conn = setup_test_db();
        let loaded = load_provider(&conn, "nonexistent-id").expect("Load should succeed");
        assert!(loaded.is_none());
    }

    #[test]
    fn test_update_provider() {
        let conn = setup_test_db();
        let mut provider = ProviderConfig::new("Test Ollama".to_string(), ProviderType::Ollama);
        insert_provider(&conn, &provider).expect("Insert should succeed");

        provider.name = "Updated Ollama".to_string();
        provider.enabled = false;
        update_provider(&conn, &provider).expect("Update should succeed");

        let loaded = load_provider(&conn, &provider.id)
            .expect("Load should succeed")
            .expect("Provider should exist");
        assert_eq!(loaded.name, "Updated Ollama");
        assert!(!loaded.enabled);
    }

    #[test]
    fn test_delete_provider() {
        let conn = setup_test_db();
        let provider = ProviderConfig::new("To Delete".to_string(), ProviderType::OpenAI);
        insert_provider(&conn, &provider).expect("Insert should succeed");

        delete_provider(&conn, &provider.id).expect("Delete should succeed");

        let loaded = load_providers(&conn).expect("Load should succeed");
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_provider_with_generation_params() {
        let conn = setup_test_db();
        let mut provider = ProviderConfig::new("Test".to_string(), ProviderType::OpenAI);
        provider.default_params = Some(crate::domain::provider::GenerationParams {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(4096),
            frequency_penalty: None,
            presence_penalty: None,
        });
        insert_provider(&conn, &provider).expect("Insert should succeed");

        let loaded = load_provider(&conn, &provider.id)
            .expect("Load should succeed")
            .expect("Provider should exist");
        assert!(loaded.default_params.is_some());
        let params = loaded.default_params.unwrap();
        assert_eq!(params.temperature, Some(0.7));
        assert_eq!(params.max_tokens, Some(4096));
    }
}
