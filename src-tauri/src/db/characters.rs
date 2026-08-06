use crate::db::models::row_to_character;
use crate::domain::character::CharacterCardV2;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Inserts a new character into the database.
/// JSON fields (alternate_greetings, tags, extensions, character_book) are serialized to TEXT.
/// Timestamps are stored as RFC 3339 TEXT.
pub fn insert_character(conn: &Connection, character: &CharacterCardV2) -> Result<()> {
    let alternate_greetings_str = serde_json::to_string(&character.alternate_greetings)?;
    let tags_str = serde_json::to_string(&character.tags)?;
    let extensions_str = serde_json::to_string(&character.extensions)?;
    let character_book_str = serde_json::to_string(&character.character_book)?;

    conn.prepare_cached(
        "INSERT INTO characters (id, name, description, personality, scenario, first_mes,
         mes_example, system_prompt, post_history_instructions, alternate_greetings, tags,
         creator, character_version, avatar_path, extensions, creator_notes, character_book,
         default_persona_id, default_preset_id, lorebook_scan_depth, lorebook_recursive_depth,
         created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                 ?18, ?19, ?20, ?21, ?22, ?23)",
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
        character.creator_notes,
        character_book_str,
        character.default_persona_id,
        character.default_preset_id,
        character.lorebook_scan_depth,
        character.lorebook_recursive_depth,
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
         character_version, avatar_path, extensions, creator_notes, character_book,
         default_persona_id, default_preset_id, lorebook_scan_depth, lorebook_recursive_depth,
         created_at, updated_at
         FROM characters ORDER BY updated_at DESC",
    )?;
    let characters = stmt.query_map([], row_to_character)?;
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
         character_version, avatar_path, extensions, creator_notes, character_book,
         default_persona_id, default_preset_id, lorebook_scan_depth, lorebook_recursive_depth,
         created_at, updated_at
         FROM characters WHERE id = ?1",
    )?;
    let mut characters = stmt.query_map(params![id], row_to_character)?;
    match characters.next() {
        Some(c) => Ok(Some(c?)),
        None => Ok(None),
    }
}

