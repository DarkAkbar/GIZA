use dark_pc_core::large_files::{self, LargeFileQuery};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn list_large_files(
    state: State<'_, AppState>,
    min_size_bytes: u64,
    scan_id: Option<i64>,
    limit: Option<u32>,
) -> Result<large_files::LargeFilesResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    large_files::list_large_files(
        db.conn(),
        LargeFileQuery {
            min_size_bytes,
            scan_id,
            limit: limit.unwrap_or(large_files::DEFAULT_LIMIT),
        },
    )
    .map_err(|e| e.to_string())
}
