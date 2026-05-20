use crate::db::models::row_to_regex_script;
use crate::domain::regex_script::RegexScript;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Loads all regex scripts for a given character, ordered by sort_order ascending.
pub fn load_regex_scripts(conn: &Connection, character_id: &str) -> Result<Vec<RegexScript>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, name, find_pattern, replace_pattern, flags,
                sort_order, enabled, created_at, updated_at
         FROM regex_scripts WHERE character_id = ?1 ORDER BY sort_order ASC",
    )?;
    let scripts = stmt
        .query_map(params![character_id], |row| row_to_regex_script(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(scripts)
}

/// Loads all regex scripts (for testing).
pub fn load_all_regex_scripts(conn: &Connection) -> Result<Vec<RegexScript>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, name, find_pattern, replace_pattern, flags,
                sort_order, enabled, created_at, updated_at
         FROM regex_scripts ORDER BY character_id, sort_order ASC",
    )?;
    let scripts = stmt
        .query_map([], |row| row_to_regex_script(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(scripts)
}

/// Inserts a new regex script into the database.
/// `enabled` is stored as INTEGER (1/0) per SQLite convention.
pub fn insert_regex_script(conn: &Connection, script: &RegexScript) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO regex_scripts (id, character_id, name, find_pattern, replace_pattern,
         flags, sort_order, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?
    .execute(params![
        script.id,
        script.character_id,
        script.name,
        script.find_pattern,
        script.replace_pattern,
        script.flags,
        script.sort_order,
        script.enabled as i32,
        script.created_at.to_rfc3339(),
        script.updated_at.to_rfc3339(),
    ])?;
    Ok(())
}

/// Updates an existing regex script in the database.
pub fn update_regex_script(conn: &Connection, script: &RegexScript) -> Result<()> {
    conn.prepare_cached(
        "UPDATE regex_scripts SET name = ?1, find_pattern = ?2, replace_pattern = ?3,
         flags = ?4, sort_order = ?5, enabled = ?6, updated_at = ?7
         WHERE id = ?8",
    )?
    .execute(params![
        script.name,
        script.find_pattern,
        script.replace_pattern,
        script.flags,
        script.sort_order,
        script.enabled as i32,
        script.updated_at.to_rfc3339(),
        script.id,
    ])?;
    Ok(())
}

/// Deletes a regex script from the database by ID.
pub fn delete_regex_script(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM regex_scripts WHERE id = ?1")?
        .execute(params![id])?;
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

    fn make_script(character_id: &str, name: &str, sort_order: i32, enabled: bool) -> RegexScript {
        let mut script = RegexScript::new(character_id.to_string(), name.to_string());
        script.find_pattern = "test".to_string();
        script.replace_pattern = "result".to_string();
        script.sort_order = sort_order;
        script.enabled = enabled;
        script
    }

    #[test]
    fn test_insert_and_load_regex_scripts() {
        let conn = setup_test_db();
        // Need a character for FK constraint
        let character = CharacterCardV2::new("Test Character".to_string());
        crate::db::characters::insert_character(&conn, &character)
            .expect("Insert character should succeed");

        let s1 = make_script(&character.id, "Script 1", 0, true);
        let s2 = make_script(&character.id, "Script 2", 1, true);
        insert_regex_script(&conn, &s1).expect("Insert s1 should succeed");
        insert_regex_script(&conn, &s2).expect("Insert s2 should succeed");

        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 2, "Should have 2 scripts");
        assert_eq!(loaded[0].name, "Script 1");
        assert_eq!(loaded[1].name, "Script 2");
    }

    #[test]
    fn test_load_regex_scripts_filters_by_character() {
        let conn = setup_test_db();
        let char1 = CharacterCardV2::new("Char 1".to_string());
        let char2 = CharacterCardV2::new("Char 2".to_string());
        crate::db::characters::insert_character(&conn, &char1).expect("Insert char1 should succeed");
        crate::db::characters::insert_character(&conn, &char2).expect("Insert char2 should succeed");

        let s1 = make_script(&char1.id, "Char1 Script", 0, true);
        let s2 = make_script(&char2.id, "Char2 Script", 0, true);
        insert_regex_script(&conn, &s1).expect("Insert s1 should succeed");
        insert_regex_script(&conn, &s2).expect("Insert s2 should succeed");

        let loaded1 = load_regex_scripts(&conn, &char1.id).expect("Load should succeed");
        let loaded2 = load_regex_scripts(&conn, &char2.id).expect("Load should succeed");

        assert_eq!(loaded1.len(), 1, "Char1 should have 1 script");
        assert_eq!(loaded2.len(), 1, "Char2 should have 1 script");
        assert_eq!(loaded1[0].name, "Char1 Script");
        assert_eq!(loaded2[0].name, "Char2 Script");
    }

    #[test]
    fn test_update_regex_script() {
        let conn = setup_test_db();
        let character = CharacterCardV2::new("Test Character".to_string());
        crate::db::characters::insert_character(&conn, &character)
            .expect("Insert character should succeed");

        let mut script = make_script(&character.id, "Original", 0, true);
        insert_regex_script(&conn, &script).expect("Insert should succeed");

        script.name = "Updated".to_string();
        script.find_pattern = "new_pattern".to_string();
        script.updated_at = chrono::Utc::now();
        update_regex_script(&conn, &script).expect("Update should succeed");

        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Updated");
        assert_eq!(loaded[0].find_pattern, "new_pattern");
    }

    #[test]
    fn test_delete_regex_script() {
        let conn = setup_test_db();
        let character = CharacterCardV2::new("Test Character".to_string());
        crate::db::characters::insert_character(&conn, &character)
            .expect("Insert character should succeed");

        let script = make_script(&character.id, "To Delete", 0, true);
        insert_regex_script(&conn, &script).expect("Insert should succeed");

        delete_regex_script(&conn, &script.id).expect("Delete should succeed");

        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert!(loaded.is_empty(), "Script should be deleted");
    }

    #[test]
    fn test_regex_script_enabled_stored_as_integer() {
        let conn = setup_test_db();
        let character = CharacterCardV2::new("Test Character".to_string());
        crate::db::characters::insert_character(&conn, &character)
            .expect("Insert character should succeed");

        let mut script = make_script(&character.id, "Disabled Script", 0, false);
        script.enabled = false;
        insert_regex_script(&conn, &script).expect("Insert should succeed");

        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 1);
        assert!(!loaded[0].enabled, "enabled should be false when stored as INTEGER 0");
    }
}
