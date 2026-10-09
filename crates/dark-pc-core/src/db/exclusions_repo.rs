use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserExclusionRow {
    pub id: i64,
    pub path: String,
    pub created_at: String,
}

pub fn list_user_exclusions(conn: &Connection) -> Result<Vec<UserExclusionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, created_at FROM user_exclusions ORDER BY datetime(created_at) ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(UserExclusionRow {
                id: row.get(0)?,
                path: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn find_by_path(conn: &Connection, path: &str) -> Result<Option<UserExclusionRow>> {
    conn.query_row(
        "SELECT id, path, created_at FROM user_exclusions WHERE path = ?1 COLLATE NOCASE",
        [path],
        |row| {
            Ok(UserExclusionRow {
                id: row.get(0)?,
                path: row.get(1)?,
                created_at: row.get(2)?,
            })
        },
    )
    .optional()
    .context("recherche exclusion utilisateur")
}

pub fn insert_user_exclusion(conn: &Connection, path: &str) -> Result<UserExclusionRow> {
    let created_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO user_exclusions (path, created_at) VALUES (?1, ?2)",
        (path, created_at.as_str()),
    )?;
    find_by_path(conn, path)?.context("lecture exclusion insérée")
}

pub fn remove_user_exclusion(conn: &Connection, id: i64) -> Result<bool> {
    let n = conn.execute("DELETE FROM user_exclusions WHERE id = ?1", [id])?;
    Ok(n > 0)
}

pub fn clear_user_exclusions(conn: &Connection) -> Result<u64> {
    let n = conn.execute("DELETE FROM user_exclusions", [])?;
    Ok(n as u64)
}
