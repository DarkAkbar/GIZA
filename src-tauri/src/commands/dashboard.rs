use dark_pc_core::dashboard;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_storage_insights(
    state: State<'_, AppState>,
) -> Result<dashboard::StorageInsights, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    dashboard::get_storage_insights(db.conn()).map_err(|e| e.to_string())
}
