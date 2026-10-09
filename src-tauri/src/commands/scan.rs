use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use dark_pc_core::db::scan_repo;
use dark_pc_core::scanner::{run_scan, ScanOptions, ScanProgress};
use tauri::{AppHandle, Emitter, State};

use crate::state::{AppState, ScanRuntime, ScanSnapshot};

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanProgressEvent {
    scan_id: i64,
    files_seen: u64,
    bytes_total: u64,
    current_path: Option<String>,
    status: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanFinishedEvent {
    scan_id: i64,
    status: String,
    files_seen: u64,
    bytes_total: u64,
    error_message: Option<String>,
}

#[tauri::command]
pub fn start_scan(app: AppHandle, state: State<'_, AppState>, root_path: String) -> Result<ScanSnapshot, String> {
    spawn_scan(app, state.inner().clone(), root_path)?;
    get_scan_status(state)
}

pub fn spawn_scan(app: AppHandle, state: AppState, root_path: String) -> Result<(), String> {
    if !Path::new(&root_path).is_dir() {
        return Err(format!("Dossier introuvable : {root_path}"));
    }

    {
        let active = state.active_scan.lock().map_err(|e| e.to_string())?;
        if active.is_some() {
            return Err("Une analyse est déjà en cours.".into());
        }
    }

    let scan_id = {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        scan_repo::create_scan(db.conn_mut(), &root_path).map_err(|e| e.to_string())?
    };

    let runtime = ScanRuntime {
        scan_id,
        root_path: root_path.clone(),
        cancel: Arc::new(AtomicBool::new(false)),
    };
    let cancel_flag = runtime.cancel.clone();

    {
        let mut active = state.active_scan.lock().map_err(|e| e.to_string())?;
        *active = Some(runtime);
    }

    set_snapshot(
        &state,
        ScanSnapshot {
            scan_id: Some(scan_id),
            root_path: Some(root_path.clone()),
            status: "running".into(),
            files_seen: 0,
            bytes_total: 0,
            current_path: None,
            error_message: None,
        },
    )?;

    thread::spawn(move || {
        run_scan_worker(app, state, scan_id, root_path, cancel_flag);
    });

    Ok(())
}

fn run_scan_worker(
    app: AppHandle,
    state: AppState,
    scan_id: i64,
    root_path: String,
    cancel: Arc<AtomicBool>,
) {
    let run_result = {
        let mut db = match state.db.lock() {
            Ok(db) => db,
            Err(_) => return,
        };
        let options = match ScanOptions::from_connection(db.conn(), root_path.clone().into()) {
            Ok(o) => o,
            Err(err) => {
                let message = err.to_string();
                let _ = scan_repo::finish_scan(db.conn_mut(), scan_id, "failed", Some(&message));
                return;
            }
        };
        let mut last_emit = Instant::now();
        run_scan(
            &mut db,
            scan_id,
            options,
            &cancel,
            |progress: ScanProgress| {
                let now = Instant::now();
                if now.duration_since(last_emit) < Duration::from_millis(300)
                    && progress.files_seen % 800 != 0
                {
                    return;
                }
                last_emit = now;
                let _ = emit_progress_throttled(&app, &state, &progress, "running");
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
        Err(err) => {
            let message = err.to_string();
            if let Ok(mut db) = state.db.lock() {
                let _ = scan_repo::finish_scan(db.conn_mut(), scan_id, "failed", Some(&message));
            }
            ("failed".to_string(), Some(message))
        }
    };

    let snapshot = state.scan_snapshot.lock().map(|s| s.clone()).unwrap_or_else(|_| ScanSnapshot::idle());
    let _ = app.emit(
        "scan-finished",
        ScanFinishedEvent {
            scan_id,
            status: status.clone(),
            files_seen: snapshot.files_seen,
            bytes_total: snapshot.bytes_total,
            error_message: error_message.clone(),
        },
    );

    crate::commands::notifications::try_notify_scan_finished(&app, &state, scan_id, &status);

    let final_status = status.clone();
    let _ = set_snapshot(
        &state,
        ScanSnapshot {
            scan_id: Some(scan_id),
            root_path: Some(snapshot.root_path.unwrap_or_default()),
            status,
            files_seen: snapshot.files_seen,
            bytes_total: snapshot.bytes_total,
            current_path: None,
            error_message,
        },
    );

    if let Ok(mut active) = state.active_scan.lock() {
        *active = None;
    }

    crate::bootstrap::on_scan_worker_finished(&app, &state, scan_id, &final_status);
}

fn emit_progress_throttled(
    app: &AppHandle,
    state: &AppState,
    progress: &ScanProgress,
    status: &str,
) -> Result<(), String> {
    let root_path = state
        .scan_snapshot
        .lock()
        .ok()
        .and_then(|s| s.root_path.clone());

    set_snapshot(
        state,
        ScanSnapshot {
            scan_id: Some(progress.scan_id),
            root_path,
            status: status.into(),
            files_seen: progress.files_seen,
            bytes_total: progress.bytes_total,
            current_path: progress.current_path.clone(),
            error_message: None,
        },
    )?;

    let _ = app.emit(
        "scan-progress",
        ScanProgressEvent {
            scan_id: progress.scan_id,
            files_seen: progress.files_seen,
            bytes_total: progress.bytes_total,
            current_path: progress.current_path.clone(),
            status: status.into(),
        },
    );
    Ok(())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) -> Result<ScanSnapshot, String> {
    let runtime = state
        .active_scan
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    let Some(runtime) = runtime else {
        return get_scan_status(state);
    };

    runtime.cancel.store(true, Ordering::Relaxed);

    let next = {
        let current = state.scan_snapshot.lock().map_err(|e| e.to_string())?;
        ScanSnapshot {
            status: "cancelling".into(),
            ..current.clone()
        }
    };
    set_snapshot(&state, next)?;

    get_scan_status(state)
}

#[tauri::command]
pub fn get_scan_status(state: State<'_, AppState>) -> Result<ScanSnapshot, String> {
    state
        .scan_snapshot
        .lock()
        .map(|s| s.clone())
        .map_err(|e| e.to_string())
}

fn set_snapshot(state: &AppState, snapshot: ScanSnapshot) -> Result<(), String> {
    let mut current = state.scan_snapshot.lock().map_err(|e| e.to_string())?;
    *current = snapshot;
    Ok(())
}
