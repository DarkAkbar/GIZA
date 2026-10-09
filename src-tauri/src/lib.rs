mod bootstrap;
mod commands;
mod db_path;
mod dto;
mod state;

use std::sync::{Arc, Mutex};

use dark_pc_core::Database;
use state::{AppState, DuplicateSnapshot, ScanSnapshot};
use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let db_path = db_path::resolve_database_path(&app.handle())?;

            let db = Database::open(&db_path)?;
            tracing::info!(path = %db_path.display(), "base SQLite initialisée");

            match dark_pc_core::storage_monitor::record_snapshots(db.conn(), false) {
                Ok(result) => tracing::info!(
                    recorded = result.recorded_count,
                    skipped = result.skipped_cooldown_count,
                    "snapshots stockage au démarrage"
                ),
                Err(err) => tracing::warn!(error = %err, "snapshots stockage au démarrage ignorés"),
            }

            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                db_path,
                active_scan: Arc::new(Mutex::new(None)),
                scan_snapshot: Arc::new(Mutex::new(ScanSnapshot::idle())),
                active_duplicate: Arc::new(Mutex::new(None)),
                duplicate_snapshot: Arc::new(Mutex::new(DuplicateSnapshot::idle())),
                bootstrap_queue: Arc::new(Mutex::new(Vec::new())),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_info,
            commands::apps::list_installed_apps,
            commands::apps::uninstall_application,
            commands::cleanup::analyze_temp_files,
            commands::dashboard::get_storage_insights,
            commands::recommendations::get_recommendations,
            commands::disk::get_volume_stats,
            commands::disk::list_mounted_volumes,
            commands::disk::get_scan_root_suggestions,
            commands::bootstrap::ensure_bootstrap_analysis,
            commands::bootstrap::get_bootstrap_status,
            commands::analysis_schedule::get_analysis_schedule_status,
            commands::analysis_schedule::request_volume_rescan,
            commands::scan::start_scan,
            commands::scan::cancel_scan,
            commands::scan::get_scan_status,
            commands::large_files::list_large_files,
            commands::organize::get_organize_proposals,
            commands::organize::apply_organize_moves,
            commands::duplicates::start_duplicate_analysis,
            commands::duplicates::cancel_duplicate_analysis,
            commands::duplicates::get_duplicate_status,
            commands::duplicates::list_duplicate_groups,
            commands::duplicates::get_duplicate_group_details,
            commands::duplicates::get_duplicate_merge_plan,
            commands::shell::show_item_in_folder,
            commands::safe_delete::move_files_to_trash,
            commands::safe_delete::list_operations,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::exclusions::get_exclusions,
            commands::exclusions::add_user_exclusion,
            commands::exclusions::remove_user_exclusion,
            commands::exclusions::reset_user_exclusions,
            commands::storage::record_storage_snapshots,
            commands::storage::get_storage_history,
            commands::notifications::get_notification_state,
            commands::notifications::process_notifications,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au démarrage Tauri");
}
