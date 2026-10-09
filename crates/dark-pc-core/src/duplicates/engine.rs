use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::db::duplicate_repo;
use crate::db::scan_repo;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DuplicateProgress {
    pub scan_id: i64,
    pub phase: String,
    pub candidates_hashed: u64,
    pub groups_found: u64,
    pub recoverable_bytes: u64,
}

pub fn compute_duplicates(
    conn: &mut Connection,
    scan_id: i64,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(DuplicateProgress),
) -> Result<()> {
    duplicate_repo::clear_groups_for_scan(conn, scan_id)?;

    let sizes = duplicate_repo::sizes_with_multiple_files(conn, scan_id)?;
    let mut candidates_hashed = 0u64;
    let mut groups_found = 0u64;
    let mut recoverable_total = 0u64;

    for size_bytes in sizes {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }

        let files = duplicate_repo::files_with_size(conn, scan_id, size_bytes)?;
        let mut by_hash: HashMap<String, Vec<(i64, u64)>> = HashMap::new();

        for (file_id, path) in files {
            if cancel.load(Ordering::Relaxed) {
                return Ok(());
            }

            let hash = match hash_file(Path::new(&path)) {
                Ok(h) => h,
                Err(err) => {
                    tracing::warn!(path = %path, error = %err, "hash impossible");
                    continue;
                }
            };
            duplicate_repo::update_file_hash(conn, file_id, &hash)?;
            candidates_hashed += 1;
            by_hash
                .entry(hash)
                .or_default()
                .push((file_id, size_bytes as u64));

            if candidates_hashed % 250 == 0 {
                on_progress(DuplicateProgress {
                    scan_id,
                    phase: "hashing".into(),
                    candidates_hashed,
                    groups_found,
                    recoverable_bytes: recoverable_total,
                });
            }
        }

        for (hash, members) in by_hash {
            if members.len() < 2 {
                continue;
            }
            let file_count = members.len() as i64;
            let unit_size = members[0].1;
            let total_bytes = unit_size.saturating_mul(members.len() as u64) as i64;
            let recoverable_bytes = unit_size.saturating_mul(members.len() as u64 - 1) as i64;

            let group_id = duplicate_repo::insert_group(
                conn,
                scan_id,
                &hash,
                file_count,
                total_bytes,
                recoverable_bytes,
            )?;
            for (file_id, _) in &members {
                duplicate_repo::insert_member(conn, group_id, *file_id)?;
            }

            groups_found += 1;
            recoverable_total = recoverable_total.saturating_add(recoverable_bytes as u64);
        }
    }

    on_progress(DuplicateProgress {
        scan_id,
        phase: "done".into(),
        candidates_hashed,
        groups_found,
        recoverable_bytes: recoverable_total,
    });
    Ok(())
}

pub fn resolve_scan_id(conn: &Connection, scan_id: Option<i64>) -> Result<Option<i64>> {
    if let Some(id) = scan_id {
        if duplicate_repo::completed_scan_by_id(conn, id)? {
            return Ok(Some(id));
        }
        return Ok(None);
    }
    Ok(scan_repo::latest_completed_scan(conn)?.map(|s| s.id))
}

fn hash_file(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| format!("ouverture {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65_536];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
