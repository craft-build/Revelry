use crate::db::models::row_to_lorebook_entry;
use crate::domain::lorebook::LorebookEntry;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Inserts a new lorebook entry into the database.
/// Keywords and secondary_terms are serialized as JSON arrays in TEXT columns.
/// InsertionPosition is stored as TEXT ("before_char" / "after_char").
pub fn insert_lorebook_entry(conn: &Connection, entry: &LorebookEntry) -> Result<()> {
    let keywords_str = serde_json::to_string(&entry.keywords)?;
    let secondary_terms_str = serde_json::to_string(&entry.secondary_terms)?;
    let insertion_position_str = entry.insertion_position.to_string();
    conn.prepare_cached(
        "INSERT INTO lorebook_entries (id, character_id, name, keywords, secondary_terms,
         content, insertion_position, depth_from_bottom, enabled, token_budget, sort_order,
         created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
    )?
    .execute(params![
        entry.id,
        entry.character_id,
        entry.name,
        keywords_str,
        secondary_terms_str,
        entry.content,
        insertion_position_str,
        entry.depth_from_bottom,
        entry.enabled as i32,
        entry.token_budget,
        entry.sort_order,
        entry.created_at.to_string(),
        entry.updated_at.to_string(),
    ])?;
    Ok(())
}

/// Loads all lorebook entries for a character, ordered by sort_order.
pub fn load_lorebook_entries(conn: &Connection, character_id: &str) -> Result<Vec<LorebookEntry>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, name, keywords, secondary_terms,
         content, insertion_position, depth_from_bottom, enabled, token_budget, sort_order,
         created_at, updated_at
         FROM lorebook_entries WHERE character_id = ?1 ORDER BY sort_order"
    )?;
    let entries = stmt.query_map(params![character_id], |row| row_to_lorebook_entry(row))?;
    let mut result = Vec::new();
    for entry in entries {
        result.push(entry?);
    }
    Ok(result)
}

/// Loads a single lorebook entry by ID.
pub fn load_lorebook_entry(conn: &Connection, id: &str) -> Result<Option<LorebookEntry>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, name, keywords, secondary_terms,
         content, insertion_position, depth_from_bottom, enabled, token_budget, sort_order,
         created_at, updated_at
         FROM lorebook_entries WHERE id = ?1"
    )?;
    let mut entries = stmt.query_map(params![id], |row| row_to_lorebook_entry(row))?;
    match entries.next() {
        Some(e) => Ok(Some(e?)),
        None => Ok(None),
    }
}

/// Updates an existing lorebook entry in the database.
pub fn update_lorebook_entry(conn: &Connection, entry: &LorebookEntry) -> Result<()> {
    let keywords_str = serde_json::to_string(&entry.keywords)?;
    let secondary_terms_str = serde_json::to_string(&entry.secondary_terms)?;
    let insertion_position_str = entry.insertion_position.to_string();
    conn.prepare_cached(
        "UPDATE lorebook_entries SET name = ?1, keywords = ?2, secondary_terms = ?3,
         content = ?4, insertion_position = ?5, depth_from_bottom = ?6, enabled = ?7,
         token_budget = ?8, sort_order = ?9, updated_at = ?10 WHERE id = ?11"
    )?
    .execute(params![
        entry.name,
        keywords_str,
        secondary_terms_str,
        entry.content,
        insertion_position_str,
        entry.depth_from_bottom,
        entry.enabled as i32,
        entry.token_budget,
        entry.sort_order,
        entry.updated_at.to_string(),
        entry.id,
    ])?;
    Ok(())
}

