use crate::db::models::{row_to_lorebook, row_to_lorebook_entry};
use crate::domain::lorebook::{Lorebook, LorebookEntry};
use anyhow::Result;
use rusqlite::{params, Connection};

// ── Lorebook CRUD ──────────────────────────────────────────────────────────

/// Loads all lorebooks from the database, ordered by updated_at descending.
pub fn load_lorebooks(conn: &Connection) -> Result<Vec<Lorebook>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, created_at, updated_at
         FROM lorebooks ORDER BY updated_at DESC",
    )?;
    let lorebooks = stmt.query_map([], |row| row_to_lorebook(row))?;
    let mut result = Vec::new();
    for lorebook in lorebooks {
        result.push(lorebook?);
    }
    Ok(result)
}

/// Loads a single lorebook by ID.
pub fn load_lorebook(conn: &Connection, id: &str) -> Result<Option<Lorebook>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, created_at, updated_at
         FROM lorebooks WHERE id = ?1",
    )?;
    let mut lorebooks = stmt.query_map(params![id], |row| row_to_lorebook(row))?;
    match lorebooks.next() {
        Some(lb) => Ok(Some(lb?)),
        None => Ok(None),
    }
}

/// Inserts a new lorebook into the database.
pub fn insert_lorebook(conn: &Connection, lorebook: &Lorebook) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO lorebooks (id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?
    .execute(params![
        lorebook.id,
        lorebook.name,
        lorebook.description,
        lorebook.created_at.to_rfc3339(),
        lorebook.updated_at.to_rfc3339(),
    ])?;
    Ok(())
}

/// Updates an existing lorebook in the database.
pub fn update_lorebook(conn: &Connection, lorebook: &Lorebook) -> Result<()> {
    conn.prepare_cached(
        "UPDATE lorebooks SET name = ?1, description = ?2, updated_at = ?3
         WHERE id = ?4",
    )?
    .execute(params![
        lorebook.name,
        lorebook.description,
        lorebook.updated_at.to_rfc3339(),
        lorebook.id,
    ])?;
    Ok(())
}

/// Deletes a lorebook from the database by ID.
pub fn delete_lorebook(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM lorebooks WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

// ── LorebookEntry CRUD ─────────────────────────────────────────────────────

/// Loads all entries for a given lorebook, ordered by created_at ascending.
pub fn load_lorebook_entries(conn: &Connection, lorebook_id: &str) -> Result<Vec<LorebookEntry>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, lorebook_id, keys, secondary_keys, content, position, depth,
         activation_state, extensions, created_at, updated_at
         FROM lorebook_entries WHERE lorebook_id = ?1 ORDER BY created_at ASC",
    )?;
    let entries = stmt.query_map(params![lorebook_id], |row| row_to_lorebook_entry(row))?;
    let mut result = Vec::new();
    for entry in entries {
        result.push(entry?);
    }
    Ok(result)
}

