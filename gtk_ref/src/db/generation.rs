use crate::db::models::row_to_generation_preset;
use crate::domain::generation::GenerationPreset;
use anyhow::Result;
use rusqlite::{params, Connection};

/// Seeds built-in presets (Creative, Balanced, Precise) using INSERT OR IGNORE.
/// Idempotent — safe to call multiple times. Fixed UUIDs ensure stability.
pub fn seed_builtin_presets(conn: &Connection) -> Result<()> {
    let presets = [
        GenerationPreset::creative(),
        GenerationPreset::balanced(),
        GenerationPreset::precise(),
    ];
    for preset in &presets {
        let stop_sequences_str = serde_json::to_string(&preset.stop_sequences)?;
        conn.prepare_cached(
            "INSERT OR IGNORE INTO generation_presets
             (id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty,
              presence_penalty, stop_sequences, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        )?
        .execute(params![
            preset.id,
            preset.name,
            preset.is_builtin as i32,
            preset.temperature,
            preset.top_p,
            preset.max_tokens,
            preset.frequency_penalty,
            preset.presence_penalty,
            stop_sequences_str,
            preset.created_at.to_string(),
            preset.updated_at.to_string(),
        ])?;
    }
    Ok(())
}

/// Inserts a new generation preset into the database.
/// stop_sequences is serialized as a JSON TEXT column.
pub fn insert_preset(conn: &Connection, preset: &GenerationPreset) -> Result<()> {
    let stop_sequences_str = serde_json::to_string(&preset.stop_sequences)?;
    conn.prepare_cached(
        "INSERT INTO generation_presets
         (id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty,
          presence_penalty, stop_sequences, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    )?
    .execute(params![
        preset.id,
        preset.name,
        preset.is_builtin as i32,
        preset.temperature,
        preset.top_p,
        preset.max_tokens,
        preset.frequency_penalty,
        preset.presence_penalty,
        stop_sequences_str,
        preset.created_at.to_string(),
        preset.updated_at.to_string(),
    ])?;
    Ok(())
}

/// Loads all generation presets ordered by is_builtin DESC, then name.
pub fn load_presets(conn: &Connection) -> Result<Vec<GenerationPreset>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty,
                presence_penalty, stop_sequences, created_at, updated_at
         FROM generation_presets ORDER BY is_builtin DESC, name"
    )?;
    let presets = stmt.query_map([], |row| row_to_generation_preset(row))?;
    let mut result = Vec::new();
    for preset in presets {
        result.push(preset?);
    }
    Ok(result)
}

/// Loads a single generation preset by ID.
pub fn load_preset_by_id(conn: &Connection, id: &str) -> Result<Option<GenerationPreset>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty,
                presence_penalty, stop_sequences, created_at, updated_at
         FROM generation_presets WHERE id = ?1"
    )?;
    let mut presets = stmt.query_map(params![id], |row| row_to_generation_preset(row))?;
    match presets.next() {
        Some(p) => Ok(Some(p?)),
        None => Ok(None),
    }
}

/// Updates an existing generation preset.
/// Returns error if preset is_builtin=true (built-in presets are read-only per D-14).
pub fn update_preset(conn: &Connection, preset: &GenerationPreset) -> Result<()> {
    // Check if preset is built-in — cannot update built-in presets
    let existing = load_preset_by_id(conn, &preset.id)?;
    match existing {
        Some(p) if p.is_builtin => {
            return Err(anyhow::anyhow!(
                "Cannot update built-in preset '{}'. Built-in presets are read-only.",
                p.name
            ));
        }
        Some(_) => {} // Not built-in, proceed with update
        None => {
            return Err(anyhow::anyhow!("Preset not found: {}", preset.id));
        }
    }

    let stop_sequences_str = serde_json::to_string(&preset.stop_sequences)?;
    conn.prepare_cached(
        "UPDATE generation_presets SET name = ?1, temperature = ?2, top_p = ?3,
         max_tokens = ?4, frequency_penalty = ?5, presence_penalty = ?6,
         stop_sequences = ?7, updated_at = ?8 WHERE id = ?9"
    )?
    .execute(params![
        preset.name,
        preset.temperature,
        preset.top_p,
        preset.max_tokens,
        preset.frequency_penalty,
        preset.presence_penalty,
        stop_sequences_str,
        preset.updated_at.to_string(),
        preset.id,
    ])?;
    Ok(())
}

