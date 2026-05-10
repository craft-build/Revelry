use crate::db::models::row_to_persona;
use crate::domain::persona::Persona;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

/// Inserts a new persona into the database.
/// Tags are serialized as a JSON array in the TEXT column.
pub fn insert_persona(conn: &Connection, persona: &Persona) -> Result<()> {
    let tags_str = serde_json::to_string(&persona.tags)?;
    conn.prepare_cached(
        "INSERT INTO personas (id, name, appearance, personality, background, role_description,
         relationships, speech_style, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    )?
    .execute(params![
        persona.id,
        persona.name,
        persona.appearance,
        persona.personality,
        persona.background,
        persona.role_description,
        persona.relationships,
        persona.speech_style,
        tags_str,
        persona.created_at.to_string(),
        persona.updated_at.to_string(),
    ])?;
    Ok(())
}

/// Loads all personas from the database, ordered by updated_at descending.
pub fn load_personas(conn: &Connection) -> Result<Vec<Persona>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, appearance, personality, background, role_description,
         relationships, speech_style, tags, created_at, updated_at
         FROM personas ORDER BY updated_at DESC"
    )?;
    let personas = stmt.query_map([], |row| row_to_persona(row))?;
    let mut result = Vec::new();
    for persona in personas {
        result.push(persona?);
    }
    Ok(result)
}

/// Loads a single persona by ID.
pub fn load_persona(conn: &Connection, id: &str) -> Result<Option<Persona>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, appearance, personality, background, role_description,
         relationships, speech_style, tags, created_at, updated_at
         FROM personas WHERE id = ?1"
    )?;
    let mut personas = stmt.query_map(params![id], |row| row_to_persona(row))?;
    match personas.next() {
        Some(p) => Ok(Some(p?)),
        None => Ok(None),
    }
}

/// Updates an existing persona in the database.
pub fn update_persona(conn: &Connection, persona: &Persona) -> Result<()> {
    let tags_str = serde_json::to_string(&persona.tags)?;
    conn.prepare_cached(
        "UPDATE personas SET name = ?1, appearance = ?2, personality = ?3, background = ?4,
         role_description = ?5, relationships = ?6, speech_style = ?7, tags = ?8,
         updated_at = ?9 WHERE id = ?10"
    )?
    .execute(params![
        persona.name,
        persona.appearance,
        persona.personality,
        persona.background,
        persona.role_description,
        persona.relationships,
        persona.speech_style,
        tags_str,
        persona.updated_at.to_string(),
        persona.id,
    ])?;
    Ok(())
}

/// Deletes a persona from the database by ID.
pub fn delete_persona(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM personas WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

/// Gets the global default persona ID from app settings.
pub fn get_global_default_persona_id(conn: &Connection) -> Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT value FROM app_settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params!["default_persona_id"], |row| row.get::<_, String>(0))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Sets the global default persona ID in app settings.
pub fn set_global_default_persona_id(conn: &Connection, id: Option<&str>) -> Result<()> {
    match id {
        Some(persona_id) => {
            conn.prepare_cached(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2"
            )?
            .execute(params!["default_persona_id", persona_id])?;
        }
        None => {
            conn.prepare_cached("DELETE FROM app_settings WHERE key = ?1")?
                .execute(params!["default_persona_id"])?;
        }
    }
    Ok(())
}

