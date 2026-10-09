//! V2 — Analyse des fichiers temporaires (lecture seule + corbeille via safe_delete).

use std::env;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::models::RiskLevel;
use crate::scanner::path_matches_exclusion;
use crate::config;

pub const DEFAULT_MAX_FILES: u32 = 4_000;
pub const MAX_DEPTH: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCategorySummary {
    pub category: String,
    pub label: String,
    pub file_count: u64,
    pub total_bytes: u64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupItem {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
    pub category: String,
    pub category_label: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupAnalysisResult {
    pub roots_scanned: Vec<String>,
    pub truncated: bool,
    pub items: Vec<CleanupItem>,
    pub categories: Vec<CleanupCategorySummary>,
}

pub fn analyze_temp_files(max_files: u32) -> Result<CleanupAnalysisResult> {
    let limit = max_files.max(100).min(10_000) as usize;
    let exclusions: Vec<String> = config::default_scan_exclusions()
        .into_iter()
        .map(str::to_string)
        .collect();

    let roots = discover_cleanup_roots();
    let mut items = Vec::new();
    let mut truncated = false;

    'roots: for root in &roots {
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(root)
            .follow_links(false)
            .max_depth(MAX_DEPTH)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if items.len() >= limit {
                truncated = true;
                break 'roots;
            }
            let entry_type = entry.file_type();
            if !entry_type.is_file() {
                continue;
            }
            let path = entry.path();
            if path_matches_exclusion(path, &exclusions) {
                continue;
            }
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let (category, category_label, risk) = classify_path(path);
            let modified_at = metadata.modified().ok().and_then(format_system_time);
            items.push(CleanupItem {
                path: path.to_string_lossy().into_owned(),
                name: entry.file_name().to_string_lossy().into_owned(),
                size_bytes: metadata.len(),
                modified_at,
                category: category.to_string(),
                category_label: category_label.to_string(),
                risk_level: risk,
            });
        }
    }

    let categories = summarize_categories(&items);
    Ok(CleanupAnalysisResult {
        roots_scanned: roots
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
        truncated,
        items,
        categories,
    })
}

fn discover_cleanup_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["TEMP", "TMP"] {
        if let Ok(value) = env::var(key) {
            let path = PathBuf::from(&value);
            if path.is_dir() && !roots.iter().any(|r| r == &path) {
                roots.push(path);
            }
        }
    }
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let user_temp = PathBuf::from(&local).join("Temp");
        if user_temp.is_dir() && !roots.iter().any(|r| r == &user_temp) {
            roots.push(user_temp);
        }
        let chrome_cache = PathBuf::from(&local)
            .join("Google")
            .join("Chrome")
            .join("User Data")
            .join("Default")
            .join("Cache");
        if chrome_cache.is_dir() {
            roots.push(chrome_cache);
        }
        let edge_cache = PathBuf::from(&local)
            .join("Microsoft")
            .join("Edge")
            .join("User Data")
            .join("Default")
            .join("Cache");
        if edge_cache.is_dir() {
            roots.push(edge_cache);
        }
    }
    if let Ok(windir) = env::var("WINDIR") {
        let win_temp = PathBuf::from(windir).join("Temp");
        if win_temp.is_dir() {
            roots.push(win_temp);
        }
    }
    roots
}

fn classify_path(path: &Path) -> (&'static str, &'static str, RiskLevel) {
    let lower = path.to_string_lossy().to_lowercase();
    if lower.contains("\\windows\\temp")
        || lower.contains("\\windows\\softwaredistribution\\")
        || lower.contains("\\windows\\logs\\")
    {
        return ("system", "Système Windows", RiskLevel::Risky);
    }
    if lower.contains("\\cache") || lower.contains("\\code cache") {
        return ("browser_cache", "Cache navigateur", RiskLevel::Review);
    }
    if lower.contains("\\temp\\") || lower.contains("\\tmp\\") {
        if is_older_than_days(path, 14) {
            return ("user_temp", "Temp utilisateur", RiskLevel::Safe);
        }
        return ("user_temp", "Temp utilisateur", RiskLevel::Review);
    }
    ("misc_temp", "Fichier temporaire", RiskLevel::Review)
}

fn is_older_than_days(path: &Path, days: u64) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return false;
    };
    age.as_secs() >= days * 86_400
}

fn format_system_time(time: SystemTime) -> Option<String> {
    let datetime: chrono::DateTime<chrono::Utc> = time.into();
    Some(datetime.to_rfc3339())
}

fn summarize_categories(items: &[CleanupItem]) -> Vec<CleanupCategorySummary> {
    use std::collections::HashMap;
    let mut map: HashMap<String, (String, RiskLevel, u64, u64)> = HashMap::new();
    for item in items {
        let entry = map
            .entry(item.category.clone())
            .or_insert((item.category_label.clone(), item.risk_level, 0, 0));
        entry.2 += 1;
        entry.3 = entry.3.saturating_add(item.size_bytes);
        if risk_rank(item.risk_level) > risk_rank(entry.1) {
            entry.1 = item.risk_level;
        }
    }
    let mut categories: Vec<CleanupCategorySummary> = map
        .into_iter()
        .map(|(category, (label, risk, count, bytes))| CleanupCategorySummary {
            category,
            label,
            file_count: count,
            total_bytes: bytes,
            risk_level: risk,
        })
        .collect();
    categories.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    categories
}

fn risk_rank(level: RiskLevel) -> u8 {
    match level {
        RiskLevel::Safe => 0,
        RiskLevel::Review => 1,
        RiskLevel::Risky => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_user_temp_files() {
        let dir = tempdir().unwrap();
        let tmp = dir.path().join("Temp");
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("old.tmp"), b"hello").unwrap();

        env::set_var("TEMP", tmp.as_os_str());
        let result = analyze_temp_files(100).unwrap();
        assert!(!result.items.is_empty());
        assert!(result.items.iter().any(|i| i.name == "old.tmp"));
    }
}
