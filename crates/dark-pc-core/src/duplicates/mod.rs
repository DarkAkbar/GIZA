//! MODULE 3 — Duplicate Engine

mod engine;

pub use engine::{compute_duplicates, resolve_scan_id, DuplicateProgress};

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::duplicate_repo::{self, DuplicateGroupRow, DuplicateMemberRow};
use crate::large_files::ScanSummaryDto;
use crate::settings::{self, DuplicateKeepRule};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroupsResult {
    pub scan: Option<ScanSummaryDto>,
    pub group_count: u64,
    pub recoverable_bytes: u64,
    pub groups: Vec<DuplicateGroupRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroupDetails {
    pub group: DuplicateGroupRow,
    pub members: Vec<DuplicateMemberRow>,
}

pub fn list_duplicate_groups(conn: &Connection, scan_id: Option<i64>, limit: u32) -> Result<DuplicateGroupsResult> {
    let resolved = engine::resolve_scan_id(conn, scan_id)?;
    let Some(scan_id) = resolved else {
        return Ok(DuplicateGroupsResult {
            scan: None,
            group_count: 0,
            recoverable_bytes: 0,
            groups: Vec::new(),
        });
    };

    let scan = scan_by_id(conn, scan_id)?.map(ScanSummaryDto::from);

    let groups = duplicate_repo::list_groups(conn, scan_id, limit)?;
    let (group_count, recoverable_bytes) = duplicate_repo::duplicate_summary(conn, scan_id)?;

    Ok(DuplicateGroupsResult {
        scan,
        group_count,
        recoverable_bytes,
        groups,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateMergeItem {
    pub group_id: i64,
    pub path: String,
    pub size_bytes: u64,
    pub keep_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateMergePlan {
    pub scan_id: i64,
    pub to_remove: Vec<DuplicateMergeItem>,
    pub total_bytes: u64,
}

/// Chemins à envoyer à la corbeille pour fusionner ; la copie conservée suit `duplicate_keep_rule`.
pub fn duplicate_merge_plan(conn: &Connection, scan_id: Option<i64>) -> Result<Option<DuplicateMergePlan>> {
    let resolved = engine::resolve_scan_id(conn, scan_id)?;
    let Some(scan_id) = resolved else {
        return Ok(None);
    };
    let keep_rule = settings::load(conn)?.duplicate_keep_rule;
    duplicate_merge_plan_with_rule(conn, scan_id, keep_rule)
}

pub fn duplicate_merge_plan_with_rule(
    conn: &Connection,
    scan_id: i64,
    keep_rule: DuplicateKeepRule,
) -> Result<Option<DuplicateMergePlan>> {
    let groups = duplicate_repo::list_groups(conn, scan_id, 2_000)?;
    let mut to_remove = Vec::new();
    let mut total_bytes = 0_u64;
    for group in groups {
        let members = duplicate_repo::list_members(conn, group.id)?;
        if members.len() < 2 {
            continue;
        }
        let keep = pick_keep_member(&members, keep_rule);
        let keep_path = keep.path.clone();
        for member in &members {
            if member.path == keep_path {
                continue;
            }
            let size = member.size_bytes.max(0) as u64;
            total_bytes = total_bytes.saturating_add(size);
            to_remove.push(DuplicateMergeItem {
                group_id: group.id,
                path: member.path.clone(),
                size_bytes: size,
                keep_path: keep_path.clone(),
            });
        }
    }
    Ok(Some(DuplicateMergePlan {
        scan_id,
        to_remove,
        total_bytes,
    }))
}

fn pick_keep_member<'a>(members: &'a [DuplicateMemberRow], rule: DuplicateKeepRule) -> &'a DuplicateMemberRow {
    match rule {
        DuplicateKeepRule::ShortestPath => members
            .iter()
            .min_by_key(|m| (m.path.len(), m.path.as_str()))
            .expect("members non vide"),
        DuplicateKeepRule::Newest => members
            .iter()
            .max_by(|a, b| compare_modified_desc(a.modified_at.as_deref(), b.modified_at.as_deref()))
            .expect("members non vide"),
        DuplicateKeepRule::Oldest => members
            .iter()
            .min_by(|a, b| compare_modified_desc(a.modified_at.as_deref(), b.modified_at.as_deref()))
            .expect("members non vide"),
    }
}

fn compare_modified_desc(a: Option<&str>, b: Option<&str>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(b),
    }
}

pub fn get_duplicate_group_details(conn: &Connection, group_id: i64) -> Result<Option<DuplicateGroupDetails>> {
    let group: Option<DuplicateGroupRow> = conn
        .query_row(
            "SELECT id, scan_id, content_hash, file_count, total_bytes, recoverable_bytes
             FROM duplicate_groups WHERE id = ?1",
            [group_id],
            |row| {
                Ok(DuplicateGroupRow {
                    id: row.get(0)?,
                    scan_id: row.get(1)?,
                    content_hash: row.get(2)?,
                    file_count: row.get(3)?,
                    total_bytes: row.get(4)?,
                    recoverable_bytes: row.get(5)?,
                })
            },
        )
        .optional()?;

    let Some(group) = group else {
        return Ok(None);
    };
    let members = duplicate_repo::list_members(conn, group_id)?;
    Ok(Some(DuplicateGroupDetails { group, members }))
}

pub fn scan_by_id(conn: &Connection, scan_id: i64) -> Result<Option<crate::db::scan_repo::ScanSummary>> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT id, root_path, files_seen, bytes_total, finished_at
         FROM scans WHERE id = ?1 AND status = 'completed'",
        [scan_id],
        |row| {
            Ok(crate::db::scan_repo::ScanSummary {
                id: row.get(0)?,
                root_path: row.get(1)?,
                files_seen: row.get(2)?,
                bytes_total: row.get(3)?,
                finished_at: row.get(4)?,
            })
        },
    )
    .optional()
    .context("scan par id")
}

