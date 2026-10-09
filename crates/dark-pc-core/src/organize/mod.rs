//! V2 — Propositions de classement dans l'arborescence du scan.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::operations_repo;
use crate::db::scan_repo;
use crate::large_files::ScanSummaryDto;
use crate::exclusions::{self, path_is_skipped};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedMove {
    pub source_path: String,
    pub dest_path: String,
    pub file_name: String,
    pub category: String,
    pub category_label: String,
    pub size_bytes: u64,
    pub conflict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeProposalsResult {
    pub scan: Option<ScanSummaryDto>,
    pub root_path: String,
    pub move_count: u64,
    pub total_bytes: u64,
    pub moves: Vec<ProposedMove>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveItemOutcome {
    pub source_path: String,
    pub dest_path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOrganizeResult {
    pub operation_id: i64,
    pub outcomes: Vec<MoveItemOutcome>,
    pub success_count: u32,
    pub failed_count: u32,
}

pub fn get_proposals(conn: &Connection, scan_id: Option<i64>) -> Result<OrganizeProposalsResult> {
    let scan = match scan_id {
        Some(id) => scan_repo::completed_scan_summary(conn, id)?,
        None => scan_repo::latest_completed_scan(conn)?,
    };

    let Some(scan) = scan else {
        return Ok(OrganizeProposalsResult {
            scan: None,
            root_path: String::new(),
            move_count: 0,
            total_bytes: 0,
            moves: Vec::new(),
        });
    };

    let root = PathBuf::from(&scan.root_path);
    let mut stmt = conn.prepare(
        "SELECT path, name, extension, size_bytes FROM files WHERE scan_id = ?1 ORDER BY size_bytes DESC",
    )?;
    let rows = stmt
        .query_map([scan.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let effective = exclusions::effective_from_connection(conn)?;
    let mut moves = Vec::new();
    let mut total_bytes = 0u64;

    for (path, name, extension, size) in rows {
        let file_path = PathBuf::from(&path);
        if path_is_skipped(&file_path, &effective) {
            continue;
        }
        let Some(parent) = file_path.parent() else {
            continue;
        };
        if parent != root.as_path() {
            continue;
        }
        let ext = extension
            .as_deref()
            .or_else(|| file_path.extension().and_then(|e| e.to_str()))
            .unwrap_or("");
        let (category, label) = category_for_extension(ext);
        let dest = root.join(category).join(&name);
        let dest_str = dest.to_string_lossy().into_owned();
        let conflict = dest.exists();
        total_bytes = total_bytes.saturating_add(size as u64);
        moves.push(ProposedMove {
            source_path: path,
            dest_path: dest_str,
            file_name: name,
            category: category.to_string(),
            category_label: label.to_string(),
            size_bytes: size as u64,
            conflict,
        });
    }

    let move_count = moves.len() as u64;
    Ok(OrganizeProposalsResult {
        scan: Some(scan.into()),
        root_path: root.to_string_lossy().into_owned(),
        move_count,
        total_bytes,
        moves,
    })
}

pub fn apply_moves(conn: &mut Connection, pairs: Vec<(String, String)>) -> Result<ApplyOrganizeResult> {
    if pairs.is_empty() {
        bail!("aucun déplacement sélectionné");
    }

    let effective = exclusions::effective_from_connection(conn)?;
    let mut outcomes = Vec::with_capacity(pairs.len());
    let mut success_count = 0u32;

    for (source, dest) in &pairs {
        match perform_move(&source, dest, &effective) {
            Ok(()) => {
                update_file_path(conn, source, dest)?;
                success_count += 1;
                outcomes.push(MoveItemOutcome {
                    source_path: source.clone(),
                    dest_path: dest.clone(),
                    success: true,
                    error: None,
                });
            }
            Err(err) => outcomes.push(MoveItemOutcome {
                source_path: source.clone(),
                dest_path: dest.clone(),
                success: false,
                error: Some(err.to_string()),
            }),
        }
    }

    let failed_count = outcomes.len() as u32 - success_count;
    let payload = serde_json::to_string(&outcomes).context("sérialisation payload organize")?;
    let op_id = operations_repo::insert_operation(
        conn,
        "organize",
        &payload,
        None,
        true,
    )?;

    Ok(ApplyOrganizeResult {
        operation_id: op_id,
        outcomes,
        success_count,
        failed_count,
    })
}

fn perform_move(source: &str, dest: &str, effective: &exclusions::EffectiveExclusions) -> Result<()> {
    let source_path = PathBuf::from(source);
    let dest_path = PathBuf::from(dest);
    if !source_path.is_absolute() || !dest_path.is_absolute() {
        bail!("chemins absolus requis");
    }
    if path_is_skipped(&source_path, effective) || path_is_skipped(&dest_path, effective) {
        bail!("chemin dans une zone exclue");
    }
    if !source_path.is_file() {
        bail!("source introuvable ou n'est pas un fichier");
    }
    if dest_path.exists() {
        bail!("destination déjà existante");
    }
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("création dossier {}", parent.display()))?;
    }
    std::fs::rename(&source_path, &dest_path)
        .with_context(|| format!("déplacement {} → {}", source, dest))?;
    Ok(())
}

fn update_file_path(conn: &Connection, old_path: &str, new_path: &str) -> Result<()> {
    let name = Path::new(new_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| new_path.to_string());
    conn.execute(
        "UPDATE files SET path = ?1, name = ?2 WHERE path = ?3",
        (new_path, name, old_path),
    )?;
    Ok(())
}

pub fn category_for_extension(ext: &str) -> (&'static str, &'static str) {
    let e = ext.trim_start_matches('.').to_ascii_lowercase();
    match e.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" | "heic" => {
            ("Images", "Images")
        }
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" => ("Videos", "Vidéos"),
        "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" => ("Musique", "Musique"),
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "rtf" => {
            ("Documents", "Documents")
        }
        "zip" | "rar" | "7z" | "tar" | "gz" => ("Archives", "Archives"),
        "exe" | "msi" | "msix" => ("Installateurs", "Installateurs"),
        _ => ("Autres", "Autres"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::scan_repo::{self, NewScanFile};

    #[test]
    fn proposes_flat_files_in_scan_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Downloads");
        std::fs::create_dir_all(&root).unwrap();
        let root_str = root.to_string_lossy().into_owned();

        let mut db = crate::db::Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn_mut();
        let scan_id = scan_repo::create_scan(conn, &root_str).unwrap();
        scan_repo::finish_scan(conn, scan_id, "completed", None).unwrap();

        let tx = conn.transaction().unwrap();
        scan_repo::insert_files_batch(
            &tx,
            scan_id,
            &[NewScanFile {
                path: format!("{}\\photo.jpg", root_str),
                name: "photo.jpg".into(),
                extension: Some("jpg".into()),
                size_bytes: 1024,
                created_at: None,
                modified_at: None,
                accessed_at: None,
                mime_or_kind: None,
                is_system_hint: false,
            }],
        )
        .unwrap();
        tx.commit().unwrap();

        let proposals = get_proposals(conn, Some(scan_id)).unwrap();
        assert_eq!(proposals.move_count, 1);
        assert!(proposals.moves[0].dest_path.contains("Images"));
    }
}
