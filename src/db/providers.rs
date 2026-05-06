use crate::db::models::row_to_provider;
use crate::domain::provider::ProviderConfig;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Inserts a new provider into the database.
pub fn insert_provider(conn: &Connection, provider: &ProviderConfig) -> Result<()> {
    let provider_type_str = serde_json::to_string(&provider.provider_type)?;
    conn.prepare_cached(
        "INSERT INTO providers (id, name, provider_type, base_url, model_name, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?
    .execute(params![
        provider.id,
        provider.name,
        provider_type_str,
        provider.base_url,
        provider.model_name,
        provider.enabled as i32,
        provider.created_at.to_string(),
        provider.updated_at.to_string(),
    ])?;
    Ok(())
}

/// Loads all providers from the database.
pub fn load_providers(conn: &Connection) -> Result<Vec<ProviderConfig>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, provider_type, base_url, model_name, enabled, created_at, updated_at FROM providers"
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
        "SELECT id, name, provider_type, base_url, model_name, enabled, created_at, updated_at FROM providers WHERE id = ?1"
    )?;
    let mut providers = stmt.query_map(params![id], |row| row_to_provider(row))?;
    match providers.next() {
        Some(p) => Ok(Some(p?)),
        None => Ok(None),
    }
}

/// Updates an existing provider in the database.
pub fn update_provider(conn: &Connection, provider: &ProviderConfig) -> Result<()> {
    let provider_type_str = serde_json::to_string(&provider.provider_type)?;
    conn.prepare_cached(
        "UPDATE providers SET name = ?1, provider_type = ?2, base_url = ?3, model_name = ?4, enabled = ?5,
         updated_at = ?6 WHERE id = ?7"
    )?
    .execute(params![
        provider.name,
        provider_type_str,
        provider.base_url,
        provider.model_name,
        provider.enabled as i32,
        provider.updated_at.to_string(),
        provider.id,
    ])?;
    Ok(())
}

/// Deletes a provider from the database by ID.
pub fn delete_provider(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM providers WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

/// Saves an app setting (insert or update).
pub fn save_app_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2"
    )?
    .execute(params![key, value])?;
    Ok(())
}

/// Loads an app setting by key.
pub fn load_app_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT value FROM app_settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
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

    fn make_test_provider(name: &str, provider_type: ProviderType) -> ProviderConfig {
        ProviderConfig::new(name.to_string(), provider_type)
    }

    #[test]
    fn test_insert_and_load_provider() {
        let conn = setup_test_db();
        let provider = make_test_provider("My OpenAI", ProviderType::OpenAI);
        insert_provider(&conn, &provider).expect("Insert should succeed");

        let loaded = load_providers(&conn).expect("Load should succeed");
        assert_eq!(loaded.len(), 1, "Should have 1 provider");
        assert_eq!(loaded[0].id, provider.id);
        assert_eq!(loaded[0].name, "My OpenAI");
        assert_eq!(loaded[0].provider_type, ProviderType::OpenAI);
        assert_eq!(loaded[0].base_url, Some("https://api.openai.com/v1".to_string()));
        assert!(loaded[0].enabled);
    }

    #[test]
    fn test_load_provider_by_id() {
        let conn = setup_test_db();
        let provider = make_test_provider("My Anthropic", ProviderType::Anthropic);
        insert_provider(&conn, &provider).expect("Insert should succeed");

        let loaded = load_provider(&conn, &provider.id).expect("Load should succeed");
        assert!(loaded.is_some(), "Should find provider by ID");
        assert_eq!(loaded.unwrap().name, "My Anthropic");
    }

    #[test]
    fn test_load_provider_not_found() {
        let conn = setup_test_db();
        let loaded = load_provider(&conn, "nonexistent-id").expect("Load should succeed");
        assert!(loaded.is_none(), "Should return None for nonexistent provider");
    }

    #[test]
    fn test_delete_provider() {
        let conn = setup_test_db();
        let provider = make_test_provider("To Delete", ProviderType::Groq);
        insert_provider(&conn, &provider).expect("Insert should succeed");

        delete_provider(&conn, &provider.id).expect("Delete should succeed");
        let loaded = load_providers(&conn).expect("Load should succeed");
        assert!(loaded.is_empty(), "Provider should be deleted");
    }

    #[test]
    fn test_update_provider() {
        let conn = setup_test_db();
        let mut provider = make_test_provider("Original", ProviderType::OpenAI);
        insert_provider(&conn, &provider).expect("Insert should succeed");

        provider.name = "Updated".to_string();
        provider.enabled = false;
        update_provider(&conn, &provider).expect("Update should succeed");

        let loaded = load_provider(&conn, &provider.id)
            .expect("Load should succeed")
            .expect("Provider should exist");
        assert_eq!(loaded.name, "Updated");
        assert!(!loaded.enabled);
    }

    #[test]
    fn test_insert_multiple_providers() {
        let conn = setup_test_db();
        let p1 = make_test_provider("OpenAI", ProviderType::OpenAI);
        let p2 = make_test_provider("Ollama", ProviderType::Ollama);
        let p3 = make_test_provider("Anthropic", ProviderType::Anthropic);

        insert_provider(&conn, &p1).expect("Insert p1");
        insert_provider(&conn, &p2).expect("Insert p2");
        insert_provider(&conn, &p3).expect("Insert p3");

        let loaded = load_providers(&conn).expect("Load should succeed");
        assert_eq!(loaded.len(), 3, "Should have 3 providers");
    }

    #[test]
    fn test_app_setting_save_and_load() {
        let conn = setup_test_db();
        save_app_setting(&conn, "theme", "dark").expect("Save should succeed");
        let value = load_app_setting(&conn, "theme").expect("Load should succeed");
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn test_app_setting_not_found() {
        let conn = setup_test_db();
        let value = load_app_setting(&conn, "nonexistent").expect("Load should succeed");
        assert!(value.is_none(), "Should return None for nonexistent setting");
    }

    #[test]
    fn test_app_setting_update() {
        let conn = setup_test_db();
        save_app_setting(&conn, "theme", "dark").expect("Save");
        save_app_setting(&conn, "theme", "light").expect("Update");
        let value = load_app_setting(&conn, "theme").expect("Load");
        assert_eq!(value, Some("light".to_string()), "Setting should be updated");
    }

    #[test]
    fn test_ollama_provider_roundtrip() {
        let conn = setup_test_db();
        let provider = make_test_provider("Local Ollama", ProviderType::Ollama);
        insert_provider(&conn, &provider).expect("Insert");

        let loaded = load_providers(&conn).expect("Load");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].provider_type, ProviderType::Ollama);
        assert_eq!(loaded[0].base_url, Some("http://localhost:11434".to_string()));
    }
}
