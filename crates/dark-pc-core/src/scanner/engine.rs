use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::db::scan_repo::{self, NewScanFile};
use crate::db::Database;
use crate::exclusions::{path_is_skipped, system_only_effective, EffectiveExclusions};
use rusqlite::Connection;

const BATCH_SIZE: usize = 256;
const PROGRESS_EVERY: u64 = 400;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanProgress {
    pub scan_id: i64,
    pub files_seen: u64,
    pub bytes_total: u64,
    pub current_path: Option<String>,
}

pub struct ScanOptions {
    pub root: PathBuf,
    pub exclusions: EffectiveExclusions,
}

impl ScanOptions {
    pub fn with_default_exclusions(root: PathBuf) -> Self {
        Self {
            root,
            exclusions: system_only_effective(),
        }
    }

    pub fn from_connection(conn: &Connection, root: PathBuf) -> Result<Self> {
        Ok(Self {
            root,
            exclusions: crate::exclusions::effective_from_connection(conn)?,
        })
    }
}

pub fn run_scan(
    db: &mut Database,
    scan_id: i64,
    options: ScanOptions,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(ScanProgress),
) -> Result<()> {
    let root = options.root.canonicalize().unwrap_or(options.root.clone());
    if !root.is_dir() {
        anyhow::bail!("le chemin n'est pas un dossier : {}", root.display());
    }

    let conn = db.conn_mut();
    let mut batch: Vec<NewScanFile> = Vec::with_capacity(BATCH_SIZE);
    let mut files_seen: u64 = 0;
    let mut bytes_total: u64 = 0;
    let mut last_progress_at = 0u64;

    let walker = WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !path_is_skipped(entry.path(), &options.exclusions));

    for entry in walker {
        if cancel.load(Ordering::Relaxed) {
            flush_batch(conn, scan_id, &mut batch)?;
            scan_repo::update_scan_progress(
                conn,
                scan_id,
                files_seen as i64,
                bytes_total as i64,
            )?;
            scan_repo::finish_scan(conn, scan_id, "cancelled", None)?;
            return Ok(());
        }

        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(error = %err, "entrée ignorée pendant le scan");
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(err) => {
                tracing::warn!(path = %path.display(), error = %err, "métadonnées indisponibles");
                continue;
            }
        };

        let size_bytes = metadata.len();
        let file_row = build_file_row(path, size_bytes, &metadata);
        bytes_total = bytes_total.saturating_add(size_bytes);
        files_seen += 1;
        batch.push(file_row);

        if batch.len() >= BATCH_SIZE {
            flush_batch(conn, scan_id, &mut batch)?;
        }

        if files_seen - last_progress_at >= PROGRESS_EVERY {
            last_progress_at = files_seen;
            scan_repo::update_scan_progress(
                conn,
                scan_id,
                files_seen as i64,
                bytes_total as i64,
            )?;
            on_progress(ScanProgress {
                scan_id,
                files_seen,
                bytes_total,
                current_path: Some(path.display().to_string()),
            });
        }
    }

    flush_batch(conn, scan_id, &mut batch)?;
    scan_repo::update_scan_progress(
        conn,
        scan_id,
        files_seen as i64,
        bytes_total as i64,
    )?;
    scan_repo::finish_scan(conn, scan_id, "completed", None)?;
    on_progress(ScanProgress {
        scan_id,
        files_seen,
        bytes_total,
        current_path: None,
    });
    Ok(())
}

fn flush_batch(conn: &mut rusqlite::Connection, scan_id: i64, batch: &mut Vec<NewScanFile>) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let tx = conn
        .transaction()
        .context("transaction insertion fichiers")?;
    scan_repo::insert_files_batch(&tx, scan_id, batch)?;
    tx.commit()?;
    batch.clear();
    Ok(())
}

fn build_file_row(path: &Path, size_bytes: u64, metadata: &std::fs::Metadata) -> NewScanFile {
    let path_str = path.display().to_string();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path_str.clone());
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().into_owned());
    let mime_or_kind = mime_guess::from_path(path)
        .first_raw()
        .map(str::to_string);

    NewScanFile {
        path: path_str,
        name,
        extension,
        size_bytes: size_bytes as i64,
        created_at: system_time_to_rfc3339(metadata.created().ok()),
        modified_at: system_time_to_rfc3339(metadata.modified().ok()),
        accessed_at: system_time_to_rfc3339(metadata.accessed().ok()),
        mime_or_kind,
        is_system_hint: false,
    }
}

fn system_time_to_rfc3339(time: Option<SystemTime>) -> Option<String> {
    time.map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
}
