use dark_pc_core::cleanup::{self, CleanupAnalysisResult};

#[tauri::command]
pub fn analyze_temp_files(max_files: Option<u32>) -> Result<CleanupAnalysisResult, String> {
    cleanup::analyze_temp_files(max_files.unwrap_or(cleanup::DEFAULT_MAX_FILES))
        .map_err(|e| e.to_string())
}