/// Updates an existing character in the database.
/// All mutable fields are updated; id is used in the WHERE clause.
pub fn update_character(conn: &Connection, character: &CharacterCardV2) -> Result<()> {
    let alternate_greetings_str = serde_json::to_string(&character.alternate_greetings)?;
    let tags_str = serde_json::to_string(&character.tags)?;
    let extensions_str = serde_json::to_string(&character.extensions)?;
    let character_book_str = serde_json::to_string(&character.character_book)?;

    conn.prepare_cached(
        "UPDATE characters SET name = ?1, description = ?2, personality = ?3, scenario = ?4,
         first_mes = ?5, mes_example = ?6, system_prompt = ?7, post_history_instructions = ?8,
         alternate_greetings = ?9, tags = ?10, creator = ?11, character_version = ?12,
         avatar_path = ?13, extensions = ?14, creator_notes = ?15, character_book = ?16,
         default_persona_id = ?17, default_preset_id = ?18, lorebook_scan_depth = ?19,
         lorebook_recursive_depth = ?20, updated_at = ?21 WHERE id = ?22",
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
        character.creator_notes,
        character_book_str,
        character.default_persona_id,
        character.default_preset_id,
        character.lorebook_scan_depth,
        character.lorebook_recursive_depth,
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
/// Returns results ranked by relevance. Empty or whitespace-only queries return an empty result.
/// FTS5 special characters are escaped to prevent query injection.
pub fn search_characters(conn: &Connection, query: &str) -> Result<Vec<CharacterCardV2>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let sanitized = sanitize_fts_query(trimmed);

    let mut stmt = conn.prepare_cached(
        "SELECT c.id, c.name, c.description, c.personality, c.scenario, c.first_mes,
         c.mes_example, c.system_prompt, c.post_history_instructions, c.alternate_greetings,
         c.tags, c.creator, c.character_version, c.avatar_path, c.extensions,
         c.creator_notes, c.character_book, c.default_persona_id, c.default_preset_id,
         c.lorebook_scan_depth, c.lorebook_recursive_depth, c.created_at, c.updated_at
         FROM characters c
         JOIN characters_fts f ON c.rowid = f.rowid
         WHERE characters_fts MATCH ?1
         ORDER BY rank",
    )?;
    let characters = stmt.query_map(params![sanitized], row_to_character)?;
    let mut result = Vec::new();
    for character in characters {
        result.push(character?);
    }
    Ok(result)
}

/// Sanitizes a query string for FTS5 MATCH.
/// Wraps the query in double quotes after escaping internal quotes,
/// preventing FTS5 operator injection.
pub(crate) fn sanitize_fts_query(query: &str) -> String {
    // Escape double quotes by doubling them
    let escaped = query.replace('"', "\"\"");
    // Wrap in double quotes for phrase matching
    format!("\"{}\"", escaped)
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

    fn make_test_character(name: &str) -> CharacterCardV2 {
        CharacterCardV2::new(name.to_string())
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
        assert!(
            loaded.is_none(),
            "Should return None for nonexistent character"
        );
    }

    #[test]
    fn test_update_character() {
        let conn = setup_test_db();
        let mut character = make_test_character("Original");
        insert_character(&conn, &character).expect("Insert should succeed");

        character.name = "Updated".to_string();
        character.description = "A brave hero".to_string();
        character.tags = vec!["fantasy".to_string(), "warrior".to_string()];
        character.updated_at = jiff::Timestamp::now();
        update_character(&conn, &character).expect("Update should succeed");

        let loaded = load_character(&conn, &character.id)
            .expect("Load should succeed")
            .expect("Character should exist");
        assert_eq!(loaded.name, "Updated");
        assert_eq!(loaded.description, "A brave hero");
        assert_eq!(loaded.tags, vec!["fantasy", "warrior"]);
    }

    #[test]
    fn test_update_character_with_default_preset_id() {
        let conn = setup_test_db();
        let mut character = make_test_character("PresetTest");
        insert_character(&conn, &character).expect("Insert should succeed");

        character.default_preset_id = Some("builtin-creative".to_string());
        character.updated_at = jiff::Timestamp::now();
        update_character(&conn, &character).expect("Update should succeed");

        let loaded = load_character(&conn, &character.id)
            .expect("Load should succeed")
            .expect("Character should exist");
        assert_eq!(
            loaded.default_preset_id,
            Some("builtin-creative".to_string())
        );
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
        assert_eq!(
            results.len(),
            1,
            "Should find 1 character matching 'fantasy'"
        );
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
    fn test_search_characters_empty_query() {
        let conn = setup_test_db();
        let c1 = make_test_character("Alice");
        insert_character(&conn, &c1).expect("Insert c1");

        let results = search_characters(&conn, "").expect("Search should succeed");
        assert!(
            results.is_empty(),
            "Empty query should return empty results"
        );

        let results = search_characters(&conn, "   ").expect("Search should succeed");
        assert!(
            results.is_empty(),
            "Whitespace query should return empty results"
        );
    }

    #[test]
    fn test_fts_stays_in_sync_after_update() {
        let conn = setup_test_db();

        let mut c = make_test_character("Test Char");
        c.tags = vec!["original".to_string()];
        insert_character(&conn, &c).expect("Insert");

        // Search for "updated" — should not find
        let results = search_characters(&conn, "updated").expect("Search");
        assert!(
            results.is_empty(),
            "Should not find 'updated' before update"
        );

        // Update tags to include "updated"
        c.tags = vec!["updated".to_string()];
        c.updated_at = jiff::Timestamp::now();
        update_character(&conn, &c).expect("Update");

        // Now search should find it
        let results = search_characters(&conn, "updated").expect("Search after update");
        assert_eq!(
            results.len(),
            1,
            "Should find character after updating tags"
        );
    }

    #[test]
    fn test_fts_stays_in_sync_after_delete() {
        let conn = setup_test_db();

        let mut c = make_test_character("DeleteTest Char");
        c.description = "A character to be deleted".to_string();
        insert_character(&conn, &c).expect("Insert");

        let results = search_characters(&conn, "DeleteTest").expect("Search");
        assert_eq!(results.len(), 1);

        delete_character(&conn, &c.id).expect("Delete");

        let results = search_characters(&conn, "DeleteTest").expect("Search after delete");
        assert!(results.is_empty(), "FTS should be in sync after deletion");
    }

    #[test]
    fn test_sanitize_fts_query() {
        assert_eq!(sanitize_fts_query("hello world"), "\"hello world\"");
        assert_eq!(sanitize_fts_query("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(sanitize_fts_query("test"), "\"test\"");
    }

    #[test]
    fn test_insert_character_with_json_fields() {
        let conn = setup_test_db();
        let mut character = make_test_character("JSON Char");
        character.alternate_greetings = vec!["Hi there!".to_string(), "Greetings!".to_string()];
        character.tags = vec!["fantasy".to_string(), "hero".to_string()];
        character.extensions = serde_json::json!({"custom_field": "value"});
        character.character_book = serde_json::json!({"entries": [], "name": "test"});
        insert_character(&conn, &character).expect("Insert should succeed");

        let loaded = load_character(&conn, &character.id)
            .expect("Load should succeed")
            .expect("Character should exist");
        assert_eq!(
            loaded.alternate_greetings,
            vec!["Hi there!".to_string(), "Greetings!".to_string()]
        );
        assert_eq!(loaded.tags, vec!["fantasy", "hero"]);
        assert_eq!(
            loaded.extensions,
            serde_json::json!({"custom_field": "value"})
        );
        assert_eq!(
            loaded.character_book,
            serde_json::json!({"entries": [], "name": "test"})
        );
    }
}
