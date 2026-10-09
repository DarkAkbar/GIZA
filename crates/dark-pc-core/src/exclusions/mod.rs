//! Exclusions système + utilisateur — logique centrale.

use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Result};
use rusqlite::Connection;

use crate::config;
use crate::db::exclusions_repo::{self, UserExclusionRow};
use crate::scanner::path_matches_exclusion;

#[derive(Debug, Clone)]
pub struct EffectiveExclusions {
    pub system_segments: Vec<String>,
    pub user_paths: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemExclusionInfo {
    pub name: String,
    pub protected: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionsOverview {
    pub system: Vec<SystemExclusionInfo>,
    pub user: Vec<UserExclusionRow>,
}

pub fn system_segments() -> Vec<String> {
    config::default_scan_exclusions()
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub fn effective_from_connection(conn: &Connection) -> Result<EffectiveExclusions> {
    let user_paths = exclusions_repo::list_user_exclusions(conn)?
        .into_iter()
        .map(|row| row.path)
        .collect();
    Ok(EffectiveExclusions {
        system_segments: system_segments(),
        user_paths,
    })
}

pub fn system_only_effective() -> EffectiveExclusions {
    EffectiveExclusions {
        system_segments: system_segments(),
        user_paths: Vec::new(),
    }
}

pub fn get_overview(conn: &Connection) -> Result<ExclusionsOverview> {
    Ok(ExclusionsOverview {
        system: system_segments()
            .into_iter()
            .map(|name| SystemExclusionInfo {
                name,
                protected: true,
            })
            .collect(),
        user: exclusions_repo::list_user_exclusions(conn)?,
    })
}

pub fn path_is_skipped(path: &Path, effective: &EffectiveExclusions) -> bool {
    if path_matches_exclusion(path, &effective.system_segments) {
        return true;
    }
    path_under_user_exclusion(path, &effective.user_paths)
}

pub fn normalize_user_exclusion_path(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("Ce dossier ne peut pas être ajouté aux exclusions.");
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        bail!("Le chemin doit être absolu (ex. C:\\Users\\…).");
    }
    let path = if path.exists() {
        path.canonicalize()
            .map_err(|_| anyhow::anyhow!("Ce dossier est introuvable."))?
    } else {
        bail!("Ce dossier est introuvable.");
    };
    if !path.is_dir() {
        bail!("Seuls les dossiers peuvent être exclus.");
    }
    Ok(format_normalized_dir(&path))
}

pub fn validate_user_exclusion_path(normalized_path: &str) -> Result<()> {
    let path = PathBuf::from(normalized_path);
    if is_protected_system_location(&path) {
        bail!("Ce dossier est protégé par Dark PC et ne peut pas être exclu.");
    }
    Ok(())
}

pub fn add_user_exclusion(conn: &Connection, raw_path: &str) -> Result<UserExclusionRow> {
    let normalized = normalize_user_exclusion_path(raw_path)?;
    validate_user_exclusion_path(&normalized)?;
    if exclusions_repo::find_by_path(conn, &normalized)?.is_some() {
        bail!("Ce dossier est déjà dans vos exclusions.");
    }
    exclusions_repo::insert_user_exclusion(conn, &normalized)
}

pub fn remove_user_exclusion(conn: &Connection, id: i64) -> Result<bool> {
    exclusions_repo::remove_user_exclusion(conn, id)
}

pub fn reset_user_exclusions(conn: &Connection) -> Result<u64> {
    exclusions_repo::clear_user_exclusions(conn)
}

pub fn is_protected_system_location(path: &Path) -> bool {
    if path_matches_exclusion(path, &system_segments()) {
        return true;
    }
    for root in protected_system_roots() {
        if path_is_prefix_of(path, &root) || path_is_prefix_of(&root, path) {
            return true;
        }
    }
    false
}

fn protected_system_roots() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let mut roots = Vec::new();
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let windows = PathBuf::from(format!("{drive}Windows"));
            if windows.is_dir() {
                roots.push(windows);
            }
            for sub in ["Program Files", "Program Files (x86)", "ProgramData"] {
                let p = PathBuf::from(format!("{drive}{sub}"));
                if p.is_dir() {
                    roots.push(p);
                }
            }
        }
        roots
    }
    #[cfg(not(windows))]
    {
        vec![PathBuf::from("/Windows")]
    }
}

