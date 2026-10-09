//! MODULE 4 — Large File Analyzer

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::scan_repo::{self, ScanSummary};

pub const DEFAULT_LIMIT: u32 = 500;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LargeFileQuery {
    pub min_size_bytes: u64,
    pub scan_id: Option<i64>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeFileItem {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
    pub mime_or_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeFilesResult {
    pub scan: Option<ScanSummaryDto>,
    pub min_size_bytes: u64,
    pub match_count: u64,
    pub match_total_bytes: u64,
    pub items: Vec<LargeFileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummaryDto {
    pub id: i64,
    pub root_path: String,
    pub files_seen: i64,
    pub bytes_total: i64,
    pub finished_at: Option<String>,
}

impl From<ScanSummary> for ScanSummaryDto {
    fn from(value: ScanSummary) -> Self {
        Self {
            id: value.id,
            root_path: value.root_path,
            files_seen: value.files_seen,
            bytes_total: value.bytes_total,
            finished_at: value.finished_at,
        }
    }
}

pub fn list_large_files(conn: &Connection, query: LargeFileQuery) -> Result<LargeFilesResult> {
    let limit = query.limit.max(1).min(5_000);
    let scan = if let Some(scan_id) = query.scan_id {
        scan_by_id(conn, scan_id)?
    } else {
        scan_repo::latest_completed_scan(conn)?
    };

    let Some(scan) = scan else {
        return Ok(LargeFilesResult {
            scan: None,
            min_size_bytes: query.min_size_bytes,
            match_count: 0,
            match_total_bytes: 0,
            items: Vec::new(),
        });
    };

    let min_size = query.min_size_bytes as i64;
    let (match_count, match_total_bytes): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0)
         FROM files
         WHERE scan_id = ?1 AND size_bytes >= ?2",
        (scan.id, min_size),
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let mut stmt = conn
        .prepare(
            "SELECT id, path, name, extension, size_bytes, modified_at, mime_or_kind
             FROM files
             WHERE scan_id = ?1 AND size_bytes >= ?2
             ORDER BY size_bytes DESC
             LIMIT ?3",
        )
        .context("préparation requête gros fichiers")?;

    let items = stmt
        .query_map((scan.id, min_size, limit), |row| {
            Ok(LargeFileItem {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                extension: row.get(3)?,
                size_bytes: row.get::<_, i64>(4)? as u64,
                modified_at: row.get(5)?,
                mime_or_kind: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(LargeFilesResult {
        scan: Some(scan.into()),
        min_size_bytes: query.min_size_bytes,
        match_count: match_count as u64,
        match_total_bytes: match_total_bytes as u64,
        items,
    })
}

fn scan_by_id(conn: &Connection, scan_id: i64) -> Result<Option<ScanSummary>> {
    conn.query_row(
        "SELECT id, root_path, files_seen, bytes_total, finished_at
         FROM scans WHERE id = ?1 AND status = 'completed'",
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
    .context("scan par identifiant")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::scan_repo::{self, NewScanFile};

    fn insert_file(conn: &mut Connection, scan_id: i64, path: &str, size: i64) {
        let name = path.rsplit('\\').next().or_else(|| path.rsplit('/').next()).unwrap_or(path);
        let row = NewScanFile {
            path: path.into(),
            name: name.into(),
            extension: Some("bin".into()),
            size_bytes: size,
            created_at: None,
            modified_at: Some("2024-01-01T00:00:00Z".into()),
            accessed_at: None,
            mime_or_kind: Some("application/octet-stream".into()),
            is_system_hint: false,
        };
        let tx = conn.transaction().unwrap();
        scan_repo::insert_files_batch(&tx, scan_id, std::slice::from_ref(&row)).unwrap();
        tx.commit().unwrap();
    }

    #[test]
    fn filters_by_min_size_desc() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = crate::db::Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn_mut();
        let scan_id = scan_repo::create_scan(conn, "C:\\test").unwrap();
        scan_repo::finish_scan(conn, scan_id, "completed", None).unwrap();
        insert_file(conn, scan_id, "C:\\test\\small.bin", 10 * 1024 * 1024);
        insert_file(conn, scan_id, "C:\\test\\big.bin", 200 * 1024 * 1024);

        let result = list_large_files(
            conn,
            LargeFileQuery {
                min_size_bytes: 100 * 1024 * 1024,
                scan_id: Some(scan_id),
                limit: 50,
            },
        )
        .unwrap();

        assert_eq!(result.match_count, 1);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].name, "big.bin");
    }
}
