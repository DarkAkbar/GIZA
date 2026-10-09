//! Rappel discret de ré-analyse (sans lancement automatique).

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::config::keys;
use crate::db::{scan_repo, settings_repo};
use crate::notifications::PendingNotification;
use crate::settings;

pub const DEFAULT_REMINDER_INTERVAL_DAYS: u64 = 7;
pub const MIN_REMINDER_INTERVAL_DAYS: u64 = 1;
pub const MAX_REMINDER_INTERVAL_DAYS: u64 = 90;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisScheduleStatus {
    pub reminder_enabled: bool,
    pub interval_days: u64,
    pub last_scan_finished_at: Option<String>,
    pub days_since_last_scan: Option<u64>,
    pub stale: bool,
    pub has_ever_scanned: bool,
}

pub fn get_status(conn: &Connection) -> Result<AnalysisScheduleStatus> {
    let settings = settings::load(conn)?;
    let latest = scan_repo::latest_completed_scan(conn)?;
    let has_ever_scanned = latest.is_some();
    let last_scan_finished_at = latest.and_then(|s| s.finished_at);
    let days_since_last_scan = last_scan_finished_at
        .as_deref()
        .and_then(days_since_rfc3339);
    let stale = match days_since_last_scan {
        None => false,
        Some(days) => days >= settings.analysis_reminder_interval_days,
    };
    Ok(AnalysisScheduleStatus {
        reminder_enabled: settings.analysis_reminder_enabled,
        interval_days: settings.analysis_reminder_interval_days,
        last_scan_finished_at,
        days_since_last_scan,
        stale,
        has_ever_scanned,
    })
}

pub fn plan_analysis_reminder(conn: &Connection) -> Result<Option<PendingNotification>> {
    let settings = settings::load(conn)?;
    if !settings.analysis_reminder_enabled || !settings.notifications_enabled {
        return Ok(None);
    }
    let status = get_status(conn)?;
    if !status.stale {
        return Ok(None);
    }
    if reminder_sent_recently(conn, settings.analysis_reminder_interval_days)? {
        return Ok(None);
    }

    let days = status
        .days_since_last_scan
        .unwrap_or(settings.analysis_reminder_interval_days);
    let pending = PendingNotification {
        id: "analysis_reminder".into(),
        title: "Ré-analyser votre stockage ?".into(),
        body: format!(
            "Votre dernière analyse date d'il y a {days} jour(s). Ouvrez Giza pour relancer une indexation (aucun nettoyage automatique)."
        ),
        fingerprint: format!(
            "analysis_reminder|{}|{}",
            settings.analysis_reminder_interval_days, days
        ),
        action_route: Some("/dashboard".into()),
    };
    Ok(Some(pending))
}

pub fn record_reminder_sent(conn: &Connection) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    settings_repo::set(conn, keys::LAST_ANALYSIS_REMINDER_SENT_AT, &now)
}

fn reminder_sent_recently(conn: &Connection, interval_days: u64) -> Result<bool> {
    let Some(raw) = settings_repo::get(conn, keys::LAST_ANALYSIS_REMINDER_SENT_AT)? else {
        return Ok(false);
    };
    let Some(sent) = parse_rfc3339(&raw) else {
        return Ok(false);
    };
    let elapsed = Utc::now().signed_duration_since(sent);
    Ok((elapsed.num_days().max(0) as u64) < interval_days)
}

pub fn days_since_rfc3339(raw: &str) -> Option<u64> {
    let parsed = parse_rfc3339(raw)?;
    let elapsed = Utc::now().signed_duration_since(parsed);
    Some(elapsed.num_days().max(0) as u64)
}

fn parse_rfc3339(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn validate_interval_days(days: u64) -> Result<()> {
    if days < MIN_REMINDER_INTERVAL_DAYS {
        anyhow::bail!(
            "l'intervalle de rappel doit être d'au moins {} jour(s)",
            MIN_REMINDER_INTERVAL_DAYS
        );
    }
    if days > MAX_REMINDER_INTERVAL_DAYS {
        anyhow::bail!(
            "l'intervalle de rappel ne peut pas dépasser {} jours",
            MAX_REMINDER_INTERVAL_DAYS
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::scan_repo;

    fn conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../../../migrations/001_initial.sql"))
            .unwrap();
        conn
    }

    #[test]
    fn not_stale_without_scan() {
        let conn = conn();
        let status = get_status(&conn).unwrap();
        assert!(!status.has_ever_scanned);
        assert!(!status.stale);
    }

    #[test]
    fn stale_when_scan_old() {
        let conn = conn();
        let scan_id = scan_repo::create_scan(&conn, "C:\\Users\\Test").unwrap();
        scan_repo::finish_scan(&conn, scan_id, "completed", None).unwrap();
        conn.execute(
            "UPDATE scans SET finished_at = ?1 WHERE id = ?2",
            ("2000-01-01T00:00:00Z", scan_id),
        )
        .unwrap();
        let status = get_status(&conn).unwrap();
        assert!(status.stale);
    }
}
