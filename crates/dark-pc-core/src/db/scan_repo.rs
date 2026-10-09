use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, Transaction};

#[derive(Debug, Clone)]
pub struct NewScanFile {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub accessed_at: Option<String>,
    pub mime_or_kind: Option<String>,
    pub is_system_hint: bool,
}

#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub id: i64,
    pub root_path: String,
    pub files_seen: i64,
    pub bytes_total: i64,
    pub finished_at: Option<String>,
}

pub fn create_scan(conn: &Connection, root_path: &str) -> Result<i64> {
    let started_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO scans (root_path, status, started_at) VALUES (?1, 'running', ?2)",
        [root_path, started_at.as_str()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_scan_progress(
    conn: &Connection,
    scan_id: i64,
    files_seen: i64,
    bytes_total: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE scans SET files_seen = ?1, bytes_total = ?2 WHERE id = ?3",
        (files_seen, bytes_total, scan_id),
    )?;
    Ok(())
}

pub fn finish_scan(
    conn: &Connection,
    scan_id: i64,
    status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    let finished_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE scans SET status = ?1, finished_at = ?2, error_message = ?3 WHERE id = ?4",
        (status, finished_at.as_str(), error_message, scan_id),
    )?;
    Ok(())
}

fn normalize_root_path(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches(['\\', '/']);
    #[cfg(windows)]
    {
        let upper = trimmed.to_ascii_uppercase();
        if upper.len() >= 2 && upper.as_bytes()[1] == b':' {
            return format!("{}:\\", upper.as_bytes()[0] as char);
        }
    }
    trimmed.to_string()
}

pub fn has_completed_scan_for_root(conn: &Connection, root_path: &str) -> Result<bool> {
    Ok(latest_completed_scan_for_root(conn, root_path)?.is_some())
}

pub fn latest_completed_scan_for_root(
    conn: &Connection,
    root_path: &str,
) -> Result<Option<ScanSummary>> {
    let target = normalize_root_path(root_path);
    let mut stmt = conn.prepare(
        "SELECT id, root_path, files_seen, bytes_total, finished_at
         FROM scans
         WHERE status = 'completed'
         ORDER BY datetime(finished_at) DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ScanSummary {
            id: row.get(0)?,
            root_path: row.get(1)?,
            files_seen: row.get(2)?,
            bytes_total: row.get(3)?,
            finished_at: row.get(4)?,
        })
    })?;
    for row in rows {
        let scan = row?;
        if normalize_root_path(&scan.root_path) == target {
            return Ok(Some(scan));
        }
    }
    Ok(None)
}

pub fn latest_completed_scan(conn: &Connection) -> Result<Option<ScanSummary>> {
    conn.query_row(
        "SELECT id, root_path, files_seen, bytes_total, finished_at
         FROM scans
         WHERE status = 'completed'
         ORDER BY datetime(finished_at) DESC
         LIMIT 1",
        [],
        |row| {
            Ok(ScanSummary {
                id: row.get(0)?,
                root_path: row.get(1)?,
                files_seen: row.get(2)?,
                bytes_total: row.get(3)?,
                finished_at: row.get(4)?,
            })
        },
    )
    .optional()
    .context("lecture du dernier scan terminé")
}

pub fn completed_scan_summary(conn: &Connection, scan_id: i64) -> Result<Option<ScanSummary>> {
    conn.query_row(
        "SELECT id, root_path, files_seen, bytes_total, finished_at
         FROM scans
         WHERE id = ?1 AND status = 'completed'",
        [scan_id],
        |row| {
            Ok(ScanSummary {
                id: row.get(0)?,
                root_path: row.get(1)?,
                files_seen: row.get(2)?,
                bytes_total: row.get(3)?,
                finished_at: row.get(4)?,
            })
        },
    )
    .optional()
    .context("scan terminé par identifiant")
}

pub fn insert_files_batch(
    tx: &Transaction<'_>,
    scan_id: i64,
    files: &[NewScanFile],
) -> Result<()> {
    let mut stmt = tx.prepare(
        "INSERT OR IGNORE INTO files (
            scan_id, path, name, extension, size_bytes,
            created_at, modified_at, accessed_at, mime_or_kind, is_system_hint
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    for file in files {
        stmt.execute(rusqlite::params![
            scan_id,
            file.path,
            file.name,
            file.extension,
            file.size_bytes,
            file.created_at,
            file.modified_at,
            file.accessed_at,
            file.mime_or_kind,
            file.is_system_hint as i64,
        ])?;
    }
    Ok(())
}
