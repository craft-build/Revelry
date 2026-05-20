use crate::db::models::row_to_generation_preset;
use crate::domain::generation::GenerationPreset;
use anyhow::Result;
use chrono::Utc;
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
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
            preset.created_at.to_rfc3339(),
            preset.updated_at.to_rfc3339(),
        ])?;
    }
    Ok(())
}

/// Loads all generation presets from the database, ordered by name.
pub fn load_presets(conn: &Connection) -> Result<Vec<GenerationPreset>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, is_builtin, temperature, top_p, max_tokens,
         frequency_penalty, presence_penalty, stop_sequences, created_at, updated_at
         FROM generation_presets ORDER BY name",
    )?;
    let presets = stmt.query_map([], |row| row_to_generation_preset(row))?;
    let mut result = Vec::new();
    for preset in presets {
        result.push(preset?);
    }
    Ok(result)
}

/// Inserts a new user-created preset into the database.
/// Returns the inserted preset.
pub fn insert_preset(conn: &Connection, preset: &GenerationPreset) -> Result<()> {
    let stop_sequences_str = serde_json::to_string(&preset.stop_sequences)?;
    conn.prepare_cached(
        "INSERT INTO generation_presets
         (id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty,
          presence_penalty, stop_sequences, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
        preset.created_at.to_rfc3339(),
        preset.updated_at.to_rfc3339(),
    ])?;
    Ok(())
}

/// Updates an existing user-created preset. Rejects built-in presets.
pub fn update_preset(conn: &Connection, preset: &GenerationPreset) -> Result<()> {
    if preset.is_builtin {
        anyhow::bail!("Cannot modify built-in preset '{}'", preset.name);
    }
    let stop_sequences_str = serde_json::to_string(&preset.stop_sequences)?;
    let now = Utc::now();
    let rows = conn.prepare_cached(
        "UPDATE generation_presets SET
         name = ?1, temperature = ?2, top_p = ?3, max_tokens = ?4,
         frequency_penalty = ?5, presence_penalty = ?6, stop_sequences = ?7,
         updated_at = ?8
         WHERE id = ?9 AND is_builtin = 0",
    )?
    .execute(params![
        preset.name,
        preset.temperature,
        preset.top_p,
        preset.max_tokens,
        preset.frequency_penalty,
        preset.presence_penalty,
        stop_sequences_str,
        now.to_rfc3339(),
        preset.id,
    ])?;
    if rows == 0 {
        anyhow::bail!("Preset not found or is built-in: {}", preset.id);
    }
    Ok(())
}

/// Deletes a user-created preset by id. Rejects built-in presets.
pub fn delete_preset(conn: &Connection, id: &str) -> Result<()> {
    let rows = conn.prepare_cached(
        "DELETE FROM generation_presets WHERE id = ?1 AND is_builtin = 0",
    )?
    .execute(params![id])?;
    if rows == 0 {
        anyhow::bail!("Preset not found or is built-in: {}", id);
    }
    Ok(())
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

    #[test]
    fn test_builtin_presets_seeded() {
        let conn = setup_test_db();
        seed_builtin_presets(&conn).expect("Seed should succeed");

        let presets = load_presets(&conn).expect("Load should succeed");
        assert!(
            presets.len() >= 3,
            "Should have at least 3 built-in presets"
        );
        let names: Vec<&str> = presets.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"Creative"), "Should have Creative preset");
        assert!(names.contains(&"Balanced"), "Should have Balanced preset");
        assert!(names.contains(&"Precise"), "Should have Precise preset");
    }

    #[test]
    fn test_load_presets_returns_domain_models() {
        let conn = setup_test_db();
        seed_builtin_presets(&conn).expect("Seed should succeed");

        let presets = load_presets(&conn).expect("Load should succeed");
        let creative = presets
            .iter()
            .find(|p| p.id == "builtin-creative")
            .expect("Should find Creative");
        assert_eq!(creative.name, "Creative");
        assert!(creative.is_builtin);
        assert_eq!(creative.temperature, 1.2);
        assert_eq!(creative.top_p, 0.9);
        assert!(creative.stop_sequences.is_empty());
    }

    #[test]
    fn test_seed_builtin_presets_idempotent() {
        let conn = setup_test_db();

        // Seed twice — should not error or create duplicates
        seed_builtin_presets(&conn).expect("First seed should succeed");
        seed_builtin_presets(&conn).expect("Second seed should succeed");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM generation_presets WHERE is_builtin = 1",
                [],
                |row| row.get(0),
            )
            .expect("count failed");
        assert_eq!(
            count, 3,
            "Should have exactly 3 built-in presets after seeding twice"
        );
    }
}
