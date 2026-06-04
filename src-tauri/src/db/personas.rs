use crate::db::models::row_to_persona;
use crate::domain::persona::Persona;
use anyhow::Result;
use jiff::Timestamp;
use rusqlite::{params, Connection};

/// Inserts a new persona into the database.
/// Tags are serialized as a JSON array in the TEXT column.
/// Timestamps are stored as RFC 3339 TEXT.
pub fn insert_persona(conn: &Connection, persona: &Persona) -> Result<()> {
    let tags_str = serde_json::to_string(&persona.tags)?;
    conn.prepare_cached(
        "INSERT INTO personas (id, name, appearance, personality, background, role_description,
         relationships, speech_style, tags, avatar_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
        persona.avatar_path,
        persona.created_at.to_string(),
        persona.updated_at.to_string(),
    ])?;
    Ok(())
}

/// Loads all personas from the database, ordered by updated_at descending.
pub fn load_personas(conn: &Connection) -> Result<Vec<Persona>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, appearance, personality, background, role_description,
         relationships, speech_style, tags, avatar_path, created_at, updated_at
         FROM personas ORDER BY updated_at DESC",
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
         relationships, speech_style, tags, avatar_path, created_at, updated_at
         FROM personas WHERE id = ?1",
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
         avatar_path = ?9, updated_at = ?10 WHERE id = ?11",
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
        persona.avatar_path,
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

/// Searches personas using SQL LIKE on name and personality columns.
/// Returns all personas where either column contains the query string.
pub fn search_personas(conn: &Connection, query: &str) -> Result<Vec<Persona>> {
    let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, appearance, personality, background, role_description,
         relationships, speech_style, tags, avatar_path, created_at, updated_at
         FROM personas
         WHERE name LIKE ?1 ESCAPE '\\' OR personality LIKE ?1 ESCAPE '\\'
         ORDER BY updated_at DESC",
    )?;
    let personas = stmt.query_map(params![pattern], |row| row_to_persona(row))?;
    let mut result = Vec::new();
    for persona in personas {
        result.push(persona?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

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
        assert!(
            loaded.is_none(),
            "Should return None for nonexistent persona"
        );
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
        persona.updated_at = Timestamp::now();
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
    fn test_search_personas_by_name() {
        let conn = setup_test_db();
        let p1 = make_test_persona("Alice the Brave", "Courageous and bold");
        let p2 = make_test_persona("Bob the Wise", "Quiet and thoughtful");
        insert_persona(&conn, &p1).expect("Insert p1");
        insert_persona(&conn, &p2).expect("Insert p2");

        let results = search_personas(&conn, "Alice").expect("Search should succeed");
        assert_eq!(results.len(), 1, "Should find 1 persona matching 'Alice'");
        assert_eq!(results[0].name, "Alice the Brave");
    }

    #[test]
    fn test_search_personas_by_personality() {
        let conn = setup_test_db();
        let p1 = make_test_persona("Alice", "Courageous and bold");
        let p2 = make_test_persona("Bob", "Quiet and thoughtful");
        insert_persona(&conn, &p1).expect("Insert p1");
        insert_persona(&conn, &p2).expect("Insert p2");

        let results = search_personas(&conn, "Courageous").expect("Search should succeed");
        assert_eq!(
            results.len(),
            1,
            "Should find 1 persona matching 'Courageous'"
        );
        assert_eq!(results[0].name, "Alice");
    }

    #[test]
    fn test_search_personas_no_match() {
        let conn = setup_test_db();
        let p1 = make_test_persona("Alice", "Brave");
        insert_persona(&conn, &p1).expect("Insert p1");

        let results = search_personas(&conn, "nonexistent").expect("Search should succeed");
        assert!(results.is_empty(), "Should find no personas");
    }
}
