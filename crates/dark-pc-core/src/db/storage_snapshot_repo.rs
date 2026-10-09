use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSnapshotRow {
    pub id: i64,
    pub mount_path: String,
    pub total_bytes: i64,
    pub used_bytes: i64,
    pub free_bytes: i64,
    pub usage_percent: f64,
    pub created_at: String,
}

pub fn insert(
    conn: &Connection,
    mount_path: &str,
    total_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    usage_percent: f32,
) -> Result<i64> {
    let created_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO storage_snapshots
         (mount_path, total_bytes, used_bytes, free_bytes, usage_percent, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            mount_path,
            total_bytes as i64,
            used_bytes as i64,
            free_bytes as i64,
            f64::from(usage_percent),
            created_at.as_str(),
        ),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn latest_created_at(conn: &Connection, mount_path: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT created_at FROM storage_snapshots
         WHERE mount_path = ?1
         ORDER BY datetime(created_at) DESC
         LIMIT 1",
        [mount_path],
        |row| row.get(0),
    )
    .optional()
    .context("dernier snapshot volume")
}

pub fn list_distinct_mount_paths(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT mount_path FROM storage_snapshots ORDER BY mount_path ASC",
    )?;
    let paths = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(paths)
}

/// Snapshots pour graphique : filtre volume + depuis `since` (RFC3339), ordre chronologique croissant.
pub fn list_history(
    conn: &Connection,
    mount_path: &str,
    since: &str,
    limit: u32,
) -> Result<Vec<StorageSnapshotRow>> {
    let limit = limit.max(1).min(2_000) as i64;
    let mut stmt = conn.prepare(
        "SELECT id, mount_path, total_bytes, used_bytes, free_bytes, usage_percent, created_at
         FROM storage_snapshots
         WHERE mount_path = ?1 COLLATE NOCASE
           AND datetime(created_at) >= datetime(?2)
         ORDER BY datetime(created_at) ASC
         LIMIT ?3",
    )?;
    let rows = stmt
        .query_map((mount_path, since, limit), map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_recent(conn: &Connection, limit: u32, mount_path: Option<&str>) -> Result<Vec<StorageSnapshotRow>> {
    let limit = limit.max(1).min(5_000) as i64;
    if let Some(mount) = mount_path {
        let mut stmt = conn.prepare(
            "SELECT id, mount_path, total_bytes, used_bytes, free_bytes, usage_percent, created_at
             FROM storage_snapshots
             WHERE mount_path = ?1
             ORDER BY datetime(created_at) DESC
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map((mount, limit), map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, mount_path, total_bytes, used_bytes, free_bytes, usage_percent, created_at
             FROM storage_snapshots
             ORDER BY datetime(created_at) DESC
             LIMIT ?1",
        )?;
        let rows = stmt
            .query_map([limit], map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn history_is_chronological_and_filters_volume() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn();
        insert(conn, "E:\\", 100, 50, 50, 50.0).unwrap();
        conn.execute(
            "UPDATE storage_snapshots SET created_at = '2024-01-01T10:00:00Z' WHERE id = last_insert_rowid()",
            [],
        )
        .unwrap();
        insert(conn, "E:\\", 100, 60, 40, 60.0).unwrap();
        conn.execute(
            "UPDATE storage_snapshots SET created_at = '2024-01-02T10:00:00Z' WHERE id = last_insert_rowid()",
            [],
        )
        .unwrap();
        insert(conn, "F:\\", 100, 10, 90, 10.0).unwrap();
        let since = "1970-01-01T00:00:00Z";
        let rows = list_history(conn, "E:\\", since, 50).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].used_bytes, 50);
        assert_eq!(rows[1].used_bytes, 60);
        let f = list_history(conn, "F:\\", since, 10).unwrap();
        assert_eq!(f.len(), 1);
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StorageSnapshotRow> {
    Ok(StorageSnapshotRow {
        id: row.get(0)?,
        mount_path: row.get(1)?,
        total_bytes: row.get(2)?,
        used_bytes: row.get(3)?,
        free_bytes: row.get(4)?,
        usage_percent: row.get(5)?,
        created_at: row.get(6)?,
    })
}
