use anyhow::Result;
use rusqlite::Connection;

pub fn load_app_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT value FROM app_settings WHERE key = ?1")?;
    let mut rows = stmt.query(rusqlite::params![key])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

pub fn save_app_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

/// Loads every (key, value) pair whose key starts with `prefix` — used for
/// bulk-reading the per-character memory cutoffs.
pub fn load_app_settings_by_prefix(
    conn: &Connection,
    prefix: &str,
) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare_cached("SELECT key, value FROM app_settings WHERE key LIKE ?1")?;
    let rows = stmt.query_map(rusqlite::params![format!("{prefix}%")], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
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

    #[test]
    fn test_load_settings_by_prefix() {
        let conn = setup_test_db();
        save_app_setting(&conn, "memory_forgotten:char-1", "2026-01-01T00:00:00Z").expect("save");
        save_app_setting(&conn, "memory_forgotten:char-2", "").expect("save");
        save_app_setting(&conn, "memory_cross_conversation", "true").expect("save");

        let rows = load_app_settings_by_prefix(&conn, "memory_forgotten:").expect("load");
        assert_eq!(rows.len(), 2);
        assert!(rows.contains(&(
            "memory_forgotten:char-1".to_string(),
            "2026-01-01T00:00:00Z".to_string()
        )));
    }
}