/// Deletes a lorebook entry from the database by ID.
pub fn delete_lorebook_entry(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM lorebook_entries WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::domain::character::CharacterCardV2;
    use crate::domain::lorebook::InsertionPosition;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    /// Helper to insert a test character (required because lorebook_entries has FK to characters).
    fn insert_test_character(conn: &Connection) -> CharacterCardV2 {
        let character = CharacterCardV2::new("Test Character".to_string());
        crate::db::characters::insert_character(conn, &character).expect("Insert character");
        character
    }

    fn make_test_entry(character_id: &str, name: &str) -> LorebookEntry {
        LorebookEntry::new(
            character_id.to_string(),
            name.to_string(),
            vec!["keyword1".to_string()],
            "Test content".to_string(),
        )
    }

    #[test]
    fn test_insert_and_load_lorebook_entries() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);
        let entry = make_test_entry(&character.id, "Test Entry");
        insert_lorebook_entry(&conn, &entry).expect("Insert should succeed");

        let loaded = load_lorebook_entries(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 1, "Should have 1 entry");
        assert_eq!(loaded[0].id, entry.id);
        assert_eq!(loaded[0].name, "Test Entry");
        assert_eq!(loaded[0].keywords, vec!["keyword1"]);
        assert_eq!(loaded[0].content, "Test content");
    }

    #[test]
    fn test_update_lorebook_entry() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);
        let mut entry = make_test_entry(&character.id, "Original");
        insert_lorebook_entry(&conn, &entry).expect("Insert should succeed");

        entry.name = "Updated".to_string();
        entry.content = "New content".to_string();
        entry.keywords = vec!["kw1".to_string(), "kw2".to_string()];
        entry.insertion_position = InsertionPosition::BeforeChar;
        entry.depth_from_bottom = 2;
        entry.token_budget = Some(500);
        entry.updated_at = chrono::Local::now().naive_local();
        update_lorebook_entry(&conn, &entry).expect("Update should succeed");

        let loaded = load_lorebook_entry(&conn, &entry.id)
            .expect("Load should succeed")
            .expect("Entry should exist");
        assert_eq!(loaded.name, "Updated");
        assert_eq!(loaded.content, "New content");
        assert_eq!(loaded.keywords, vec!["kw1", "kw2"]);
        assert_eq!(loaded.insertion_position, InsertionPosition::BeforeChar);
        assert_eq!(loaded.depth_from_bottom, 2);
        assert_eq!(loaded.token_budget, Some(500));
    }

    #[test]
    fn test_delete_lorebook_entry() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);
        let entry = make_test_entry(&character.id, "To Delete");
        insert_lorebook_entry(&conn, &entry).expect("Insert should succeed");

        delete_lorebook_entry(&conn, &entry.id).expect("Delete should succeed");
        let loaded = load_lorebook_entries(&conn, &character.id).expect("Load should succeed");
        assert!(loaded.is_empty(), "Entry should be deleted");
    }

    #[test]
    fn test_load_entries_per_character_sorted() {
        let conn = setup_test_db();
        let char1 = insert_test_character(&conn);

        // Insert a second character
        let char2 = CharacterCardV2::new("Character 2".to_string());
        crate::db::characters::insert_character(&conn, &char2).expect("Insert char2");

        // Create entries with different sort orders for char1
        let mut entry1 = make_test_entry(&char1.id, "Entry Z");
        entry1.sort_order = 10;
        insert_lorebook_entry(&conn, &entry1).expect("Insert entry1");

        let mut entry2 = make_test_entry(&char1.id, "Entry A");
        entry2.sort_order = 1;
        insert_lorebook_entry(&conn, &entry2).expect("Insert entry2");

        // Create entry for char2 — should not appear in char1's entries
        let entry3 = make_test_entry(&char2.id, "Other Char Entry");
        insert_lorebook_entry(&conn, &entry3).expect("Insert entry3");

        // Load char1's entries — should be sorted by sort_order
        let loaded = load_lorebook_entries(&conn, &char1.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 2, "Should have 2 entries for char1");
        assert_eq!(loaded[0].name, "Entry A"); // sort_order 1
        assert_eq!(loaded[1].name, "Entry Z"); // sort_order 10
    }

    #[test]
    fn test_load_lorebook_entry_by_id() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);
        let entry = make_test_entry(&character.id, "FindByID");
        insert_lorebook_entry(&conn, &entry).expect("Insert should succeed");

        let loaded = load_lorebook_entry(&conn, &entry.id).expect("Load should succeed");
        assert!(loaded.is_some(), "Should find entry by ID");
        assert_eq!(loaded.unwrap().name, "FindByID");
    }

    #[test]
    fn test_load_lorebook_entry_not_found() {
        let conn = setup_test_db();
        let loaded = load_lorebook_entry(&conn, "nonexistent-id").expect("Load should succeed");
        assert!(loaded.is_none(), "Should return None for nonexistent entry");
    }

    #[test]
    fn test_secondary_terms_serialization() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);
        let mut entry = make_test_entry(&character.id, "Secondary Terms");
        entry.secondary_terms = vec!["sec1".to_string(), "sec2".to_string()];
        insert_lorebook_entry(&conn, &entry).expect("Insert should succeed");

        let loaded = load_lorebook_entry(&conn, &entry.id)
            .expect("Load should succeed")
            .expect("Entry should exist");
        assert_eq!(loaded.secondary_terms, vec!["sec1", "sec2"]);
    }
}
