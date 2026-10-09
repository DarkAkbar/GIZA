use dark_pc_core::organize::{self, ApplyOrganizeResult, OrganizeProposalsResult};
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeMoveInput {
    source_path: String,
    dest_path: String,
}

#[tauri::command]
pub fn get_organize_proposals(
    state: State<'_, AppState>,
    scan_id: Option<i64>,
) -> Result<OrganizeProposalsResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    organize::get_proposals(db.conn(), scan_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_organize_moves(
    state: State<'_, AppState>,
    moves: Vec<OrganizeMoveInput>,
) -> Result<ApplyOrganizeResult, String> {
    let pairs = moves
        .into_iter()
        .map(|m| (m.source_path, m.dest_path))
        .collect();
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    organize::apply_moves(db.conn_mut(), pairs).map_err(|e| e.to_string())
}
