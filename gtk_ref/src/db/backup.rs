use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::Path;

/// Exports the entire database as a portable SQL dump per D-08.
/// The dump recreates all tables and inserts all rows.
pub fn export_sql_dump(conn: &Connection, path: &Path) -> Result<()> {
    let mut file = fs::File::create(path)
        .with_context(|| format!("Failed to create SQL dump file at {:?}", path))?;

    // Note: PRAGMA foreign_keys=OFF must be set BEFORE the transaction,
    // not inside it. We don't include it in the dump — the import function
    // sets it externally.
    writeln!(file, "BEGIN TRANSACTION;")?;

    // Save user_version to restore after import
    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);

    // Get all user tables (not sqlite_ internal tables, not FTS5 shadow tables)
    // FTS5 shadow tables have names like {table}_fts_config, {table}_fts_data, etc.
    let mut stmt = conn.prepare(
        "SELECT name, sql FROM sqlite_master WHERE type='table'
         AND name NOT LIKE 'sqlite_%'
         AND name NOT GLOB '*_fts_*'
         AND name NOT GLOB '*_fts'
         ORDER BY name"
    )?;

    let tables: Vec<(String, Option<String>)> = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let sql: Option<String> = row.get(1)?;
        Ok((name, sql))
    })?.filter_map(|r| r.ok()).collect();

    for (table_name, create_sql) in &tables {
        // Write DROP TABLE IF EXISTS
        writeln!(file, "DROP TABLE IF EXISTS {};", table_name)?;

        // Write CREATE TABLE DDL
        if let Some(sql) = create_sql {
            // Ensure DDL ends with semicolon (sqlite_master may or may not include it)
            let sql_trimmed = sql.trim();
            if sql_trimmed.ends_with(';') {
                writeln!(file, "{}", sql)?;
            } else {
                writeln!(file, "{};", sql)?;
            }
        }

        // Write INSERT statements for each row
        let mut row_stmt = conn.prepare(&format!("SELECT * FROM {}", table_name))?;
        let column_count = row_stmt.column_count();

        let mut rows = row_stmt.query([])?;
        while let Some(row) = rows.next()? {
            write!(file, "INSERT INTO {} VALUES (", table_name)?;
            for i in 0..column_count {
                if i > 0 {
                    write!(file, ", ")?;
                }
                let value: rusqlite::types::Value = row.get(i)?;
                match value {
                    rusqlite::types::Value::Null => write!(file, "NULL")?,
                    rusqlite::types::Value::Integer(n) => write!(file, "{}", n)?,
                    rusqlite::types::Value::Real(f) => write!(file, "{:.15}", f)?,
                    rusqlite::types::Value::Text(s) => {
                        let escaped = s.replace('\'', "''");
                        write!(file, "'{}'", escaped)?;
                    }
                    rusqlite::types::Value::Blob(b) => {
                        write!(file, "X'")?;
                        for byte in &b {
                            write!(file, "{:02x}", byte)?;
                        }
                        write!(file, "'")?;
                    }
                }
            }
            writeln!(file, ");")?;
        }
    }

    // Export FTS5 content rebuild triggers (created by migrations on import)
    // We don't export virtual tables or triggers — they're recreated by migrations

    writeln!(file, "COMMIT;")?;

    // Set user_version OUTSIDE the transaction so that run_migrations
    // after import knows which migrations are already applied.
    writeln!(file, "PRAGMA user_version = {};", user_version)?;

    Ok(())
}

/// Exports all data as a ZIP archive per D-09.
/// Contains: revelry.db, avatars/, app_settings.json
pub fn export_zip_archive(
    db_path: &Path,
    avatar_dir: &Path,
    conn: &Connection,
    output_path: &Path,
) -> Result<()> {
    // Checkpoint WAL to flush before copying DB
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .context("Failed to checkpoint WAL before ZIP export")?;

    let file = fs::File::create(output_path)
        .with_context(|| format!("Failed to create ZIP file at {:?}", output_path))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Add database file
    let db_bytes = fs::read(db_path)
        .with_context(|| format!("Failed to read database file at {:?}", db_path))?;
    zip.start_file("revelry.db", options)?;
    zip.write_all(&db_bytes)?;

    // Add avatar directory and files
    if avatar_dir.exists() {
        zip.add_directory("avatars/", options)?;

        let entries = fs::read_dir(avatar_dir)
            .with_context(|| format!("Failed to read avatar directory at {:?}", avatar_dir))?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let zip_path = format!("avatars/{}", file_name);
                let avatar_bytes = fs::read(&path)
                    .with_context(|| format!("Failed to read avatar file at {:?}", path))?;
                zip.start_file(&zip_path, options)?;
                zip.write_all(&avatar_bytes)?;
            }
        }
    } else {
        zip.add_directory("avatars/", options)?;
    }

    // Add app_settings.json
    let settings = export_app_settings_json(conn)?;
    zip.start_file("app_settings.json", options)?;
    zip.write_all(settings.as_bytes())?;

    zip.finish()?;
    Ok(())
}

