//! Préférences utilisateur persistées dans SQLite (`settings`).

use anyhow::{bail, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::config::keys;
use crate::db::settings_repo;

pub const DEFAULT_LARGE_FILE_THRESHOLD_MB: u64 = 100;
pub const MIN_LARGE_FILE_THRESHOLD_MB: u64 = 1;
pub const MAX_LARGE_FILE_THRESHOLD_MB: u64 = 512 * 1024; // 512 Go — plafond anti-valeur absurde

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateKeepRule {
    #[default]
    ShortestPath,
    Newest,
    Oldest,
}

impl DuplicateKeepRule {
    pub fn parse_stored(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "shortestpath" | "shortest_path" | "shortest" => Some(Self::ShortestPath),
            "newest" => Some(Self::Newest),
            "oldest" => Some(Self::Oldest),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShortestPath => "shortestPath",
            Self::Newest => "newest",
            Self::Oldest => "oldest",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub large_file_threshold_mb: u64,
    pub confirm_sensitive_actions: bool,
    pub notifications_enabled: bool,
    /// Préférence enregistrée ; le lancement au démarrage n'est pas encore branché côté OS.
    pub launch_at_startup: bool,
    pub duplicate_keep_rule: DuplicateKeepRule,
    pub analysis_reminder_enabled: bool,
    pub analysis_reminder_interval_days: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            large_file_threshold_mb: DEFAULT_LARGE_FILE_THRESHOLD_MB,
            confirm_sensitive_actions: true,
            notifications_enabled: true,
            launch_at_startup: false,
            duplicate_keep_rule: DuplicateKeepRule::default(),
            analysis_reminder_enabled: true,
            analysis_reminder_interval_days: crate::analysis_schedule::DEFAULT_REMINDER_INTERVAL_DAYS,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettings {
    pub large_file_threshold_mb: Option<u64>,
    pub confirm_sensitive_actions: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub launch_at_startup: Option<bool>,
    pub duplicate_keep_rule: Option<DuplicateKeepRule>,
    pub analysis_reminder_enabled: Option<bool>,
    pub analysis_reminder_interval_days: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsCapabilities {
    pub launch_at_startup_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResponse {
    pub settings: AppSettings,
    pub capabilities: SettingsCapabilities,
}

pub fn validate_large_file_threshold_mb(mb: u64) -> Result<()> {
    if mb < MIN_LARGE_FILE_THRESHOLD_MB {
        bail!(
            "le seuil des gros fichiers doit être d'au moins {} Mo",
            MIN_LARGE_FILE_THRESHOLD_MB
        );
    }
    if mb > MAX_LARGE_FILE_THRESHOLD_MB {
        bail!(
            "le seuil des gros fichiers ne peut pas dépasser {} Mo",
            MAX_LARGE_FILE_THRESHOLD_MB
        );
    }
    Ok(())
}

pub fn large_file_threshold_bytes(conn: &Connection) -> Result<u64> {
    let mb = load(conn)?.large_file_threshold_mb;
    Ok(mb.saturating_mul(1024 * 1024))
}

pub fn load(conn: &Connection) -> Result<AppSettings> {
    let defaults = AppSettings::default();
    Ok(AppSettings {
        large_file_threshold_mb: read_u64(
            conn,
            keys::LARGE_FILE_THRESHOLD_MB,
            defaults.large_file_threshold_mb,
        )?,
        confirm_sensitive_actions: read_bool(
            conn,
            keys::CONFIRM_SENSITIVE_ACTIONS,
            defaults.confirm_sensitive_actions,
        )?,
        notifications_enabled: read_bool(
            conn,
            keys::NOTIFICATIONS_ENABLED,
            defaults.notifications_enabled,
        )?,
        launch_at_startup: read_bool(conn, keys::LAUNCH_AT_STARTUP, defaults.launch_at_startup)?,
        duplicate_keep_rule: read_duplicate_keep_rule(conn, defaults.duplicate_keep_rule)?,
        analysis_reminder_enabled: read_bool(
            conn,
            keys::ANALYSIS_REMINDER_ENABLED,
            defaults.analysis_reminder_enabled,
        )?,
        analysis_reminder_interval_days: read_u64(
            conn,
            keys::ANALYSIS_REMINDER_INTERVAL_DAYS,
            defaults.analysis_reminder_interval_days,
        )?,
    })
}

pub fn get_response(conn: &Connection) -> Result<SettingsResponse> {
    Ok(SettingsResponse {
        settings: load(conn)?,
        capabilities: SettingsCapabilities {
            launch_at_startup_available: false,
        },
    })
}

pub fn update(conn: &Connection, patch: UpdateSettings) -> Result<AppSettings> {
    let mut current = load(conn)?;

    if let Some(mb) = patch.large_file_threshold_mb {
        validate_large_file_threshold_mb(mb)?;
        current.large_file_threshold_mb = mb;
        settings_repo::set(
            conn,
            keys::LARGE_FILE_THRESHOLD_MB,
            &mb.to_string(),
        )?;
    }
    if let Some(v) = patch.confirm_sensitive_actions {
        current.confirm_sensitive_actions = v;
        settings_repo::set(conn, keys::CONFIRM_SENSITIVE_ACTIONS, bool_str(v))?;
    }
    if let Some(v) = patch.notifications_enabled {
        current.notifications_enabled = v;
        settings_repo::set(conn, keys::NOTIFICATIONS_ENABLED, bool_str(v))?;
    }
    if let Some(v) = patch.launch_at_startup {
        current.launch_at_startup = v;
        settings_repo::set(conn, keys::LAUNCH_AT_STARTUP, bool_str(v))?;
    }
    if let Some(rule) = patch.duplicate_keep_rule {
        current.duplicate_keep_rule = rule;
        settings_repo::set(conn, keys::DUPLICATE_KEEP_RULE, rule.as_str())?;
    }
    if let Some(v) = patch.analysis_reminder_enabled {
        current.analysis_reminder_enabled = v;
        settings_repo::set(conn, keys::ANALYSIS_REMINDER_ENABLED, bool_str(v))?;
    }
    if let Some(days) = patch.analysis_reminder_interval_days {
        crate::analysis_schedule::validate_interval_days(days)?;
        current.analysis_reminder_interval_days = days;
        settings_repo::set(
            conn,
            keys::ANALYSIS_REMINDER_INTERVAL_DAYS,
            &days.to_string(),
        )?;
    }

    Ok(current)
}

fn read_duplicate_keep_rule(conn: &Connection, default: DuplicateKeepRule) -> Result<DuplicateKeepRule> {
    let Some(raw) = settings_repo::get(conn, keys::DUPLICATE_KEEP_RULE)? else {
        return Ok(default);
    };
    Ok(DuplicateKeepRule::parse_stored(&raw).unwrap_or(default))
}

fn read_u64(conn: &Connection, key: &str, default: u64) -> Result<u64> {
    let Some(raw) = settings_repo::get(conn, key)? else {
        return Ok(default);
    };
    raw.parse::<u64>()
        .map_err(|_| anyhow::anyhow!("valeur invalide pour {key}"))
}

fn read_bool(conn: &Connection, key: &str, default: bool) -> Result<bool> {
    let Some(raw) = settings_repo::get(conn, key)? else {
        return Ok(default);
    };
    parse_bool(&raw).ok_or_else(|| anyhow::anyhow!("valeur booléenne invalide pour {key}"))
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn bool_str(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../../../migrations/001_initial.sql"))
            .unwrap();
        conn
    }

    #[test]
    fn defaults_without_rows() {
        let conn = db();
        let s = load(&conn).unwrap();
        assert_eq!(s.large_file_threshold_mb, 100);
        assert!(s.confirm_sensitive_actions);
    }

    #[test]
    fn persists_threshold() {
        let conn = db();
        update(
            &conn,
            UpdateSettings {
                large_file_threshold_mb: Some(500),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(load(&conn).unwrap().large_file_threshold_mb, 500);
        assert_eq!(large_file_threshold_bytes(&conn).unwrap(), 500 * 1024 * 1024);
    }

    #[test]
    fn rejects_invalid_threshold() {
        assert!(validate_large_file_threshold_mb(0).is_err());
        assert!(validate_large_file_threshold_mb(MAX_LARGE_FILE_THRESHOLD_MB + 1).is_err());
        let conn = db();
        assert!(
            update(
                &conn,
                UpdateSettings {
                    large_file_threshold_mb: Some(0),
                    ..Default::default()
                },
            )
            .is_err()
        );
    }

    #[test]
    fn persists_duplicate_keep_rule() {
        let conn = db();
        update(
            &conn,
            UpdateSettings {
                duplicate_keep_rule: Some(DuplicateKeepRule::Newest),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(load(&conn).unwrap().duplicate_keep_rule, DuplicateKeepRule::Newest);
    }

    #[test]
    fn persists_booleans() {
        let conn = db();
        update(
            &conn,
            UpdateSettings {
                confirm_sensitive_actions: Some(false),
                notifications_enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        let s = load(&conn).unwrap();
        assert!(!s.confirm_sensitive_actions);
        assert!(!s.notifications_enabled);
    }
}