/// Deletes a generation preset by ID.
/// Returns error if preset is_builtin=true (built-in presets are read-only per D-14).
pub fn delete_preset(conn: &Connection, id: &str) -> Result<()> {
    // Check if preset is built-in — cannot delete built-in presets
    let existing = load_preset_by_id(conn, id)?;
    match existing {
        Some(p) if p.is_builtin => {
            return Err(anyhow::anyhow!(
                "Cannot delete built-in preset '{}'. Built-in presets are read-only.",
                p.name
            ));
        }
        Some(_) => {} // Not built-in, proceed with delete
        None => {
            return Err(anyhow::anyhow!("Preset not found: {}", id));
        }
    }

    conn.prepare_cached("DELETE FROM generation_presets WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
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

    #[test]
    fn test_generation_preset_round_trip() {
        let conn = setup_test_db();

        let mut preset = GenerationPreset::new("Custom Preset".to_string());
        preset.temperature = 1.5;
        preset.top_p = 0.85;
        preset.max_tokens = 4096;
        preset.frequency_penalty = 0.2;
        preset.presence_penalty = 0.1;
        preset.stop_sequences = vec!["\\n".to_string(), "STOP".to_string()];

        // Insert the preset
        insert_preset(&conn, &preset).expect("Insert should succeed");

        // Load all presets and find ours
        let loaded = load_presets(&conn).expect("Load should succeed");
        let found = loaded.iter().find(|p| p.id == preset.id)
            .expect("Should find inserted preset");

        // Verify all fields match
        assert_eq!(found.id, preset.id);
        assert_eq!(found.name, "Custom Preset");
        assert!(!found.is_builtin);
        assert_eq!(found.temperature, 1.5);
        assert_eq!(found.top_p, 0.85);
        assert_eq!(found.max_tokens, 4096);
        assert_eq!(found.frequency_penalty, 0.2);
        assert_eq!(found.presence_penalty, 0.1);
        assert_eq!(found.stop_sequences, vec!["\\n", "STOP"]);
    }

    #[test]
    fn test_builtin_preset_unique_name_constraint() {
        let conn = setup_test_db();

        // Seed built-in presets
        seed_builtin_presets(&conn).expect("Seed should succeed");

        // Try inserting a duplicate "Creative" preset — should fail due to UNIQUE constraint
        let mut duplicate = GenerationPreset::new("Creative".to_string());
        duplicate.id = "different-id".to_string();
        let result = insert_preset(&conn, &duplicate);
        assert!(result.is_err(), "Inserting duplicate preset name should fail");
    }

    #[test]
    fn test_builtin_presets_seeded() {
        let conn = setup_test_db();
        seed_builtin_presets(&conn).expect("Seed should succeed");

        let loaded = load_presets(&conn).expect("Load should succeed");
        assert!(loaded.len() >= 3, "Should have at least 3 built-in presets");

        let names: Vec<&str> = loaded.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Creative"), "Should have Creative preset");
        assert!(names.contains(&"Balanced"), "Should have Balanced preset");
        assert!(names.contains(&"Precise"), "Should have Precise preset");
    }

    #[test]
    fn test_cannot_update_builtin_preset() {
        let conn = setup_test_db();
        seed_builtin_presets(&conn).expect("Seed should succeed");

        let loaded = load_presets(&conn).expect("Load should succeed");
        let builtin = loaded.iter().find(|p| p.is_builtin).expect("Should find builtin preset");

        let mut modified = builtin.clone();
        modified.temperature = 99.0;
        let result = update_preset(&conn, &modified);
        assert!(result.is_err(), "Updating built-in preset should fail");
    }

    #[test]
    fn test_cannot_delete_builtin_preset() {
        let conn = setup_test_db();
        seed_builtin_presets(&conn).expect("Seed should succeed");

        let loaded = load_presets(&conn).expect("Load should succeed");
        let builtin = loaded.iter().find(|p| p.is_builtin).expect("Should find builtin preset");

        let result = delete_preset(&conn, &builtin.id);
        assert!(result.is_err(), "Deleting built-in preset should fail");
    }

    #[test]
    fn test_load_preset_by_id() {
        let conn = setup_test_db();

        let preset = GenerationPreset::new("FindMe".to_string());
        insert_preset(&conn, &preset).expect("Insert should succeed");

        let loaded = load_preset_by_id(&conn, &preset.id).expect("Load should succeed");
        assert!(loaded.is_some(), "Should find preset by ID");
        assert_eq!(loaded.unwrap().name, "FindMe");
    }

    #[test]
    fn test_load_preset_by_id_not_found() {
        let conn = setup_test_db();
        let loaded = load_preset_by_id(&conn, "nonexistent-id").expect("Load should succeed");
        assert!(loaded.is_none(), "Should return None for nonexistent preset");
    }

    #[test]
    fn test_seed_builtin_presets_idempotent() {
        let conn = setup_test_db();

        // Seed twice — should not error or create duplicates
        seed_builtin_presets(&conn).expect("First seed should succeed");
        seed_builtin_presets(&conn).expect("Second seed should succeed");

        let loaded = load_presets(&conn).expect("Load should succeed");
        let builtin_count = loaded.iter().filter(|p| p.is_builtin).count();
        assert_eq!(builtin_count, 3, "Should have exactly 3 built-in presets after seeding twice");
    }

    #[test]
    fn test_delete_custom_preset() {
        let conn = setup_test_db();

        let preset = GenerationPreset::new("To Delete".to_string());
        insert_preset(&conn, &preset).expect("Insert should succeed");

        delete_preset(&conn, &preset.id).expect("Delete should succeed");
        let loaded = load_preset_by_id(&conn, &preset.id).expect("Load should succeed");
        assert!(loaded.is_none(), "Deleted preset should not be found");
    }

    #[test]
    fn test_update_custom_preset() {
        let conn = setup_test_db();

        let mut preset = GenerationPreset::new("Update Me".to_string());
        insert_preset(&conn, &preset).expect("Insert should succeed");

        preset.temperature = 2.0;
        preset.updated_at = chrono::Local::now().naive_local();
        update_preset(&conn, &preset).expect("Update should succeed");

        let loaded = load_preset_by_id(&conn, &preset.id)
            .expect("Load should succeed")
            .expect("Should exist");
        assert_eq!(loaded.temperature, 2.0);
    }
}