/// Exports app_settings table as JSON
fn export_app_settings_json(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare("SELECT key, value FROM app_settings")?;
    let rows: Vec<(String, String)> = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((key, value))
    })?.filter_map(|r| r.ok()).collect();

    let mut map = serde_json::Map::new();
    for (key, value) in rows {
        map.insert(key, serde_json::Value::String(value));
    }
    Ok(serde_json::to_string_pretty(&serde_json::Value::Object(map))?)
}

/// Imports a SQL dump per D-10.
/// Destructive: replaces all existing data.
/// Runs migrations after restore per D-11.
pub fn import_sql_dump(db_path: &Path, sql_path: &Path) -> Result<()> {
    let sql_content = fs::read_to_string(sql_path)
        .with_context(|| format!("Failed to read SQL dump at {:?}", sql_path))?;

    // Delete existing DB file (destructive per D-10)
    if db_path.exists() {
        fs::remove_file(db_path)
            .with_context(|| format!("Failed to delete existing database at {:?}", db_path))?;
    }

    // Create new connection and set PRAGMAs before any transactions
    let mut conn = Connection::open(db_path)
        .with_context(|| format!("Failed to create database at {:?}", db_path))?;
    // Use execute_batch for PRAGMAs to avoid issues with ones that return rows
    conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
    // WAL mode PRAGMA returns a row, so we need to handle it
    let _ = conn.query_row("PRAGMA journal_mode=WAL;", [], |r| { let _: String = r.get(0)?; Ok(()) });
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;

    // Execute the SQL dump (contains BEGIN TRANSACTION...COMMIT)
    conn.execute_batch(&sql_content)
        .with_context(|| format!("Failed to execute SQL dump. Dump content:\n{}", sql_content))?;

    // Re-enable foreign keys after the transaction completes
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    // Run migrations per D-11 (auto-upgrade older backups)
    // Migrations use user_version to track progress, so if the backup is from
    // the current version, no migrations will run. If the backup is older,
    // new migrations will be applied automatically.
    crate::db::migrations::run_migrations(&mut conn)
        .with_context(|| "Failed to run migrations after SQL dump import")?;

    // Check schema version compatibility
    check_schema_version(&conn)?;

    Ok(())
}

/// Imports a ZIP archive per D-10.
/// Destructive: replaces database, avatars, and settings.
/// Runs migrations after restore per D-11.
pub fn import_zip_archive(db_path: &Path, avatar_dir: &Path, archive_path: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)
        .with_context(|| format!("Failed to open ZIP archive at {:?}", archive_path))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| "Failed to read ZIP archive")?;

    // Close existing DB connection before replacing file
    // (The caller is responsible for closing the DbService Worker's connection)

    // Extract revelry.db to db_path
    {
        let mut db_file = archive.by_name("revelry.db")
            .map_err(|_| anyhow::anyhow!("ZIP archive does not contain revelry.db"))?;
        let mut db_bytes = Vec::new();
        db_file.read_to_end(&mut db_bytes)?;
        // db_file borrow is dropped here

        // Delete existing DB file and write new one
        if db_path.exists() {
            fs::remove_file(db_path)?;
        }
        let mut out = fs::File::create(db_path)?;
        out.write_all(&db_bytes)?;
    }

    // Extract avatar files
    fs::create_dir_all(avatar_dir)?;
    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        let name = zip_file.name().to_string();
        if name.starts_with("avatars/") && !name.ends_with('/') {
            // Strip the "avatars/" prefix
            let file_name = &name["avatars/".len()..];
            if !file_name.is_empty() {
                let avatar_path = avatar_dir.join(file_name);
                let mut out = fs::File::create(&avatar_path)?;
                std::io::copy(&mut zip_file, &mut out)?;
            }
        }
    }

    // Restore app_settings from JSON if present
    if let Ok(mut settings_file) = archive.by_name("app_settings.json") {
        let mut settings_json = String::new();
        settings_file.read_to_string(&mut settings_json)?;
        let settings: serde_json::Value = serde_json::from_str(&settings_json)?;
        if let Some(map) = settings.as_object() {
            let conn = Connection::open(db_path)?;
            for (key, value) in map {
                if let Some(v) = value.as_str() {
                    conn.execute(
                        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
                        rusqlite::params![key, v],
                    )?;
                }
            }
        }
    }

    // Run migrations per D-11
    let mut conn = Connection::open(db_path)?;
    crate::db::migrations::run_migrations(&mut conn)
        .with_context(|| "Failed to run migrations after ZIP import")?;

    // Check schema version compatibility
    check_schema_version(&conn)?;

    Ok(())
}

