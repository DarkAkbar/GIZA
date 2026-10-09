use std::path::Path;

#[tauri::command]
pub fn show_item_in_folder(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Chemin vide.".into());
    }
    if !Path::new(trimmed).exists() {
        return Err("Fichier introuvable.".into());
    }

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{trimmed}"))
            .spawn()
            .map_err(|e| format!("Impossible d'ouvrir l'explorateur : {e}"))?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("Fonction disponible sur Windows uniquement.".into())
    }
}
