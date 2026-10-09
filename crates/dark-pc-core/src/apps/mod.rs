//! Applications installées (lecture registre Windows — sans désinstallation).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub name: String,
    pub publisher: Option<String>,
    pub estimated_size_bytes: Option<u64>,
    pub icon_data_url: Option<String>,
    pub can_uninstall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAppsResult {
    pub apps: Vec<InstalledApp>,
    pub platform_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallLogPayload {
    pub app_name: String,
    pub publisher: Option<String>,
    pub estimated_size_bytes: Option<u64>,
}

pub fn log_uninstall_requested(
    conn: &rusqlite::Connection,
    app_name: &str,
    publisher: Option<&str>,
    estimated_size_bytes: Option<u64>,
) -> Result<i64, String> {
    use crate::db::operations_repo;
    let payload = UninstallLogPayload {
        app_name: app_name.to_string(),
        publisher: publisher.map(str::to_string),
        estimated_size_bytes,
    };
    let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    operations_repo::insert_operation(
        conn,
        "uninstall",
        &json,
        estimated_size_bytes.map(|s| s as i64),
        false,
    )
    .map_err(|e| e.to_string())
}

pub fn launch_uninstall(display_name: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        return windows_impl::launch_uninstall(display_name);
    }
    #[cfg(not(windows))]
    {
        let _ = display_name;
        Err("Désinstallation disponible sur Windows uniquement.".into())
    }
}

pub fn list_installed_apps(limit: u32) -> InstalledAppsResult {
    #[cfg(windows)]
    {
        return windows_impl::list_installed_apps(limit);
    }
    #[cfg(not(windows))]
    {
        let _ = limit;
        InstalledAppsResult {
            apps: Vec::new(),
            platform_supported: false,
        }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::{InstalledApp, InstalledAppsResult};
    use std::collections::HashMap;
    use std::path::Path;
    use winreg::enums::*;
    use winreg::RegKey;

    const MAX_ICON_BYTES: usize = 96_000;

    pub fn list_installed_apps(limit: u32) -> InstalledAppsResult {
        let mut by_name: HashMap<String, InstalledApp> = HashMap::new();

        let roots = [
            (
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_CURRENT_USER,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
        ];

        for (hive, subkey) in roots {
            let root = RegKey::predef(hive);
            let Ok(uninstall) = root.open_subkey(subkey) else {
                continue;
            };
            for sub_name in uninstall.enum_keys().flatten() {
                let Ok(app_key) = uninstall.open_subkey(&sub_name) else {
                    continue;
                };
                if is_system_component(&app_key) {
                    continue;
                }
                let Ok(name) = app_key.get_value::<String, _>("DisplayName") else {
                    continue;
                };
                let name = name.trim().to_string();
                if name.is_empty() {
                    continue;
                }

                let publisher: Option<String> = app_key.get_value("Publisher").ok();
                let size_kb: Option<u32> = app_key.get_value("EstimatedSize").ok();
                let estimated_size_bytes = size_kb.map(|kb| u64::from(kb) * 1024);

                let icon_data_url = app_key
                    .get_value::<String, _>("DisplayIcon")
                    .ok()
                    .and_then(|raw| icon_to_data_url(&raw));

                let can_uninstall = read_uninstall_string(&app_key).is_some();

                let key = name.to_ascii_lowercase();
                let entry = InstalledApp {
                    name,
                    publisher,
                    estimated_size_bytes,
                    icon_data_url,
                    can_uninstall,
                };

                by_name
                    .entry(key)
                    .and_modify(|existing| {
                        let new_size = entry.estimated_size_bytes.unwrap_or(0);
                        let old_size = existing.estimated_size_bytes.unwrap_or(0);
                        if new_size > old_size {
                            *existing = entry.clone();
                        } else if existing.icon_data_url.is_none() && entry.icon_data_url.is_some() {
                            existing.icon_data_url = entry.icon_data_url.clone();
                        }
                    })
                    .or_insert(entry);
            }
        }

        let mut apps: Vec<InstalledApp> = by_name.into_values().collect();
        apps.sort_by(|a, b| {
            b.estimated_size_bytes
                .unwrap_or(0)
                .cmp(&a.estimated_size_bytes.unwrap_or(0))
                .then_with(|| a.name.cmp(&b.name))
        });
        apps.truncate(limit as usize);

        InstalledAppsResult {
            apps,
            platform_supported: true,
        }
    }

    pub fn launch_uninstall(display_name: &str) -> Result<(), String> {
        let cmd = find_uninstall_command(display_name)
            .ok_or_else(|| format!("Programme introuvable ou sans désinstalleur : {display_name}"))?;
        if cmd.len() > 4096 {
            return Err("Commande de désinstallation invalide.".into());
        }
        std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .spawn()
            .map_err(|e| format!("Impossible de lancer la désinstallation : {e}"))?;
        Ok(())
    }

    fn find_uninstall_command(display_name: &str) -> Option<String> {
        let target = display_name.trim().to_ascii_lowercase();
        if target.is_empty() {
            return None;
        }
        let roots = [
            (
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_CURRENT_USER,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
        ];
        for (hive, subkey) in roots {
            let root = RegKey::predef(hive);
            let Ok(uninstall) = root.open_subkey(subkey) else {
                continue;
            };
            for sub_name in uninstall.enum_keys().flatten() {
                let Ok(app_key) = uninstall.open_subkey(&sub_name) else {
                    continue;
                };
                if is_system_component(&app_key) {
                    continue;
                }
                let Ok(name) = app_key.get_value::<String, _>("DisplayName") else {
                    continue;
                };
                if name.trim().to_ascii_lowercase() != target {
                    continue;
                }
                if let Some(cmd) = read_uninstall_string(&app_key) {
                    return Some(cmd);
                }
            }
        }
        None
    }

    fn read_uninstall_string(key: &RegKey) -> Option<String> {
        key.get_value::<String, _>("QuietUninstallString")
            .ok()
            .or_else(|| key.get_value("UninstallString").ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn is_system_component(key: &RegKey) -> bool {
        key.get_value::<u32, _>("SystemComponent")
            .ok()
            .is_some_and(|v| v == 1)
            || key
                .get_value::<String, _>("ParentKeyName")
                .ok()
                .is_some()
    }

    fn parse_display_icon(raw: &str) -> Option<String> {
        let trimmed = raw.trim().trim_matches('"');
        if trimmed.is_empty() {
            return None;
        }
        let path_part = trimmed.split(',').next()?.trim();
        if path_part.is_empty() {
            return None;
        }
        Some(path_part.replace('/', "\\"))
    }

    fn icon_to_data_url(raw: &str) -> Option<String> {
        let path = parse_display_icon(raw)?;
        let path = Path::new(&path);
        if !path.is_file() {
            return None;
        }
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        let mime = match ext.as_str() {
            "ico" => "image/x-icon",
            "png" => "image/png",
            "bmp" => "image/bmp",
            "jpg" | "jpeg" => "image/jpeg",
            _ => return None,
        };
        let data = std::fs::read(path).ok()?;
        if data.is_empty() || data.len() > MAX_ICON_BYTES {
            return None;
        }
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let encoded = STANDARD.encode(data);
        Some(format!("data:{mime};base64,{encoded}"))
    }
}
