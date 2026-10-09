use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use dark_pc_core::duplicates::{
    self, compute_duplicates, resolve_scan_id, DuplicateGroupDetails, DuplicateGroupsResult,
    DuplicateProgress,
};
use tauri::{AppHandle, Emitter, State};

use crate::state::{AppState, DuplicateRuntime, DuplicateSnapshot};

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DuplicateProgressEvent {
    scan_id: i64,
    phase: String,
    candidates_hashed: u64,
    groups_found: u64,
    recoverable_bytes: u64,
    status: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DuplicateFinishedEvent {
    scan_id: i64,
    status: String,
    groups_found: u64,
    recoverable_bytes: u64,
    error_message: Option<String>,
}

#[tauri::command]
pub fn start_duplicate_analysis(
    app: AppHandle,
    state: State<'_, AppState>,
    scan_id: Option<i64>,
) -> Result<DuplicateSnapshot, String> {
    spawn_duplicate_analysis(app, state.inner().clone(), scan_id)?;
    get_duplicate_status(state)
}

pub fn spawn_duplicate_analysis(
    app: AppHandle,
    state: AppState,
    scan_id: Option<i64>,
) -> Result<(), String> {
    {
        let active = state.active_duplicate.lock().map_err(|e| e.to_string())?;
        if active.is_some() {
            return Err("Une analyse de doublons est déjà en cours.".into());
        }
    }

    let resolved_scan_id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        resolve_scan_id(db.conn(), scan_id).map_err(|e| e.to_string())?
    };
    let Some(resolved_scan_id) = resolved_scan_id else {
        return Err("Aucun scan terminé disponible.".into());
    };

    let runtime = DuplicateRuntime {
        scan_id: resolved_scan_id,
        cancel: Arc::new(AtomicBool::new(false)),
    };
    let cancel_flag = runtime.cancel.clone();

    {
        let mut active = state.active_duplicate.lock().map_err(|e| e.to_string())?;
        *active = Some(runtime);
    }

    set_duplicate_snapshot(
        &state,
        DuplicateSnapshot {
            scan_id: Some(resolved_scan_id),
            status: "running".into(),
            candidates_hashed: 0,
            groups_found: 0,
            recoverable_bytes: 0,
            error_message: None,
        },
    )?;

    thread::spawn(move || {
        run_duplicate_worker(app, state, resolved_scan_id, cancel_flag);
    });

    Ok(())
}

fn run_duplicate_worker(
    app: AppHandle,
    state: AppState,
    scan_id: i64,
    cancel: Arc<AtomicBool>,
) {
    let run_result = {
        let mut db = match state.db.lock() {
            Ok(db) => db,
            Err(_) => return,
        };
        compute_duplicates(
            db.conn_mut(),
            scan_id,
            &cancel,
            |progress: DuplicateProgress| {
                let _ = emit_duplicate_progress(&app, &state, &progress);
            },
        )
    };

    let (status, error_message) = match run_result {
        Ok(()) => {
            if cancel.load(Ordering::Relaxed) {
                ("cancelled".to_string(), None)
            } else {
                ("completed".to_string(), None)
            }
        }
        Err(err) => ("failed".to_string(), Some(err.to_string())),
    };

    let snapshot = state
        .duplicate_snapshot
        .lock()
        .map(|s| s.clone())
        .unwrap_or_else(|_| DuplicateSnapshot::idle());

    let _ = app.emit(
        "duplicate-finished",
        DuplicateFinishedEvent {
            scan_id,
            status: status.clone(),
            groups_found: snapshot.groups_found,
            recoverable_bytes: snapshot.recoverable_bytes,
            error_message: error_message.clone(),
        },
    );

    let final_status = status.clone();
    let _ = set_duplicate_snapshot(
        &state,
        DuplicateSnapshot {
            scan_id: Some(scan_id),
            status,
            candidates_hashed: snapshot.candidates_hashed,
            groups_found: snapshot.groups_found,
            recoverable_bytes: snapshot.recoverable_bytes,
            error_message,
        },
    );

    if let Ok(mut active) = state.active_duplicate.lock() {
        *active = None;
    }

    crate::bootstrap::on_duplicate_worker_finished(&app, &state, &final_status);
}

fn emit_duplicate_progress(
    app: &AppHandle,
    state: &AppState,
    progress: &DuplicateProgress,
) -> Result<(), String> {
    set_duplicate_snapshot(
        state,
        DuplicateSnapshot {
            scan_id: Some(progress.scan_id),
            status: "running".into(),
            candidates_hashed: progress.candidates_hashed,
            groups_found: progress.groups_found,
            recoverable_bytes: progress.recoverable_bytes,
            error_message: None,
        },
    )?;
    let _ = app.emit(
        "duplicate-progress",
        DuplicateProgressEvent {
            scan_id: progress.scan_id,
            phase: progress.phase.clone(),
            candidates_hashed: progress.candidates_hashed,
            groups_found: progress.groups_found,
            recoverable_bytes: progress.recoverable_bytes,
            status: "running".into(),
        },
    );
    Ok(())
}

#[tauri::command]
pub fn cancel_duplicate_analysis(state: State<'_, AppState>) -> Result<DuplicateSnapshot, String> {
    let runtime = state
        .active_duplicate
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let Some(runtime) = runtime else {
        return get_duplicate_status(state);
    };
    runtime.cancel.store(true, Ordering::Relaxed);
    let next = {
        let current = state.duplicate_snapshot.lock().map_err(|e| e.to_string())?;
        DuplicateSnapshot {
            status: "cancelling".into(),
            ..current.clone()
        }
    };
    set_duplicate_snapshot(&state, next)?;
    get_duplicate_status(state)
}

#[tauri::command]
pub fn get_duplicate_status(state: State<'_, AppState>) -> Result<DuplicateSnapshot, String> {
    state
        .duplicate_snapshot
        .lock()
        .map(|s| s.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_duplicate_groups(
    state: State<'_, AppState>,
    scan_id: Option<i64>,
    limit: Option<u32>,
) -> Result<DuplicateGroupsResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    duplicates::list_duplicate_groups(db.conn(), scan_id, limit.unwrap_or(200))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_duplicate_group_details(
    state: State<'_, AppState>,
    group_id: i64,
) -> Result<Option<DuplicateGroupDetails>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    duplicates::get_duplicate_group_details(db.conn(), group_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_duplicate_merge_plan(
    state: State<'_, AppState>,
    scan_id: Option<i64>,
) -> Result<Option<duplicates::DuplicateMergePlan>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    duplicates::duplicate_merge_plan(db.conn(), scan_id).map_err(|e| e.to_string())
}

fn set_duplicate_snapshot(state: &AppState, snapshot: DuplicateSnapshot) -> Result<(), String> {
    let mut current = state.duplicate_snapshot.lock().map_err(|e| e.to_string())?;
    *current = snapshot;
    Ok(())
}
