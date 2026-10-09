use dark_pc_core::apps;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn list_installed_apps(
    _state: State<'_, AppState>,
    limit: Option<u32>,
) -> apps::InstalledAppsResult {
    apps::list_installed_apps(limit.unwrap_or(24).min(48))
}

#[tauri::command]
pub fn uninstall_application(
    state: State<'_, AppState>,
    app_name: String,
    publisher: Option<String>,
    estimated_size_bytes: Option<u64>,
) -> Result<(), String> {
    apps::launch_uninstall(&app_name)?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    apps::log_uninstall_requested(
        db.conn(),
        &app_name,
        publisher.as_deref(),
        estimated_size_bytes,
    )?;
    Ok(())
}
