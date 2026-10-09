//! Lecture des volumes (sans scan de fichiers).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::Disks;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VolumeStats {
    pub mount_path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f32,
}

/// Volume système par défaut (Windows : lecteur où est installé l'OS).
pub fn default_system_volume() -> &'static str {
    #[cfg(windows)]
    {
        "C:\\"
    }
    #[cfg(not(windows))]
    {
        "/"
    }
}

pub fn volume_stats(mount_hint: Option<&str>) -> Result<VolumeStats> {
    let hint = normalize_mount(mount_hint.unwrap_or_else(|| default_system_volume()));
    let disks = Disks::new_with_refreshed_list();

    let disk = disks
        .list()
        .iter()
        .find(|d| normalize_mount(&d.mount_point().to_string_lossy()) == hint)
        .or_else(|| disks.list().first())
        .context(format!("volume introuvable pour {hint}"))?;

    let total_bytes = disk.total_space();
    let available_bytes = disk.available_space();
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let usage_percent = if total_bytes > 0 {
        ((used_bytes as f64 / total_bytes as f64) * 100.0) as f32
    } else {
        0.0
    };

    Ok(VolumeStats {
        mount_path: disk.mount_point().to_string_lossy().into_owned(),
        total_bytes,
        available_bytes,
        used_bytes,
        usage_percent,
    })
}

/// Tous les volumes montés signalés par le système (sans scan de fichiers).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanRootSuggestion {
    pub mount_path: String,
    pub label: String,
    pub path: String,
    pub scope: String,
}

/// Racines d'analyse recommandées par volume (sans parcours — chemins uniquement).
pub fn scan_root_suggestions() -> Vec<ScanRootSuggestion> {
    let volumes = list_mounted_volumes();
    let user_profile = std::env::var("USERPROFILE").ok();
    let username = user_profile.as_ref().and_then(|p| {
        std::path::Path::new(p)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
    });

    let mut suggestions = Vec::new();
    for vol in volumes {
        let mount = normalize_mount(&vol.mount_path);
        let mut chosen: Option<ScanRootSuggestion> = None;

        if let Some(ref profile) = user_profile {
            let profile_norm = normalize_mount(profile);
            if profile_norm.starts_with(&mount) || profile.starts_with(&mount) {
                chosen = Some(ScanRootSuggestion {
                    mount_path: vol.mount_path.clone(),
                    label: format!("Profil utilisateur ({})", mount.trim_end_matches('\\')),
                    path: profile.clone(),
                    scope: "user_profile".into(),
                });
            }
        }

        #[cfg(windows)]
        if chosen.is_none() {
            if let Some(ref name) = username {
                let candidate = format!("{}Users\\{}", mount, name);
                if std::path::Path::new(&candidate).is_dir() {
                    chosen = Some(ScanRootSuggestion {
                        mount_path: vol.mount_path.clone(),
                        label: format!("Utilisateur {name} ({})", mount.trim_end_matches('\\')),
                        path: candidate,
                        scope: "user_folder".into(),
                    });
                }
            }
        }

        if chosen.is_none() {
            let root = if mount.ends_with('\\') {
                mount.clone()
            } else {
                format!("{mount}\\")
            };
            chosen = Some(ScanRootSuggestion {
                mount_path: vol.mount_path.clone(),
                label: format!(
                    "Volume {} (complet — zones système exclues)",
                    mount.trim_end_matches('\\')
                ),
                path: root,
                scope: "volume".into(),
            });
        }

        if let Some(s) = chosen {
            suggestions.push(s);
        }
    }
    suggestions
}

/// Racines « volume entier » pour l'analyse automatique (exclusions scanner appliquées au parcours).
pub fn bootstrap_volume_roots() -> Vec<String> {
    list_mounted_volumes()
        .into_iter()
        .map(|vol| {
            let mount = normalize_mount(&vol.mount_path);
            if mount.ends_with('\\') {
                mount
            } else {
                format!("{mount}\\")
            }
        })
        .collect()
}

pub fn list_mounted_volumes() -> Vec<VolumeStats> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .map(|disk| {
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space();
            let used_bytes = total_bytes.saturating_sub(available_bytes);
            let usage_percent = if total_bytes > 0 {
                ((used_bytes as f64 / total_bytes as f64) * 100.0) as f32
            } else {
                0.0
            };
            VolumeStats {
                mount_path: disk.mount_point().to_string_lossy().into_owned(),
                total_bytes,
                available_bytes,
                used_bytes,
                usage_percent,
            }
        })
        .collect()
}

pub fn normalize_mount(path: &str) -> String {
    let trimmed = path.trim();
    #[cfg(windows)]
    {
        let upper = trimmed.to_ascii_uppercase();
        if upper.len() >= 2 && upper.as_bytes()[1] == b':' {
            return format!("{}:\\", upper.as_bytes()[0] as char);
        }
    }
    trimmed.trim_end_matches(['\\', '/']).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_windows_drive() {
        assert_eq!(normalize_mount("c:"), "C:\\");
        assert_eq!(normalize_mount("C:\\"), "C:\\");
    }

    #[test]
    fn reads_default_volume() {
        let stats = volume_stats(None).expect("volume système");
        assert!(stats.total_bytes > 0);
        assert!(stats.usage_percent >= 0.0 && stats.usage_percent <= 100.0);
    }

    #[test]
    fn lists_mounted_volumes() {
        let list = list_mounted_volumes();
        assert!(!list.is_empty());
        assert!(list.iter().all(|v| v.usage_percent >= 0.0 && v.usage_percent <= 100.0));
    }

    #[test]
    fn scan_root_suggestions_match_volumes() {
        let volumes = list_mounted_volumes();
        let suggestions = scan_root_suggestions();
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions.len(), volumes.len());
        for s in &suggestions {
            assert!(!s.path.is_empty());
            assert!(!s.mount_path.is_empty());
            assert!(["user_profile", "user_folder", "volume"].contains(&s.scope.as_str()));
        }
    }
}
