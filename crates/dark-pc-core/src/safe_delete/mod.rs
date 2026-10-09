//! MODULE 9 — Safe Delete : corbeille système uniquement (V1).

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use trash;

use crate::db::operations_repo;
use crate::exclusions::{self, path_is_skipped};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashItemOutcome {
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashPayload {
    pub outcomes: Vec<TrashItemOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashResult {
    pub operation_id: i64,
    pub outcomes: Vec<TrashItemOutcome>,
    pub success_count: u32,
    pub failed_count: u32,
    pub total_bytes_trashed: u64,
}

pub fn validate_path_for_trash(conn: &Connection, path: &Path) -> Result<u64> {
    if !path.is_absolute() {
        bail!("chemin non absolu : {}", path.display());
    }

    let effective = exclusions::effective_from_connection(conn)?;
    if path_is_skipped(path, &effective) {
        bail!("chemin exclu (zone sensible) : {}", path.display());
    }

    let metadata = std::fs::metadata(path)
        .with_context(|| format!("fichier inaccessible : {}", path.display()))?;
    if !metadata.is_file() {
        bail!("n'est pas un fichier : {}", path.display());
    }

    Ok(metadata.len())
}

pub fn trash_files(conn: &mut Connection, paths: Vec<String>) -> Result<TrashResult> {
    if paths.is_empty() {
        bail!("aucun fichier sélectionné");
    }

    let mut outcomes = Vec::with_capacity(paths.len());
    let mut success_paths = Vec::new();
    let mut total_bytes = 0u64;

    for raw in paths {
        let path = PathBuf::from(&raw);
        match trash_single(conn, &path) {
            Ok(size) => {
                total_bytes = total_bytes.saturating_add(size);
                success_paths.push(raw.clone());
                outcomes.push(TrashItemOutcome {
                    path: raw,
                    success: true,
                    error: None,
                    size_bytes: size,
                });
            }
            Err(err) => outcomes.push(TrashItemOutcome {
                path: raw,
                success: false,
                error: Some(err.to_string()),
                size_bytes: 0,
            }),
        }
    }

    let success_count = outcomes.iter().filter(|o| o.success).count() as u32;
    let failed_count = outcomes.len() as u32 - success_count;

    if !success_paths.is_empty() {
        let _ = operations_repo::remove_files_by_paths(conn, &success_paths);
    }

    let payload = TrashPayload {
        outcomes: outcomes.clone(),
    };
    let payload_json = serde_json::to_string(&payload)?;
    let operation_id = operations_repo::insert_operation(
        conn,
        "trash",
        &payload_json,
        Some(total_bytes as i64),
        true,
    )?;

    Ok(TrashResult {
        operation_id,
        outcomes,
        success_count,
        failed_count,
        total_bytes_trashed: total_bytes,
    })
}

fn trash_single(conn: &Connection, path: &Path) -> Result<u64> {
    let size = validate_path_for_trash(conn, path)?;
    trash::delete(path).with_context(|| format!("corbeille : {}", path.display()))?;
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::fs;

    fn conn() -> Database {
        let dir = tempfile::tempdir().unwrap();
        Database::open(dir.path().join("t.db")).unwrap()
    }

    #[test]
    fn rejects_missing_file() {
        let db = conn();
        let err = validate_path_for_trash(db.conn(), Path::new("C:\\does-not-exist-dark-pc-test.bin"));
        assert!(err.is_err());
    }

    #[test]
    fn rejects_recycle_bin_segment() {
        let db = conn();
        let err = validate_path_for_trash(db.conn(), Path::new("C:\\$Recycle.Bin\\foo.txt"));
        assert!(err.is_err());
    }

    #[test]
    fn accepts_regular_file_in_temp() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("ok.txt");
        fs::write(&file, b"data").unwrap();
        let db = Database::open(dir.path().join("t.db")).unwrap();
        let size = validate_path_for_trash(db.conn(), &file).unwrap();
        assert_eq!(size, 4);
    }
}
