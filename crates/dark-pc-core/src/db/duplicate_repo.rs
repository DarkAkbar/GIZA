use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroupRow {
    pub id: i64,
    pub scan_id: i64,
    pub content_hash: String,
    pub file_count: i64,
    pub total_bytes: i64,
    pub recoverable_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateMemberRow {
    pub file_id: i64,
    pub path: String,
    pub name: String,
    pub size_bytes: i64,
    pub modified_at: Option<String>,
}

pub fn has_groups_for_scan(conn: &Connection, scan_id: i64) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duplicate_groups WHERE scan_id = ?1",
        [scan_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn clear_groups_for_scan(conn: &Connection, scan_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM duplicate_groups WHERE scan_id = ?1",
        [scan_id],
    )
    .context("suppression groupes doublons")?;
    Ok(())
}

pub fn insert_group(
    conn: &Connection,
    scan_id: i64,
    content_hash: &str,
    file_count: i64,
    total_bytes: i64,
    recoverable_bytes: i64,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO duplicate_groups (scan_id, content_hash, file_count, total_bytes, recoverable_bytes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (scan_id, content_hash, file_count, total_bytes, recoverable_bytes),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_member(conn: &Connection, group_id: i64, file_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO duplicate_group_members (group_id, file_id) VALUES (?1, ?2)",
        (group_id, file_id),
    )?;
    Ok(())
}

pub fn update_file_hash(conn: &Connection, file_id: i64, hash: &str) -> Result<()> {
    conn.execute(
        "UPDATE files SET content_hash = ?1 WHERE id = ?2",
        (hash, file_id),
    )?;
    Ok(())
}

pub fn list_groups(conn: &Connection, scan_id: i64, limit: u32) -> Result<Vec<DuplicateGroupRow>> {
    let limit = limit.max(1).min(2_000) as i64;
    let mut stmt = conn.prepare(
        "SELECT id, scan_id, content_hash, file_count, total_bytes, recoverable_bytes
         FROM duplicate_groups
         WHERE scan_id = ?1
         ORDER BY recoverable_bytes DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map((scan_id, limit), |row| {
            Ok(DuplicateGroupRow {
                id: row.get(0)?,
                scan_id: row.get(1)?,
                content_hash: row.get(2)?,
                file_count: row.get(3)?,
                total_bytes: row.get(4)?,
                recoverable_bytes: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_members(conn: &Connection, group_id: i64) -> Result<Vec<DuplicateMemberRow>> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.path, f.name, f.size_bytes, f.modified_at
         FROM duplicate_group_members m
         JOIN files f ON f.id = m.file_id
         WHERE m.group_id = ?1
         ORDER BY f.path ASC",
    )?;
    let rows = stmt
        .query_map([group_id], |row| {
            Ok(DuplicateMemberRow {
                file_id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                size_bytes: row.get(3)?,
                modified_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn duplicate_summary(conn: &Connection, scan_id: i64) -> Result<(u64, u64)> {
    let (groups, recoverable): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(recoverable_bytes), 0)
         FROM duplicate_groups WHERE scan_id = ?1",
        [scan_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok((groups as u64, recoverable as u64))
}

pub fn scan_has_duplicate_analysis(conn: &Connection, scan_id: i64) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duplicate_groups WHERE scan_id = ?1",
        [scan_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn sizes_with_multiple_files(conn: &Connection, scan_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT size_bytes
         FROM files
         WHERE scan_id = ?1
         GROUP BY size_bytes
         HAVING COUNT(*) > 1
         ORDER BY size_bytes DESC",
    )?;
    let sizes = stmt
        .query_map([scan_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sizes)
}

pub fn files_with_size(conn: &Connection, scan_id: i64, size_bytes: i64) -> Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, path FROM files WHERE scan_id = ?1 AND size_bytes = ?2 ORDER BY path ASC",
    )?;
    let rows = stmt
        .query_map((scan_id, size_bytes), |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn completed_scan_by_id(conn: &Connection, scan_id: i64) -> Result<bool> {
    let exists = conn
        .query_row(
            "SELECT 1 FROM scans WHERE id = ?1 AND status = 'completed'",
            [scan_id],
            |_| Ok(true),
        )
        .optional()?
        .is_some();
    Ok(exists)
}
