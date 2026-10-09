use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub id: i64,
    pub operation_type: String,
    pub payload_json: String,
    pub recoverable_bytes: Option<i64>,
    pub reversible: bool,
    pub created_at: String,
}

pub fn insert_operation(
    conn: &Connection,
    operation_type: &str,
    payload_json: &str,
    recoverable_bytes: Option<i64>,
    reversible: bool,
) -> Result<i64> {
    let created_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO operations (type, payload_json, recoverable_bytes, reversible, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            operation_type,
            payload_json,
            recoverable_bytes,
            reversible as i64,
            created_at.as_str(),
        ),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_operations(conn: &Connection, limit: u32) -> Result<Vec<OperationRecord>> {
    let limit = limit.max(1).min(500) as i64;
    let mut stmt = conn.prepare(
        "SELECT id, type, payload_json, recoverable_bytes, reversible, created_at
         FROM operations
         ORDER BY datetime(created_at) DESC
         LIMIT ?1",
    )?;
    let rows = stmt
        .query_map([limit], |row| {
            Ok(OperationRecord {
                id: row.get(0)?,
                operation_type: row.get(1)?,
                payload_json: row.get(2)?,
                recoverable_bytes: row.get(3)?,
                reversible: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn remove_files_by_paths(conn: &mut Connection, paths: &[String]) -> Result<usize> {
    if paths.is_empty() {
        return Ok(0);
    }
    let tx = conn.transaction().context("transaction suppression index")?;
    let mut deleted = 0usize;
    for path in paths {
        deleted += tx.execute("DELETE FROM files WHERE path = ?1", [path])?;
    }
    tx.commit()?;
    Ok(deleted)
}
