//! Derived vector index storage. Rows are recomputable from source content —
//! losing this table loses nothing a rebuild can't restore. All access goes
//! through the DbWorker actor like every other domain.

use crate::service::vector::{decode_vector, encode_vector};
use anyhow::Result;
use rusqlite::{params, Connection};

/// An embedding row as stored in SQLite.
#[derive(Clone, Debug)]
pub struct EmbeddingRow {
    pub namespace: String,
    pub ref_id: String,
    pub model_key: String,
    pub dims: i64,
    pub vector: Vec<u8>,
    pub content_hash: String,
}

/// A decoded embedding record returned by loads.
#[derive(Clone, Debug)]
pub struct EmbeddingRecord {
    pub ref_id: String,
    pub model_key: String,
    pub content_hash: String,
    pub vector: Vec<f32>,
}

impl EmbeddingRow {
    pub fn new(
        namespace: &str,
        ref_id: &str,
        model_key: &str,
        vector: &[f32],
        content_hash: &str,
    ) -> Self {
        Self {
            namespace: namespace.to_string(),
            ref_id: ref_id.to_string(),
            model_key: model_key.to_string(),
            dims: vector.len() as i64,
            vector: encode_vector(vector),
            content_hash: content_hash.to_string(),
        }
    }
}

/// Inserts or replaces an embedding row.
pub fn upsert_embedding(conn: &Connection, row: &EmbeddingRow) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO embeddings (namespace, ref_id, model_key, dims, vector, content_hash, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(namespace, ref_id) DO UPDATE SET
            model_key = excluded.model_key,
            dims = excluded.dims,
            vector = excluded.vector,
            content_hash = excluded.content_hash,
            updated_at = excluded.updated_at",
    )?
    .execute(params![
        row.namespace,
        row.ref_id,
        row.model_key,
        row.dims,
        &row.vector,
        row.content_hash,
        jiff::Timestamp::now().to_string(),
    ])?;
    Ok(())
}

/// Deletes a single embedding row by its source ref (message id / entry id are
/// UUIDs, so ref_id alone is unambiguous).
pub fn delete_embedding_by_ref(conn: &Connection, ref_id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM embeddings WHERE ref_id = ?1")?
        .execute(params![ref_id])?;
    Ok(())
}

/// Deletes every row in a namespace (conversation / lorebook cascade).
pub fn delete_embeddings_by_namespace(conn: &Connection, namespace: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM embeddings WHERE namespace = ?1")?
        .execute(params![namespace])?;
    Ok(())
}

/// Deletes every row produced by a given model (rebuild cleanup).
pub fn delete_embeddings_by_model(conn: &Connection, model_key: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM embeddings WHERE model_key = ?1")?
        .execute(params![model_key])?;
    Ok(())
}

/// Loads one embedding record (indexed by primary key). Returns None when the
/// ref has no row for the given model.
pub fn get_embedding(
    conn: &Connection,
    namespace: &str,
    ref_id: &str,
) -> Result<Option<EmbeddingRecord>> {
    let mut stmt = conn.prepare_cached(
        "SELECT ref_id, model_key, content_hash, vector FROM embeddings
         WHERE namespace = ?1 AND ref_id = ?2",
    )?;
    let mut rows = stmt.query_map(params![namespace, ref_id], row_to_record)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<EmbeddingRecord> {
    let vector_bytes: Vec<u8> = row.get("vector")?;
    Ok(EmbeddingRecord {
        ref_id: row.get("ref_id")?,
        model_key: row.get("model_key")?,
        content_hash: row.get("content_hash")?,
        vector: decode_vector(&vector_bytes),
    })
}

/// Loads embedding records for one namespace. When `model_key` is given, only
/// rows from that model are returned (stale rows are skipped by the caller).
pub fn load_embeddings(
    conn: &Connection,
    namespace: &str,
    model_key: Option<&str>,
) -> Result<Vec<EmbeddingRecord>> {
    let sql = match model_key {
        Some(_) => {
            "SELECT ref_id, model_key, content_hash, vector FROM embeddings
             WHERE namespace = ?1 AND model_key = ?2"
        }
        None => {
            "SELECT ref_id, model_key, content_hash, vector FROM embeddings
             WHERE namespace = ?1"
        }
    };
    let mut stmt = conn.prepare_cached(sql)?;
    let records = match model_key {
        Some(key) => stmt
            .query_map(params![namespace, key], row_to_record)?
            .collect::<Result<Vec<_>, _>>()?,
        None => stmt
            .query_map(params![namespace], row_to_record)?
            .collect::<Result<Vec<_>, _>>()?,
    };
    Ok(records)
}

/// Loads embedding records across several namespaces in one round trip
/// (semantic lorebook activation loads all active lorebook namespaces).
/// Namespaces with no rows are simply absent from the result map.
pub fn load_embeddings_for_namespaces(
    conn: &Connection,
    namespaces: &[String],
    model_key: Option<&str>,
) -> Result<std::collections::HashMap<String, Vec<EmbeddingRecord>>> {
    let mut result = std::collections::HashMap::new();
    if namespaces.is_empty() {
        return Ok(result);
    }

    // Dynamic IN clause: WHERE namespace IN (?1..?N) [AND model_key = ?N+1]
    let mut placeholders: Vec<String> = Vec::with_capacity(namespaces.len());
    for i in 0..namespaces.len() {
        placeholders.push(format!("?{}", i + 1));
    }
    let mut sql = format!(
        "SELECT namespace, ref_id, model_key, content_hash, vector FROM embeddings
         WHERE namespace IN ({})",
        placeholders.join(", ")
    );
    let mut bind_values: Vec<String> = namespaces.to_vec();
    if let Some(key) = model_key {
        sql.push_str(&format!(" AND model_key = ?{}", namespaces.len() + 1));
        bind_values.push(key.to_string());
    }

    let params: Vec<&dyn rusqlite::types::ToSql> = bind_values
        .iter()
        .map(|v| v as &dyn rusqlite::types::ToSql)
        .collect();

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params.as_slice())?;
    while let Some(row) = rows.next()? {
        let namespace: String = row.get("namespace")?;
        let record = row_to_record(row)?;
        result.entry(namespace).or_default().push(record);
    }
    Ok(result)
}

