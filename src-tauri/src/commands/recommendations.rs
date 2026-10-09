use dark_pc_core::cleanup;

use dark_pc_core::recommendations;

use tauri::State;



use crate::state::AppState;



#[tauri::command]

pub fn get_recommendations(

    state: State<'_, AppState>,

    include_cleanup_scan: Option<bool>,

) -> Result<recommendations::RecommendationsResult, String> {

    let db = state.db.lock().map_err(|e| e.to_string())?;

    let cleanup = if include_cleanup_scan.unwrap_or(false) {

        Some(cleanup::analyze_temp_files(600).map_err(|e| e.to_string())?)

    } else {

        None

    };

    let cleanup_ref = cleanup.as_ref();

    recommendations::generate_recommendations_with_cleanup(db.conn(), cleanup_ref)

        .map_err(|e| e.to_string())

}


