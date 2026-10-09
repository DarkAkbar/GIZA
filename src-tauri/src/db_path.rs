use std::path::PathBuf;

use anyhow::{Context, Result};
use tauri::AppHandle;
use tauri::Manager;

const DB_FILE_NAME: &str = "giza.db";
const LEGACY_IDENTIFIER_DIR: &str = "com.darkpc.app";
const LEGACY_DB_FILE: &str = "dark-pc.db";

pub fn resolve_database_path(app: &AppHandle) -> Result<PathBuf> {
    let data_dir = app
        .path()
        .app_data_dir()
        .context("dossier AppData Giza")?;
    std::fs::create_dir_all(&data_dir).context("création AppData")?;

    let target = data_dir.join(DB_FILE_NAME);
    if target.exists() {
        return Ok(target);
    }

    let legacy_in_current = data_dir.join(LEGACY_DB_FILE);
    if legacy_in_current.exists() {
        std::fs::copy(&legacy_in_current, &target).context("migration dark-pc.db locale")?;
        return Ok(target);
    }

    if let Some(parent) = data_dir.parent() {
        let legacy_dir = parent.join(LEGACY_IDENTIFIER_DIR);
        let legacy_db = legacy_dir.join(LEGACY_DB_FILE);
        if legacy_db.exists() {
            std::fs::copy(&legacy_db, &target).context("migration base Dark PC")?;
            tracing::info!(
                from = %legacy_db.display(),
                to = %target.display(),
                "base SQLite migrée depuis com.darkpc.app"
            );
            return Ok(target);
        }
    }

    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_paths_constants() {
        assert_eq!(LEGACY_IDENTIFIER_DIR, "com.darkpc.app");
        assert!(Path::new(DB_FILE_NAME).extension().is_none());
    }
}