/// Resolves the persona for a conversation using the assignment hierarchy:
/// 1. Conversation-level override (creator_persona_id)
/// 2. Per-character default (from character.default_persona_id)
/// 3. Global default (from app_settings)
/// 4. None
///
/// The `personas` table is queried for the actual Persona data.
pub fn resolve_persona_for_conversation(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Option<Persona>> {
    // Step 1: Check conversation
    let mut stmt = conn.prepare_cached(
        "SELECT creator_persona_id FROM conversations WHERE id = ?1"
    )?;
    let conv_persona_id: Option<String> = stmt
        .query_row(params![conversation_id], |row| row.get(0))
        .optional()?;

    if let Some(id) = conv_persona_id {
        return load_persona(conn, &id);
    }

    // Step 2: Check character default
    let mut stmt = conn.prepare_cached(
        "SELECT c.default_persona_id FROM characters c
         JOIN conversations conv ON c.id = conv.character_id
         WHERE conv.id = ?1"
    )?;
    let char_persona_id: Option<String> = stmt
        .query_row(params![conversation_id], |row| row.get(0))
        .optional()?;

    if let Some(id) = char_persona_id {
        return load_persona(conn, &id);
    }

    // Step 3: Check global default
    if let Some(id) = get_global_default_persona_id(conn)? {
        return load_persona(conn, &id);
    }

    // Step 4: No persona
    Ok(None)
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

    fn make_test_persona(name: &str, personality: &str) -> Persona {
        Persona::new(name.to_string(), personality.to_string())
    }

    #[test]
    fn test_insert_and_load_personas() {
        let conn = setup_test_db();
        let persona = make_test_persona("Test Persona", "Friendly");
        insert_persona(&conn, &persona).expect("Insert should succeed");

        let loaded = load_personas(&conn).expect("Load should succeed");
        assert_eq!(loaded.len(), 1, "Should have 1 persona");
        assert_eq!(loaded[0].id, persona.id);
        assert_eq!(loaded[0].name, "Test Persona");
        assert_eq!(loaded[0].personality, "Friendly");
    }

    #[test]
    fn test_load_persona_by_id() {
        let conn = setup_test_db();
        let persona = make_test_persona("FindByID", "Smart");
        insert_persona(&conn, &persona).expect("Insert should succeed");

        let loaded = load_persona(&conn, &persona.id).expect("Load should succeed");
        assert!(loaded.is_some(), "Should find persona by ID");
        assert_eq!(loaded.unwrap().name, "FindByID");
    }

    #[test]
    fn test_load_persona_not_found() {
        let conn = setup_test_db();
        let loaded = load_persona(&conn, "nonexistent-id").expect("Load should succeed");
        assert!(loaded.is_none(), "Should return None for nonexistent persona");
    }

    #[test]
    fn test_update_persona() {
        let conn = setup_test_db();
        let mut persona = make_test_persona("Original", "Original personality");
        insert_persona(&conn, &persona).expect("Insert should succeed");

        persona.name = "Updated".to_string();
        persona.personality = "Updated personality".to_string();
        persona.appearance = "Blue hair".to_string();
        persona.tags = vec!["fantasy".to_string(), "mage".to_string()];
        persona.updated_at = chrono::Local::now().naive_local();
        update_persona(&conn, &persona).expect("Update should succeed");

        let loaded = load_persona(&conn, &persona.id)
            .expect("Load should succeed")
            .expect("Persona should exist");
        assert_eq!(loaded.name, "Updated");
        assert_eq!(loaded.personality, "Updated personality");
        assert_eq!(loaded.appearance, "Blue hair");
        assert_eq!(loaded.tags, vec!["fantasy", "mage"]);
    }

    #[test]
    fn test_delete_persona() {
        let conn = setup_test_db();
        let persona = make_test_persona("To Delete", "Gone");
        insert_persona(&conn, &persona).expect("Insert should succeed");

        delete_persona(&conn, &persona.id).expect("Delete should succeed");
        let loaded = load_personas(&conn).expect("Load should succeed");
        assert!(loaded.is_empty(), "Persona should be deleted");
    }

    #[test]
    fn test_tags_serialization() {
        let conn = setup_test_db();
        let mut persona = make_test_persona("Tagged", "Has tags");
        persona.tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        insert_persona(&conn, &persona).expect("Insert should succeed");

        let loaded = load_persona(&conn, &persona.id)
            .expect("Load should succeed")
            .expect("Persona should exist");
        assert_eq!(loaded.tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_get_global_default_none() {
        let conn = setup_test_db();
        let result = get_global_default_persona_id(&conn).expect("Query should succeed");
        assert!(result.is_none(), "Should return None when no default is set");
    }

    #[test]
    fn test_set_and_get_global_default() {
        let conn = setup_test_db();
        let persona = make_test_persona("Default", "The default persona");
        insert_persona(&conn, &persona).expect("Insert should succeed");

        // Set as global default
        set_global_default_persona_id(&conn, Some(&persona.id)).expect("Set should succeed");
        let loaded = get_global_default_persona_id(&conn).expect("Query should succeed");
        assert_eq!(loaded, Some(persona.id.clone()));

        // Clear global default
        set_global_default_persona_id(&conn, None).expect("Clear should succeed");
        let cleared = get_global_default_persona_id(&conn).expect("Query should succeed");
        assert!(cleared.is_none(), "Should return None after clearing");
    }
}
