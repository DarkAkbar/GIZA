use dark_pc_core::analysis_schedule;
use tauri::{AppHandle, State};

use crate::bootstrap::{self, BootstrapStatusDto};
use crate::state::AppState;

#[tauri::command]
pub fn get_analysis_schedule_status(
    state: State<'_, AppState>,
) -> Result<analysis_schedule::AnalysisScheduleStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    analysis_schedule::get_status(db.conn()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn request_volume_rescan(
    app: AppHandle,
    state: State<'_, AppState>,
    force_all: Option<bool>,
) -> Result<BootstrapStatusDto, String> {
    bootstrap::request_rescan_volumes(&app, state.inner(), force_all.unwrap_or(false))
}