/// Counts rows grouped by (model_key, namespace) for status display.
pub fn count_embeddings(conn: &Connection) -> Result<Vec<(String, String, i64)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT model_key, namespace, COUNT(*) FROM embeddings GROUP BY model_key, namespace",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
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
    use crate::service::vector::content_hash;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    fn sample_row(namespace: &str, ref_id: &str, model_key: &str, vector: &[f32]) -> EmbeddingRow {
        EmbeddingRow::new(
            namespace,
            ref_id,
            model_key,
            vector,
            &content_hash("sample"),
        )
    }

    #[test]
    fn test_upsert_and_load() {
        let conn = setup_test_db();
        let row = sample_row("messages:conv-1", "msg-1", "local:test", &[0.1, 0.2, 0.3]);
        upsert_embedding(&conn, &row).expect("upsert should succeed");

        let loaded = load_embeddings(&conn, "messages:conv-1", Some("local:test"))
            .expect("load should succeed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].ref_id, "msg-1");
        assert_eq!(loaded[0].vector.len(), 3);
        assert!((loaded[0].vector[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_upsert_replaces() {
        let conn = setup_test_db();
        upsert_embedding(&conn, &sample_row("ns", "r1", "m1", &[1.0])).expect("first upsert");
        upsert_embedding(&conn, &sample_row("ns", "r1", "m1", &[1.0, 2.0])).expect("second upsert");

        let loaded = load_embeddings(&conn, "ns", Some("m1")).expect("load");
        assert_eq!(loaded.len(), 1, "primary key (namespace, ref_id) replaces");
        assert_eq!(loaded[0].vector.len(), 2);
    }

    #[test]
    fn test_load_filters_by_model() {
        let conn = setup_test_db();
        upsert_embedding(&conn, &sample_row("ns", "r1", "model-a", &[1.0])).expect("upsert a");
        upsert_embedding(&conn, &sample_row("ns", "r2", "model-b", &[1.0])).expect("upsert b");

        let only_a = load_embeddings(&conn, "ns", Some("model-a")).expect("load");
        assert_eq!(only_a.len(), 1);
        assert_eq!(only_a[0].ref_id, "r1");

        let all = load_embeddings(&conn, "ns", None).expect("load");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_delete_by_ref() {
        let conn = setup_test_db();
        upsert_embedding(&conn, &sample_row("ns-1", "r1", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("ns-2", "r1", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("ns-1", "r2", "m", &[1.0])).expect("upsert");

        delete_embedding_by_ref(&conn, "r1").expect("delete");
        assert_eq!(load_embeddings(&conn, "ns-1", None).expect("l").len(), 1);
        assert_eq!(load_embeddings(&conn, "ns-2", None).expect("l").len(), 0);
    }

    #[test]
    fn test_delete_by_namespace_and_model() {
        let conn = setup_test_db();
        upsert_embedding(&conn, &sample_row("ns-1", "r1", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("ns-1", "r2", "old-model", &[1.0])).expect("upsert");

        delete_embeddings_by_model(&conn, "old-model").expect("delete by model");
        assert_eq!(load_embeddings(&conn, "ns-1", None).expect("l").len(), 1);

        delete_embeddings_by_namespace(&conn, "ns-1").expect("delete by ns");
        assert_eq!(load_embeddings(&conn, "ns-1", None).expect("l").len(), 0);
    }

    #[test]
    fn test_load_multiple_namespaces() {
        let conn = setup_test_db();
        upsert_embedding(&conn, &sample_row("lorebook:lb-1", "e1", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("lorebook:lb-1", "e2", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("lorebook:lb-2", "e3", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("lorebook:lb-2", "e4", "stale", &[1.0]))
            .expect("upsert");

        let map = load_embeddings_for_namespaces(
            &conn,
            &[
                "lorebook:lb-1".to_string(),
                "lorebook:lb-2".to_string(),
                "lorebook:missing".to_string(),
            ],
            Some("m"),
        )
        .expect("load multi");

        assert_eq!(map.get("lorebook:lb-1").map(|v| v.len()), Some(2));
        assert_eq!(map.get("lorebook:lb-2").map(|v| v.len()), Some(1));
        assert!(!map.contains_key("lorebook:missing"));
    }

    #[test]
    fn test_count_embeddings() {
        let conn = setup_test_db();
        upsert_embedding(&conn, &sample_row("ns-1", "r1", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("ns-1", "r2", "m", &[1.0])).expect("upsert");
        upsert_embedding(&conn, &sample_row("ns-2", "r3", "other", &[1.0])).expect("upsert");

        let counts = count_embeddings(&conn).expect("count");
        assert_eq!(counts.len(), 2);
        assert!(counts.contains(&("m".to_string(), "ns-1".to_string(), 2)));
        assert!(counts.contains(&("other".to_string(), "ns-2".to_string(), 1)));
    }
}
