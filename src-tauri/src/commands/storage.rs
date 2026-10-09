use dark_pc_core::storage_monitor::{self, RecordSnapshotsResult, StorageHistoryQuery, StorageHistoryResult};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn record_storage_snapshots(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<RecordSnapshotsResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    storage_monitor::record_snapshots(db.conn(), force.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_storage_history(
    state: State<'_, AppState>,
    mount_path: Option<String>,
    period_days: Option<u32>,
    limit: Option<u32>,
) -> Result<StorageHistoryResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    storage_monitor::get_storage_history(
        db.conn(),
        StorageHistoryQuery {
            mount_path,
            period_days,
            limit,
        },
    )
    .map_err(|e| e.to_string())
}
