use dark_pc_core::db::operations_repo;
use dark_pc_core::safe_delete;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn move_files_to_trash(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<safe_delete::TrashResult, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    safe_delete::trash_files(db.conn_mut(), paths).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_operations(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<operations_repo::OperationRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    operations_repo::list_operations(db.conn(), limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}
