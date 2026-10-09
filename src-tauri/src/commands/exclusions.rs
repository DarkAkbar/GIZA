use dark_pc_core::db::exclusions_repo::UserExclusionRow;
use dark_pc_core::exclusions::{self, ExclusionsOverview};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_exclusions(state: State<'_, AppState>) -> Result<ExclusionsOverview, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    exclusions::get_overview(db.conn()).map_err(map_exclusion_error)
}

#[tauri::command]
pub fn add_user_exclusion(
    state: State<'_, AppState>,
    path: String,
) -> Result<UserExclusionRow, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    exclusions::add_user_exclusion(db.conn(), &path).map_err(map_exclusion_error)
}

#[tauri::command]
pub fn remove_user_exclusion(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    exclusions::remove_user_exclusion(db.conn(), id).map_err(map_exclusion_error)
}

#[tauri::command]
pub fn reset_user_exclusions(state: State<'_, AppState>) -> Result<u64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    exclusions::reset_user_exclusions(db.conn()).map_err(map_exclusion_error)
}

fn map_exclusion_error(err: impl ToString) -> String {
    err.to_string()
}