/// Checks that the imported database's schema version is not newer than
/// what our code supports. Per D-11: newer backups fail with clear message.
fn check_schema_version(conn: &Connection) -> Result<()> {
    // Our code's maximum supported user_version is the count of migrations.
    // If the imported DB has a higher user_version, it's from a newer app version.
    let current_version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);

    // Count our migrations to determine max supported version
    let max_version = count_migrations() as i64;

    if current_version > max_version {
        anyhow::bail!(
            "Backup is from a newer version of Revelry (schema v{}) than this app supports (schema v{}). Please update the application before importing this backup.",
            current_version,
            max_version
        );
    }

    Ok(())
}

/// Counts the number of migrations in our migration set.
fn count_migrations() -> usize {
    // We know the current migration count from the migration definitions.
    // This is a simple constant that must be updated when new migrations are added.
    // M1 through M9 = 9 migrations
    9
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
    fn test_export_sql_dump_creates_file() {
        let conn = setup_test_db();
        let dir = std::env::temp_dir().join("revelry_test_sql_dump");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_dump.sql");

        let result = export_sql_dump(&conn, &path);
        assert!(result.is_ok(), "SQL export should succeed: {:?}", result);
        assert!(path.exists(), "SQL dump file should exist");

        // Verify the file contains key SQL elements
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("BEGIN TRANSACTION"));
        assert!(content.contains("COMMIT"));
        assert!(content.contains("DROP TABLE IF EXISTS providers"));
        assert!(content.contains("CREATE TABLE"));

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_export_sql_dump_includes_data() {
        let conn = setup_test_db();
        // Insert a provider
        conn.execute(
            "INSERT INTO providers (id, name, provider_type, base_url, enabled, created_at, updated_at, model_name)
             VALUES ('test-id', 'Test Provider', '\"OpenAI\"', NULL, 1, '2026-01-01 00:00:00', '2026-01-01 00:00:00', 'gpt-4')",
            [],
        ).unwrap();

        let dir = std::env::temp_dir().join("revelry_test_sql_data");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_dump_data.sql");

        export_sql_dump(&conn, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Test Provider"), "SQL dump should contain inserted data");

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_import_sql_dump_round_trip() {
        let dir = std::env::temp_dir().join("revelry_test_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Create source DB with data
        let src_path = dir.join("src.db");
        let mut src_conn = Connection::open(&src_path).unwrap();
        run_migrations(&mut src_conn).unwrap();
        src_conn.execute(
            "INSERT INTO providers (id, name, provider_type, base_url, enabled, created_at, updated_at, model_name)
             VALUES ('rt-id', 'RoundTrip Provider', '\"Anthropic\"', NULL, 1, '2026-01-01 00:00:00', '2026-01-01 00:00:00', 'claude-3')",
            [],
        ).unwrap();

        // Export
        let sql_path = dir.join("roundtrip.sql");
        export_sql_dump(&src_conn, &sql_path).unwrap();

        // Import to new location
        let dst_path = dir.join("dst.db");
        import_sql_dump(&dst_path, &sql_path).unwrap();

        // Verify data
        let dst_conn = Connection::open(&dst_path).unwrap();
        let name: String = dst_conn
            .query_row("SELECT name FROM providers WHERE id = 'rt-id'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "RoundTrip Provider");

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_export_app_settings_json() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('test_key', 'test_value')",
            [],
        ).unwrap();

        let json = export_app_settings_json(&conn).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["test_key"], "test_value");
    }

    #[test]
    fn test_check_schema_version_rejects_newer() {
        let conn = setup_test_db();
        // Set user_version higher than our migrations support
        conn.execute("PRAGMA user_version = 99999", []).unwrap();
        let result = check_schema_version(&conn);
        assert!(result.is_err(), "Should reject newer schema version");
        assert!(result.unwrap_err().to_string().contains("newer version"));
    }

    #[test]
    fn test_check_schema_version_accepts_current() {
        let conn = setup_test_db();
        // user_version is set by migrations — should match current
        let result = check_schema_version(&conn);
        assert!(result.is_ok(), "Should accept current schema version: {:?}", result);
    }
}