/// Inserts a new lorebook entry into the database.
pub fn insert_lorebook_entry(conn: &Connection, entry: &LorebookEntry) -> Result<()> {
    let keys_str = serde_json::to_string(&entry.keys)?;
    let secondary_keys_str = serde_json::to_string(&entry.secondary_keys)?;
    let extensions_str = serde_json::to_string(&entry.extensions)?;

    conn.prepare_cached(
        "INSERT INTO lorebook_entries (id, lorebook_id, keys, secondary_keys, content,
         position, depth, activation_state, extensions, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?
    .execute(params![
        entry.id,
        entry.lorebook_id,
        keys_str,
        secondary_keys_str,
        entry.content,
        entry.position.to_string(),
        entry.depth,
        entry.activation_state.to_string(),
        extensions_str,
        entry.created_at.to_rfc3339(),
        entry.updated_at.to_rfc3339(),
    ])?;
    Ok(())
}

/// Updates an existing lorebook entry in the database.
pub fn update_lorebook_entry(conn: &Connection, entry: &LorebookEntry) -> Result<()> {
    let keys_str = serde_json::to_string(&entry.keys)?;
    let secondary_keys_str = serde_json::to_string(&entry.secondary_keys)?;
    let extensions_str = serde_json::to_string(&entry.extensions)?;

    conn.prepare_cached(
        "UPDATE lorebook_entries SET keys = ?1, secondary_keys = ?2, content = ?3,
         position = ?4, depth = ?5, activation_state = ?6, extensions = ?7,
         updated_at = ?8 WHERE id = ?9",
    )?
    .execute(params![
        keys_str,
        secondary_keys_str,
        entry.content,
        entry.position.to_string(),
        entry.depth,
        entry.activation_state.to_string(),
        extensions_str,
        entry.updated_at.to_rfc3339(),
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

// ── Character-Lorebook junction ─────────────────────────────────────────────

/// Assigns a lorebook to a character with a given sort_order.
/// Idempotent — will silently succeed if the assignment already exists.
pub fn assign_lorebook_to_character(
    conn: &Connection,
    character_id: &str,
    lorebook_id: &str,
    sort_order: i32,
) -> Result<()> {
    conn.prepare_cached(
        "INSERT OR IGNORE INTO character_lorebooks (character_id, lorebook_id, sort_order)
         VALUES (?1, ?2, ?3)",
    )?
    .execute(params![character_id, lorebook_id, sort_order])?;
    Ok(())
}

/// Removes a lorebook assignment from a character.
pub fn remove_lorebook_from_character(
    conn: &Connection,
    character_id: &str,
    lorebook_id: &str,
) -> Result<()> {
    conn.prepare_cached(
        "DELETE FROM character_lorebooks WHERE character_id = ?1 AND lorebook_id = ?2",
    )?
    .execute(params![character_id, lorebook_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::domain::character::CharacterCardV2;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    #[test]
    fn test_insert_and_load_lorebooks() {
        let conn = setup_test_db();
        let lorebook = Lorebook::new("Test Lorebook".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert should succeed");

        let loaded = load_lorebooks(&conn).expect("Load should succeed");
        assert_eq!(loaded.len(), 1, "Should have 1 lorebook");
        assert_eq!(loaded[0].id, lorebook.id);
        assert_eq!(loaded[0].name, "Test Lorebook");
    }

    #[test]
    fn test_load_lorebook_by_id() {
        let conn = setup_test_db();
        let lorebook = Lorebook::new("FindByID".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert should succeed");

        let loaded = load_lorebook(&conn, &lorebook.id).expect("Load should succeed");
        assert!(loaded.is_some(), "Should find lorebook by ID");
        assert_eq!(loaded.unwrap().name, "FindByID");
    }

    #[test]
    fn test_load_lorebook_not_found() {
        let conn = setup_test_db();
        let loaded = load_lorebook(&conn, "nonexistent-id").expect("Load should succeed");
        assert!(
            loaded.is_none(),
            "Should return None for nonexistent lorebook"
        );
    }

    #[test]
    fn test_update_lorebook() {
        let conn = setup_test_db();
        let mut lorebook = Lorebook::new("Original".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert should succeed");

        lorebook.name = "Updated".to_string();
        lorebook.description = "Updated description".to_string();
        lorebook.updated_at = chrono::Utc::now();
        update_lorebook(&conn, &lorebook).expect("Update should succeed");

        let loaded = load_lorebook(&conn, &lorebook.id)
            .expect("Load should succeed")
            .expect("Lorebook should exist");
        assert_eq!(loaded.name, "Updated");
        assert_eq!(loaded.description, "Updated description");
    }

    #[test]
    fn test_delete_lorebook() {
        let conn = setup_test_db();
        let lorebook = Lorebook::new("To Delete".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert should succeed");

        delete_lorebook(&conn, &lorebook.id).expect("Delete should succeed");
        let loaded = load_lorebooks(&conn).expect("Load should succeed");
        assert!(loaded.is_empty(), "Lorebook should be deleted");
    }

    #[test]
    fn test_insert_and_load_lorebook_entries() {
        let conn = setup_test_db();
        let lorebook = Lorebook::new("Test LB".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert lorebook should succeed");

        let mut entry = LorebookEntry::new(lorebook.id.clone());
        entry.keys = vec!["magic".to_string()];
        entry.content = "Magic is real.".to_string();
        insert_lorebook_entry(&conn, &entry).expect("Insert entry should succeed");

        let loaded =
            load_lorebook_entries(&conn, &lorebook.id).expect("Load entries should succeed");
        assert_eq!(loaded.len(), 1, "Should have 1 entry");
        assert_eq!(loaded[0].id, entry.id);
        assert_eq!(loaded[0].keys, vec!["magic"]);
        assert_eq!(loaded[0].content, "Magic is real.");
    }

    #[test]
    fn test_update_lorebook_entry() {
        let conn = setup_test_db();
        let lorebook = Lorebook::new("Test LB".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert lorebook should succeed");

        let mut entry = LorebookEntry::new(lorebook.id.clone());
        insert_lorebook_entry(&conn, &entry).expect("Insert entry should succeed");

        entry.content = "Updated content".to_string();
        entry.keys = vec!["new_key".to_string()];
        entry.updated_at = chrono::Utc::now();
        update_lorebook_entry(&conn, &entry).expect("Update entry should succeed");

        let loaded =
            load_lorebook_entries(&conn, &lorebook.id).expect("Load entries should succeed");
        assert_eq!(loaded[0].content, "Updated content");
        assert_eq!(loaded[0].keys, vec!["new_key"]);
    }

    #[test]
    fn test_delete_lorebook_entry() {
        let conn = setup_test_db();
        let lorebook = Lorebook::new("Test LB".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert lorebook should succeed");

        let entry = LorebookEntry::new(lorebook.id.clone());
        insert_lorebook_entry(&conn, &entry).expect("Insert entry should succeed");

        delete_lorebook_entry(&conn, &entry.id).expect("Delete entry should succeed");
        let loaded =
            load_lorebook_entries(&conn, &lorebook.id).expect("Load entries should succeed");
        assert!(loaded.is_empty(), "Entry should be deleted");
    }

    #[test]
    fn test_assign_and_remove_lorebook_from_character() {
        let conn = setup_test_db();
        let character = CharacterCardV2::new("Test Character".to_string());
        crate::db::characters::insert_character(&conn, &character)
            .expect("Insert character should succeed");

        let lorebook = Lorebook::new("Test Lorebook".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert lorebook should succeed");

        assign_lorebook_to_character(&conn, &character.id, &lorebook.id, 0)
            .expect("Assign should succeed");

        // Verify assignment exists
        let result: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM character_lorebooks WHERE character_id = ?1 AND lorebook_id = ?2",
                params![character.id, lorebook.id],
                |row| row.get(0),
            )
            .expect("Query should succeed");
        assert_eq!(result, 1, "Should have 1 assignment");

        // Idempotent — inserting again should not fail
        assign_lorebook_to_character(&conn, &character.id, &lorebook.id, 1)
            .expect("Duplicate assign should succeed (OR IGNORE)");

        // Remove
        remove_lorebook_from_character(&conn, &character.id, &lorebook.id)
            .expect("Remove should succeed");

        let result: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM character_lorebooks WHERE character_id = ?1 AND lorebook_id = ?2",
                params![character.id, lorebook.id],
                |row| row.get(0),
            )
            .expect("Query should succeed");
        assert_eq!(result, 0, "Assignment should be removed");
    }

    #[test]
    fn test_delete_lorebook_cascades_entries() {
        let conn = setup_test_db();
        let lorebook = Lorebook::new("Cascade Test".to_string());
        insert_lorebook(&conn, &lorebook).expect("Insert lorebook should succeed");

        let entry = LorebookEntry::new(lorebook.id.clone());
        insert_lorebook_entry(&conn, &entry).expect("Insert entry should succeed");

        // Deleting the lorebook should cascade and delete its entries
        delete_lorebook(&conn, &lorebook.id).expect("Delete lorebook should succeed");

        let entries =
            load_lorebook_entries(&conn, &lorebook.id).expect("Load entries should succeed");
        assert!(entries.is_empty(), "Entries should be cascade-deleted");
    }
}
