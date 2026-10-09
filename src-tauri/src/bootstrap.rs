use dark_pc_core::analysis_schedule;
use dark_pc_core::db::{duplicate_repo, scan_repo};
use dark_pc_core::disk;
use dark_pc_core::settings;
use tauri::{AppHandle, Emitter};

use crate::commands::duplicates;
use crate::commands::scan;
use crate::state::AppState;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapStatusDto {
    pub phase: String,
    pub volumes_remaining: u32,
    pub current_root: Option<String>,
    pub message: String,
}

pub fn ensure_bootstrap(app: &AppHandle, state: &AppState) -> Result<BootstrapStatusDto, String> {
    if state.active_scan.lock().map_err(|e| e.to_string())?.is_some() {
        return Ok(status_from_snapshot(state, "scanning", "Indexation des fichiers en cours…"));
    }
    if state.active_duplicate.lock().map_err(|e| e.to_string())?.is_some() {
        return Ok(status_from_snapshot(
            state,
            "duplicates",
            "Recherche des doublons sur le volume analysé…",
        ));
    }

    {
        let mut queue = state.bootstrap_queue.lock().map_err(|e| e.to_string())?;
        if queue.is_empty() {
            refill_bootstrap_queue(state, &mut queue)?;
        }
    }

    let queue_len = state
        .bootstrap_queue
        .lock()
        .map_err(|e| e.to_string())?
        .len();

    if queue_len == 0 {
        return Ok(BootstrapStatusDto {
            phase: "done".into(),
            volumes_remaining: 0,
            current_root: None,
            message: "Analyse initiale terminée. Consultez les solutions proposées.".into(),
        });
    }

    let next_root = {
        let queue = state.bootstrap_queue.lock().map_err(|e| e.to_string())?;
        queue.first().cloned()
    };

    let Some(root) = next_root else {
        return Ok(BootstrapStatusDto {
            phase: "done".into(),
            volumes_remaining: 0,
            current_root: None,
            message: "Analyse initiale terminée.".into(),
        });
    };

    let root_label = root.trim_end_matches('\\').to_string();
    scan::spawn_scan(app.clone(), state.clone(), root)?;

    Ok(BootstrapStatusDto {
        phase: "scanning".into(),
        volumes_remaining: queue_len as u32,
        current_root: Some(root_label.clone()),
        message: format!(
            "Analyse automatique du volume {root_label} (une étape à la fois pour préserver la stabilité)."
        ),
    })
}

fn refill_bootstrap_queue(state: &AppState, queue: &mut Vec<String>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    for root in disk::bootstrap_volume_roots() {
        let skip = scan_repo::has_completed_scan_for_root(db.conn(), &root).map_err(|e| e.to_string())?;
        if !skip {
            queue.push(root);
        }
    }
    Ok(())
}

/// Ajoute à la file les volumes dont la dernière analyse est absente ou plus vieille que `older_than_days`.
pub fn enqueue_stale_volume_rescans(state: &AppState, older_than_days: u64) -> Result<u32, String> {
    let roots = disk::bootstrap_volume_roots();
    let mut stale_roots = Vec::new();
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for root in roots {
            let latest = scan_repo::latest_completed_scan_for_root(db.conn(), &root)
                .map_err(|e| e.to_string())?;
            let is_stale = match latest.and_then(|s| s.finished_at) {
                None => true,
                Some(at) => analysis_schedule::days_since_rfc3339(&at).unwrap_or(0) >= older_than_days,
            };
            if is_stale {
                stale_roots.push(root);
            }
        }
    }
    let mut queue = state.bootstrap_queue.lock().map_err(|e| e.to_string())?;
    let mut added = 0_u32;
    for root in stale_roots {
        if queue.iter().any(|r| paths_same_volume(r, &root)) {
            continue;
        }
        queue.push(root);
        added += 1;
    }
    Ok(added)
}

pub fn enqueue_all_volume_rescans(state: &AppState) -> Result<u32, String> {
    let roots = disk::bootstrap_volume_roots();
    let mut queue = state.bootstrap_queue.lock().map_err(|e| e.to_string())?;
    let mut added = 0_u32;
    for root in roots {
        if queue.iter().any(|r| paths_same_volume(r, &root)) {
            continue;
        }
        queue.push(root);
        added += 1;
    }
    Ok(added)
}

fn paths_same_volume(a: &str, b: &str) -> bool {
    let na = a.trim().trim_end_matches('\\').to_ascii_uppercase();
    let nb = b.trim().trim_end_matches('\\').to_ascii_uppercase();
    na == nb
}

pub fn request_rescan_volumes(app: &AppHandle, state: &AppState, force_all: bool) -> Result<BootstrapStatusDto, String> {
    if state.active_scan.lock().map_err(|e| e.to_string())?.is_some() {
        return Ok(status_from_snapshot(
            state,
            "scanning",
            "Une analyse est déjà en cours…",
        ));
    }
    if state.active_duplicate.lock().map_err(|e| e.to_string())?.is_some() {
        return Ok(status_from_snapshot(
            state,
            "duplicates",
            "Analyse des doublons en cours — relance disponible ensuite.",
        ));
    }

    let older_than_days = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        settings::load(db.conn())
            .map_err(|e| e.to_string())?
            .analysis_reminder_interval_days
    };

    if force_all {
        enqueue_all_volume_rescans(state)?;
    } else {
        let added = enqueue_stale_volume_rescans(state, older_than_days)?;
        if added == 0 {
            return Ok(BootstrapStatusDto {
                phase: "idle".into(),
                volumes_remaining: 0,
                current_root: None,
                message: "Analyses récentes sur tous les volumes — aucune relance nécessaire.".into(),
            });
        }
    }

    ensure_bootstrap(app, state)
}

pub fn on_scan_worker_finished(app: &AppHandle, state: &AppState, scan_id: i64, status: &str) {
    if status != "completed" {
        drop_front_queue(state);
        let _ = try_start_next_volume_scan(app, state);
        return;
    }

    let db = match state.db.lock() {
        Ok(db) => db,
        Err(_) => return,
    };
    let has_dup = duplicate_repo::has_groups_for_scan(db.conn(), scan_id).unwrap_or(false);
    drop(db);

    if has_dup {
        let _ = try_start_next_volume_scan(app, state);
    } else if duplicates::spawn_duplicate_analysis(app.clone(), state.clone(), Some(scan_id)).is_err() {
        let _ = try_start_next_volume_scan(app, state);
    }
}

pub fn on_duplicate_worker_finished(app: &AppHandle, state: &AppState, status: &str) {
    if status == "completed" || status == "failed" || status == "cancelled" {
        let _ = try_start_next_volume_scan(app, state);
    }
}

fn try_start_next_volume_scan(app: &AppHandle, state: &AppState) -> Result<(), String> {
    drop_front_queue(state);
    let status = ensure_bootstrap(app, state)?;
    if status.phase == "done" {
        let _ = app.emit("bootstrap-complete", ());
    }
    Ok(())
}

fn drop_front_queue(state: &AppState) {
    if let Ok(mut queue) = state.bootstrap_queue.lock() {
        if !queue.is_empty() {
            queue.remove(0);
        }
    }
}

fn status_from_snapshot(state: &AppState, phase: &str, message: &str) -> BootstrapStatusDto {
    let (current_root, remaining) = match state.bootstrap_queue.lock() {
        Ok(q) => (q.first().cloned(), q.len() as u32),
        Err(_) => (None, 0),
    };
    BootstrapStatusDto {
        phase: phase.into(),
        volumes_remaining: remaining.max(1),
        current_root,
        message: message.into(),
    }
}