fn path_under_user_exclusion(path: &Path, user_paths: &[String]) -> bool {
    for raw in user_paths {
        let base = PathBuf::from(raw);
        if path_is_prefix_of(path, &base) {
            return true;
        }
    }
    false
}

fn path_is_prefix_of(path: &Path, prefix: &Path) -> bool {
    let path_norm = normalize_for_prefix(path);
    let prefix_norm = normalize_for_prefix(prefix);
    if path_norm.len() < prefix_norm.len() {
        return false;
    }
    if !path_norm.starts_with(&prefix_norm) {
        return false;
    }
    if path_norm.len() == prefix_norm.len() {
        return true;
    }
    let next = path_norm.as_bytes()[prefix_norm.len()];
    next == b'\\' || next == b'/'
}

fn normalize_for_prefix(path: &Path) -> String {
    let mut s = format_normalized_dir(path);
    s.make_ascii_lowercase();
    s
}

fn format_normalized_dir(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(p) => parts.push(p.as_os_str().to_string_lossy().into_owned()),
            Component::RootDir => {}
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            _ => {}
        }
    }
    if parts.is_empty() {
        return path.display().to_string();
    }
    if parts[0].ends_with(':') {
        format!("{}\\{}", parts[0], parts[1..].join("\\"))
    } else {
        parts.join("\\")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        Database::open(dir.path().join("t.db")).unwrap()
    }

    #[test]
    fn effective_includes_system_and_user() {
        let mut db = test_db();
        let conn = db.conn_mut();
        add_user_exclusion(conn, std::env::temp_dir().to_str().unwrap()).unwrap();
        let eff = effective_from_connection(conn).unwrap();
        assert!(!eff.system_segments.is_empty());
        assert_eq!(eff.user_paths.len(), 1);
    }

    #[test]
    #[cfg(windows)]
    fn rejects_system_subpath() {
        if !Path::new("C:\\Windows\\System32").is_dir() {
            return;
        }
        let mut db = test_db();
        let conn = db.conn_mut();
        assert!(add_user_exclusion(conn, "C:\\Windows\\System32").is_err());
    }

    #[test]
    fn rejects_duplicate() {
        let mut db = test_db();
        let conn = db.conn_mut();
        let tmp = std::env::temp_dir();
        let raw = tmp.to_str().unwrap();
        add_user_exclusion(conn, raw).unwrap();
        let err = add_user_exclusion(conn, raw).unwrap_err();
        assert!(err.to_string().contains("déjà"));
    }

    #[test]
    fn normalizes_slash_variants() {
        let tmp = std::env::temp_dir();
        let a = normalize_user_exclusion_path(&tmp.to_string_lossy()).unwrap();
        let b = normalize_user_exclusion_path(&tmp.to_string_lossy().replace('\\', "/")).unwrap();
        assert_eq!(a.to_ascii_lowercase(), b.to_ascii_lowercase());
    }

    #[test]
    fn user_prefix_skips_nested_files() {
        let eff = EffectiveExclusions {
            system_segments: system_segments(),
            user_paths: vec!["C:\\Users\\Test\\Private".into()],
        };
        assert!(path_is_skipped(
            Path::new("C:\\Users\\Test\\Private\\secret.txt"),
            &eff
        ));
        assert!(!path_is_skipped(
            Path::new("C:\\Users\\Test\\Public\\file.txt"),
            &eff
        ));
    }

    #[test]
    fn remove_and_persist() {
        let mut db = test_db();
        let conn = db.conn_mut();
        let row = add_user_exclusion(conn, std::env::temp_dir().to_str().unwrap()).unwrap();
        assert!(remove_user_exclusion(conn, row.id).unwrap());
        assert!(exclusions_repo::list_user_exclusions(conn).unwrap().is_empty());
    }
}
