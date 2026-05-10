use crate::db::models::row_to_regex_script;
use crate::domain::regex_script::RegexScript;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Inserts a new regex script into the database.
pub fn insert_regex_script(conn: &Connection, script: &RegexScript) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO regex_scripts (id, character_id, name, find_pattern, replace_pattern,
         flags, sort_order, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
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
        script.created_at.to_string(),
        script.updated_at.to_string(),
    ])?;
    Ok(())
}

/// Loads all regex scripts for a character, ordered by sort_order.
pub fn load_regex_scripts(conn: &Connection, character_id: &str) -> Result<Vec<RegexScript>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, name, find_pattern, replace_pattern,
                flags, sort_order, enabled, created_at, updated_at
         FROM regex_scripts WHERE character_id = ?1 ORDER BY sort_order"
    )?;
    let scripts = stmt.query_map(params![character_id], |row| row_to_regex_script(row))?;
    let mut result = Vec::new();
    for script in scripts {
        result.push(script?);
    }
    Ok(result)
}

/// Loads a single regex script by ID.
pub fn load_regex_script_by_id(conn: &Connection, id: &str) -> Result<Option<RegexScript>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, name, find_pattern, replace_pattern,
                flags, sort_order, enabled, created_at, updated_at
         FROM regex_scripts WHERE id = ?1"
    )?;
    let mut scripts = stmt.query_map(params![id], |row| row_to_regex_script(row))?;
    match scripts.next() {
        Some(s) => Ok(Some(s?)),
        None => Ok(None),
    }
}

/// Updates an existing regex script in the database.
pub fn update_regex_script(conn: &Connection, script: &RegexScript) -> Result<()> {
    conn.prepare_cached(
        "UPDATE regex_scripts SET name = ?1, find_pattern = ?2, replace_pattern = ?3,
         flags = ?4, sort_order = ?5, enabled = ?6, updated_at = ?7 WHERE id = ?8"
    )?
    .execute(params![
        script.name,
        script.find_pattern,
        script.replace_pattern,
        script.flags,
        script.sort_order,
        script.enabled as i32,
        script.updated_at.to_string(),
        script.id,
    ])?;
    Ok(())
}

/// Deletes a regex script by ID.
pub fn delete_regex_script(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM regex_scripts WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::characters::insert_character;
    use crate::db::migrations::run_migrations;
    use crate::domain::character::CharacterCardV2;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    fn insert_test_character(conn: &Connection) -> CharacterCardV2 {
        let character = CharacterCardV2::new("Test Character".to_string());
        insert_character(conn, &character).expect("Insert character");
        character
    }

    #[test]
    fn test_regex_script_round_trip() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);

        let mut script = RegexScript::new(
            character.id.clone(),
            "Remove OOC".to_string(),
            "\\(.*?\\)".to_string(),
            "".to_string(),
        );
        script.flags = "gi".to_string();
        script.sort_order = 1;

        insert_regex_script(&conn, &script).expect("Insert should succeed");

        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 1, "Should have 1 script");
        assert_eq!(loaded[0].id, script.id);
        assert_eq!(loaded[0].character_id, character.id);
        assert_eq!(loaded[0].name, "Remove OOC");
        assert_eq!(loaded[0].find_pattern, "\\(.*?\\)");
        assert_eq!(loaded[0].replace_pattern, "");
        assert_eq!(loaded[0].flags, "gi");
        assert_eq!(loaded[0].sort_order, 1);
        assert!(loaded[0].enabled);
    }

    #[test]
    fn test_regex_scripts_ordered_by_sort_order() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);

        let mut script1 = RegexScript::new(character.id.clone(), "Script Z".to_string(), "a".to_string(), "b".to_string());
        script1.sort_order = 10;
        insert_regex_script(&conn, &script1).expect("Insert should succeed");

        let mut script2 = RegexScript::new(character.id.clone(), "Script A".to_string(), "c".to_string(), "d".to_string());
        script2.sort_order = 1;
        insert_regex_script(&conn, &script2).expect("Insert should succeed");

        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Script A"); // sort_order 1
        assert_eq!(loaded[1].name, "Script Z"); // sort_order 10
    }

    #[test]
    fn test_cascade_delete_regex_scripts_on_character_delete() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);

        // Insert regex scripts for this character
        let script1 = RegexScript::new(character.id.clone(), "Script 1".to_string(), "a".to_string(), "b".to_string());
        insert_regex_script(&conn, &script1).expect("Insert should succeed");

        let script2 = RegexScript::new(character.id.clone(), "Script 2".to_string(), "c".to_string(), "d".to_string());
        insert_regex_script(&conn, &script2).expect("Insert should succeed");

        // Verify scripts exist
        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 2, "Should have 2 scripts before character delete");

        // Delete the character — CASCADE should remove regex_scripts
        crate::db::characters::delete_character(&conn, &character.id).expect("Delete character");

        // Verify scripts are gone
        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert!(loaded.is_empty(), "Scripts should be cascade-deleted with character");
    }

    #[test]
    fn test_update_regex_script() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);

        let mut script = RegexScript::new(character.id.clone(), "Original".to_string(), "a".to_string(), "b".to_string());
        insert_regex_script(&conn, &script).expect("Insert should succeed");

        script.name = "Updated".to_string();
        script.find_pattern = "new_pattern".to_string();
        script.replace_pattern = "new_replacement".to_string();
        script.sort_order = 5;
        script.enabled = false;
        script.updated_at = chrono::Local::now().naive_local();
        update_regex_script(&conn, &script).expect("Update should succeed");

        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Updated");
        assert_eq!(loaded[0].find_pattern, "new_pattern");
        assert_eq!(loaded[0].replace_pattern, "new_replacement");
        assert_eq!(loaded[0].sort_order, 5);
        assert!(!loaded[0].enabled);
    }

    #[test]
    fn test_delete_regex_script() {
        let conn = setup_test_db();
        let character = insert_test_character(&conn);

        let script = RegexScript::new(character.id.clone(), "To Delete".to_string(), "a".to_string(), "b".to_string());
        insert_regex_script(&conn, &script).expect("Insert should succeed");

        delete_regex_script(&conn, &script.id).expect("Delete should succeed");
        let loaded = load_regex_scripts(&conn, &character.id).expect("Load should succeed");
        assert!(loaded.is_empty(), "Script should be deleted");
    }

    #[test]
    fn test_regex_scripts_per_character_isolation() {
        let conn = setup_test_db();

        let char1 = CharacterCardV2::new("Character 1".to_string());
        insert_character(&conn, &char1).expect("Insert char1");

        let char2 = CharacterCardV2::new("Character 2".to_string());
        insert_character(&conn, &char2).expect("Insert char2");

        let script1 = RegexScript::new(char1.id.clone(), "Char1 Script".to_string(), "a".to_string(), "b".to_string());
        insert_regex_script(&conn, &script1).expect("Insert should succeed");

        let script2 = RegexScript::new(char2.id.clone(), "Char2 Script".to_string(), "c".to_string(), "d".to_string());
        insert_regex_script(&conn, &script2).expect("Insert should succeed");

        // Each character should only see its own scripts
        let loaded1 = load_regex_scripts(&conn, &char1.id).expect("Load should succeed");
        assert_eq!(loaded1.len(), 1);
        assert_eq!(loaded1[0].name, "Char1 Script");

        let loaded2 = load_regex_scripts(&conn, &char2.id).expect("Load should succeed");
        assert_eq!(loaded2.len(), 1);
        assert_eq!(loaded2[0].name, "Char2 Script");
    }
}
