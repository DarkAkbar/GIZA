//! Synthèse stockage pour le dashboard (dernier scan terminé).

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::duplicate_repo;
use crate::db::scan_repo;
use crate::large_files::ScanSummaryDto;
use crate::settings;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInsights {
    pub scan: Option<ScanSummaryDto>,
    pub has_duplicate_analysis: bool,
    pub duplicate_group_count: u64,
    pub duplicate_recoverable_bytes: u64,
    pub large_files_count: u64,
    pub large_files_total_bytes: u64,
}

pub fn get_storage_insights(conn: &Connection) -> Result<StorageInsights> {
    let scan = scan_repo::latest_completed_scan(conn)?;

    let Some(scan) = scan else {
        return Ok(StorageInsights {
            scan: None,
            has_duplicate_analysis: false,
            duplicate_group_count: 0,
            duplicate_recoverable_bytes: 0,
            large_files_count: 0,
            large_files_total_bytes: 0,
        });
    };

    let min_large = settings::large_file_threshold_bytes(conn)? as i64;
    let (large_files_count, large_files_total_bytes): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0)
         FROM files
         WHERE scan_id = ?1 AND size_bytes >= ?2",
        (scan.id, min_large),
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let has_duplicate_analysis = duplicate_repo::scan_has_duplicate_analysis(conn, scan.id)?;
    let (duplicate_group_count, duplicate_recoverable_bytes) = if has_duplicate_analysis {
        duplicate_repo::duplicate_summary(conn, scan.id)?
    } else {
        (0, 0)
    };

    Ok(StorageInsights {
        scan: Some(scan.into()),
        has_duplicate_analysis,
        duplicate_group_count,
        duplicate_recoverable_bytes,
        large_files_count: large_files_count as u64,
        large_files_total_bytes: large_files_total_bytes as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::scan_repo::{self, NewScanFile};
    #[test]
    fn aggregates_scan_duplicates_and_large_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = crate::db::Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn_mut();
        let scan_id = scan_repo::create_scan(&conn, "C:\\test").unwrap();
        let tx = conn.transaction().unwrap();
        scan_repo::insert_files_batch(
            &tx,
            scan_id,
            &[
                NewScanFile {
                    path: "C:\\test\\a.bin".into(),
                    name: "a.bin".into(),
                    extension: Some("bin".into()),
                    size_bytes: 200 * 1024 * 1024,
                    created_at: None,
                    modified_at: None,
                    accessed_at: None,
                    mime_or_kind: None,
                    is_system_hint: false,
                },
                NewScanFile {
                    path: "C:\\test\\b.bin".into(),
                    name: "b.bin".into(),
                    extension: Some("bin".into()),
                    size_bytes: 200 * 1024 * 1024,
                    created_at: None,
                    modified_at: None,
                    accessed_at: None,
                    mime_or_kind: None,
                    is_system_hint: false,
                },
            ],
        )
        .unwrap();
        tx.commit().unwrap();
        scan_repo::finish_scan(&conn, scan_id, "completed", None).unwrap();

        duplicate_repo::insert_group(&conn, scan_id, "hash", 2, 400 * 1024 * 1024, 200 * 1024 * 1024)
            .unwrap();

        let insights = get_storage_insights(&conn).unwrap();
        assert!(insights.scan.is_some());
        assert!(insights.has_duplicate_analysis);
        assert_eq!(insights.duplicate_group_count, 1);
        assert_eq!(insights.duplicate_recoverable_bytes, 200 * 1024 * 1024);
        assert_eq!(insights.large_files_count, 2);
    }
}