use anyhow::Context;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::AtomicBool;

    use super::*;
    use crate::db::scan_repo::{self, NewScanFile};
    use crate::db::Database;
    fn insert_file(conn: &mut Connection, scan_id: i64, path: &str, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
        let name = path.rsplit('\\').next().unwrap_or(path).to_string();
        let row = NewScanFile {
            path: path.into(),
            name,
            extension: Some("bin".into()),
            size_bytes: bytes.len() as i64,
            created_at: None,
            modified_at: None,
            accessed_at: None,
            mime_or_kind: None,
            is_system_hint: false,
        };
        let tx = conn.transaction().unwrap();
        scan_repo::insert_files_batch(&tx, scan_id, std::slice::from_ref(&row)).unwrap();
        tx.commit().unwrap();
    }

    #[test]
    fn detects_identical_files() {
        let temp = tempfile::tempdir().unwrap();
        let data_dir = temp.path().join("data");
        fs::create_dir(&data_dir).unwrap();
        let path_a = data_dir.join("a.bin");
        let path_b = data_dir.join("b.bin");
        let payload = vec![7u8; 1024];

        let mut db = Database::open(temp.path().join("d.db")).unwrap();
        let conn = db.conn_mut();
        let scan_id = scan_repo::create_scan(conn, data_dir.to_str().unwrap()).unwrap();
        insert_file(conn, scan_id, path_a.to_str().unwrap(), &payload);
        insert_file(conn, scan_id, path_b.to_str().unwrap(), &payload);
        scan_repo::finish_scan(conn, scan_id, "completed", None).unwrap();

        let cancel = AtomicBool::new(false);
        compute_duplicates(conn, scan_id, &cancel, |_| {}).unwrap();

        let result = list_duplicate_groups(conn, Some(scan_id), 100).unwrap();
        assert_eq!(result.group_count, 1);
        assert_eq!(result.groups[0].file_count, 2);
        assert_eq!(result.recoverable_bytes, 1024);

        let plan = duplicate_merge_plan(conn, Some(scan_id))
            .unwrap()
            .expect("plan");
        assert_eq!(plan.to_remove.len(), 1);
        assert!(plan.to_remove[0].path.ends_with("b.bin"));
    }
}
