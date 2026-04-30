use crate::db::models::row_to_character;
use crate::domain::character::CharacterCardV2;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Inserts a new character into the database.
/// JSON fields (alternate_greetings, tags, extensions) are serialized to TEXT.
pub fn insert_character(conn: &Connection, character: &CharacterCardV2) -> Result<()> {
    let alternate_greetings_str = serde_json::to_string(&character.alternate_greetings)?;
    let tags_str = serde_json::to_string(&character.tags)?;
    let extensions_str = serde_json::to_string(&character.extensions)?;

    conn.prepare_cached(
        "INSERT INTO characters (id, name, description, personality, scenario, first_mes,
         mes_example, system_prompt, post_history_instructions, alternate_greetings, tags,
         creator, character_version, avatar_path, extensions, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"
    )?
    .execute(params![
        character.id,
        character.name,
        character.description,
        character.personality,
        character.scenario,
        character.first_mes,
        character.mes_example,
        character.system_prompt,
        character.post_history_instructions,
        alternate_greetings_str,
        tags_str,
        character.creator,
        character.character_version,
        character.avatar_path,
        extensions_str,
        character.created_at.to_string(),
        character.updated_at.to_string(),
    ])?;
    Ok(())
}

/// Loads all characters from the database, ordered by updated_at descending.
pub fn load_characters(conn: &Connection) -> Result<Vec<CharacterCardV2>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, personality, scenario, first_mes, mes_example,
         system_prompt, post_history_instructions, alternate_greetings, tags, creator,
         character_version, avatar_path, extensions, created_at, updated_at
         FROM characters ORDER BY updated_at DESC"
    )?;
    let characters = stmt.query_map([], |row| row_to_character(row))?;
    let mut result = Vec::new();
    for character in characters {
        result.push(character?);
    }
    Ok(result)
}

/// Loads a single character by ID.
pub fn load_character(conn: &Connection, id: &str) -> Result<Option<CharacterCardV2>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, personality, scenario, first_mes, mes_example,
         system_prompt, post_history_instructions, alternate_greetings, tags, creator,
         character_version, avatar_path, extensions, created_at, updated_at
         FROM characters WHERE id = ?1"
    )?;
    let mut characters = stmt.query_map(params![id], |row| row_to_character(row))?;
    match characters.next() {
        Some(c) => Ok(Some(c?)),
        None => Ok(None),
    }
}

/// Updates an existing character in the database.
pub fn update_character(conn: &Connection, character: &CharacterCardV2) -> Result<()> {
    let alternate_greetings_str = serde_json::to_string(&character.alternate_greetings)?;
    let tags_str = serde_json::to_string(&character.tags)?;
    let extensions_str = serde_json::to_string(&character.extensions)?;

    conn.prepare_cached(
        "UPDATE characters SET name = ?1, description = ?2, personality = ?3, scenario = ?4,
         first_mes = ?5, mes_example = ?6, system_prompt = ?7, post_history_instructions = ?8,
         alternate_greetings = ?9, tags = ?10, creator = ?11, character_version = ?12,
         avatar_path = ?13, extensions = ?14, updated_at = ?15 WHERE id = ?16"
    )?
    .execute(params![
        character.name,
        character.description,
        character.personality,
        character.scenario,
        character.first_mes,
        character.mes_example,
        character.system_prompt,
        character.post_history_instructions,
        alternate_greetings_str,
        tags_str,
        character.creator,
        character.character_version,
        character.avatar_path,
        extensions_str,
        character.updated_at.to_string(),
        character.id,
    ])?;
    Ok(())
}

/// Deletes a character from the database by ID.
pub fn delete_character(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM characters WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

/// Searches characters using FTS5 full-text search.
/// Returns results ranked by relevance. Empty queries return an empty result.
/// FTS5 special characters are escaped to prevent query injection (T-02-02).
pub fn search_characters(conn: &Connection, query: &str) -> Result<Vec<CharacterCardV2>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // Sanitize FTS5 query: escape special characters to prevent injection (T-02-02)
    let sanitized = sanitize_fts_query(trimmed);

    let mut stmt = conn.prepare_cached(
        "SELECT c.id, c.name, c.description, c.personality, c.scenario, c.first_mes,
         c.mes_example, c.system_prompt, c.post_history_instructions, c.alternate_greetings,
         c.tags, c.creator, c.character_version, c.avatar_path, c.extensions, c.created_at,
         c.updated_at
         FROM characters c
         JOIN characters_fts f ON c.rowid = f.rowid
         WHERE characters_fts MATCH ?1
         ORDER BY rank"
    )?;
    let characters = stmt.query_map(params![sanitized], |row| row_to_character(row))?;
    let mut result = Vec::new();
    for character in characters {
        result.push(character?);
    }
    Ok(result)
}

