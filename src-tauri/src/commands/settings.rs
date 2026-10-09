use dark_pc_core::settings::{self, SettingsResponse, UpdateSettings};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    settings::get_response(db.conn()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    patch: UpdateSettings,
) -> Result<SettingsResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    settings::update(db.conn(), patch).map_err(|e| e.to_string())?;
    settings::get_response(db.conn()).map_err(|e| e.to_string())
}
