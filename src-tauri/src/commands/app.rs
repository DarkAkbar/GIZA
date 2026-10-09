use tauri::State;

use crate::dto::AppInfoDto;
use crate::state::AppState;

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfoDto, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let migration_version = db.migration_version().map_err(|e| e.to_string())?;

    Ok(AppInfoDto {
        name: "Giza".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        db_path: state.db_path.display().to_string(),
        migration_version,
        platform: std::env::consts::OS.into(),
    })
}