/// Sanitizes a query string for FTS5 MATCH.
/// Wraps the query in double quotes after escaping internal quotes,
/// preventing FTS5 syntax injection (T-02-02).
fn sanitize_fts_query(query: &str) -> String {
    // Escape double quotes by doubling them
    let escaped = query.replace('"', "\"\"");
    // Wrap in double quotes for phrase matching — prevents FTS5 operator injection
    format!("\"{}\"", escaped)
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

    fn make_test_character(name: &str) -> CharacterCardV2 {
        CharacterCardV2::new(name.to_string())
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
        assert!(result, "characters table should exist after migration 003");
    }

    #[test]
    fn test_insert_and_load_characters() {
        let conn = setup_test_db();
        let character = make_test_character("Test Character");
        insert_character(&conn, &character).expect("Insert should succeed");

        let loaded = load_characters(&conn).expect("Load should succeed");
        assert_eq!(loaded.len(), 1, "Should have 1 character");
        assert_eq!(loaded[0].id, character.id);
        assert_eq!(loaded[0].name, "Test Character");
        assert_eq!(loaded[0].character_version, "2.0");
        assert_eq!(loaded[0].creator, "Unknown");
        assert!(loaded[0].alternate_greetings.is_empty());
        assert!(loaded[0].tags.is_empty());
    }

    #[test]
    fn test_load_character_by_id() {
        let conn = setup_test_db();
        let character = make_test_character("FindByID");
        insert_character(&conn, &character).expect("Insert should succeed");

        let loaded = load_character(&conn, &character.id).expect("Load should succeed");
        assert!(loaded.is_some(), "Should find character by ID");
        assert_eq!(loaded.unwrap().name, "FindByID");
    }

    #[test]
    fn test_load_character_not_found() {
        let conn = setup_test_db();
        let loaded = load_character(&conn, "nonexistent-id").expect("Load should succeed");
        assert!(loaded.is_none(), "Should return None for nonexistent character");
    }

    #[test]
    fn test_update_character() {
        let conn = setup_test_db();
        let mut character = make_test_character("Original");
        insert_character(&conn, &character).expect("Insert should succeed");

        character.name = "Updated".to_string();
        character.description = "A brave hero".to_string();
        character.tags = vec!["fantasy".to_string(), "warrior".to_string()];
        character.updated_at = chrono::Local::now().naive_local();
        update_character(&conn, &character).expect("Update should succeed");

        let loaded = load_character(&conn, &character.id)
            .expect("Load should succeed")
            .expect("Character should exist");
        assert_eq!(loaded.name, "Updated");
        assert_eq!(loaded.description, "A brave hero");
        assert_eq!(loaded.tags, vec!["fantasy", "warrior"]);
    }

    #[test]
    fn test_delete_character() {
        let conn = setup_test_db();
        let character = make_test_character("To Delete");
        insert_character(&conn, &character).expect("Insert should succeed");

        delete_character(&conn, &character.id).expect("Delete should succeed");
        let loaded = load_characters(&conn).expect("Load should succeed");
        assert!(loaded.is_empty(), "Character should be deleted");
    }

    #[test]
    fn test_search_characters_finds_match() {
        let conn = setup_test_db();

        let mut c1 = make_test_character("Fantasy Wizard");
        c1.description = "A powerful mage from a fantasy realm".to_string();
        c1.tags = vec!["fantasy".to_string(), "magic".to_string()];
        insert_character(&conn, &c1).expect("Insert c1");

        let mut c2 = make_test_character("Sci-Fi Pilot");
        c2.description = "A brave pilot in a futuristic setting".to_string();
        c2.tags = vec!["sci-fi".to_string()];
        insert_character(&conn, &c2).expect("Insert c2");

        let results = search_characters(&conn, "fantasy").expect("Search should succeed");
        assert_eq!(results.len(), 1, "Should find 1 character matching 'fantasy'");
        assert_eq!(results[0].name, "Fantasy Wizard");
    }

    #[test]
    fn test_search_characters_no_match() {
        let conn = setup_test_db();
        let c1 = make_test_character("Alice");
        insert_character(&conn, &c1).expect("Insert c1");

        let results = search_characters(&conn, "nonexistent").expect("Search should succeed");
        assert!(results.is_empty(), "Should find no characters");
    }

    #[test]
    fn test_fts_stays_in_sync_after_update() {
        let conn = setup_test_db();

        let mut c = make_test_character("Test Char");
        c.tags = vec!["original".to_string()];
        insert_character(&conn, &c).expect("Insert");

        // Search for "updated" — should not find
        let results = search_characters(&conn, "updated").expect("Search");
        assert!(results.is_empty(), "Should not find 'updated' before update");

        // Update tags to include "updated"
        c.tags = vec!["updated".to_string()];
        c.updated_at = chrono::Local::now().naive_local();
        update_character(&conn, &c).expect("Update");

        // Now search should find it
        let results = search_characters(&conn, "updated").expect("Search after update");
        assert_eq!(results.len(), 1, "Should find character after updating tags");
    }
}
