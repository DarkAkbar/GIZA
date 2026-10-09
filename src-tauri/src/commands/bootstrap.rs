use tauri::{AppHandle, State};

use crate::bootstrap::{self, BootstrapStatusDto};
use crate::state::AppState;

#[tauri::command]
pub fn ensure_bootstrap_analysis(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<BootstrapStatusDto, String> {
    bootstrap::ensure_bootstrap(&app, state.inner())
}

#[tauri::command]
pub fn get_bootstrap_status(state: State<'_, AppState>) -> Result<BootstrapStatusDto, String> {
    let queue_len = state
        .bootstrap_queue
        .lock()
        .map_err(|e| e.to_string())?
        .len() as u32;
    if state.active_scan.lock().map_err(|e| e.to_string())?.is_some() {
        return Ok(BootstrapStatusDto {
            phase: "scanning".into(),
            volumes_remaining: queue_len.max(1),
            current_root: state
                .scan_snapshot
                .lock()
                .ok()
                .and_then(|s| s.root_path.clone()),
            message: "Indexation en cours…".into(),
        });
    }
    if state
        .active_duplicate
        .lock()
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(BootstrapStatusDto {
            phase: "duplicates".into(),
            volumes_remaining: queue_len.max(1),
            current_root: None,
            message: "Recherche des doublons…".into(),
        });
    }
    Ok(BootstrapStatusDto {
        phase: if queue_len == 0 { "done" } else { "idle" }.into(),
        volumes_remaining: queue_len,
        current_root: None,
        message: if queue_len == 0 {
            "Analyse initiale à jour.".into()
        } else {
            "En attente du prochain volume.".into()
        },
    })
}
