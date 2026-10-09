//! V2.5 étape 8 — Notifications locales (planification + anti-spam, sans IA).

use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::analysis_schedule;
use crate::db::settings_repo;
use crate::recommendations::{self, RecommendationDto};
use crate::settings;

pub const DEDUPE_SETTINGS_KEY: &str = "notification_dedupe_state";
pub const MAX_NOTIFICATIONS_PER_RUN: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingNotification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub fingerprint: String,
    pub action_route: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStateEntry {
    pub fingerprint: String,
    pub last_sent_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDedupeState {
    pub entries: std::collections::HashMap<String, NotificationStateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPlanResult {
    pub enabled: bool,
    pub to_send: Vec<PendingNotification>,
    pub skipped_duplicate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStateOverview {
    pub enabled: bool,
    pub dedupe_entries: u32,
}

pub fn notifications_enabled(conn: &Connection) -> Result<bool> {
    Ok(settings::load(conn)?.notifications_enabled)
}

pub fn get_notification_state(conn: &Connection) -> Result<NotificationStateOverview> {
    let enabled = notifications_enabled(conn)?;
    let state = load_dedupe_state(conn)?;
    Ok(NotificationStateOverview {
        enabled,
        dedupe_entries: state.entries.len() as u32,
    })
}

/// Planifie les notifications à envoyer (recommandations importantes / warning uniquement).
pub fn plan_notifications(conn: &Connection) -> Result<NotificationPlanResult> {
    if !notifications_enabled(conn)? {
        return Ok(NotificationPlanResult {
            enabled: false,
            to_send: Vec::new(),
            skipped_duplicate: 0,
        });
    }

    let recos = recommendations::generate_recommendations(conn)?.recommendations;
    let mut candidates = Vec::new();
    candidates.extend(notifications_from_recommendations(&recos));
    if let Some(reminder) = analysis_schedule::plan_analysis_reminder(conn)? {
        candidates.push(reminder);
    }
    candidates.truncate(MAX_NOTIFICATIONS_PER_RUN);

    let dedupe = load_dedupe_state(conn)?;
    let mut to_send = Vec::new();
    let mut skipped = 0_u32;
    for item in candidates {
        if should_skip(&dedupe, &item) {
            skipped += 1;
            continue;
        }
        to_send.push(item);
    }

    Ok(NotificationPlanResult {
        enabled: true,
        to_send,
        skipped_duplicate: skipped,
    })
}

pub fn plan_scan_finished(conn: &Connection, scan_id: i64, status: &str) -> Result<Option<PendingNotification>> {
    if !notifications_enabled(conn)? {
        return Ok(None);
    }
    if status != "completed" {
        return Ok(None);
    }
    let id = format!("scan_finished_{scan_id}");
    let fingerprint = format!("{scan_id}|{status}");
    let pending = PendingNotification {
        id: id.clone(),
        title: "Analyse terminée".into(),
        body: "Votre analyse de stockage est terminée. Consultez le Dashboard pour les résultats.".into(),
        fingerprint,
        action_route: Some("/dashboard".into()),
    };
    let dedupe = load_dedupe_state(conn)?;
    if should_skip(&dedupe, &pending) {
        return Ok(None);
    }
    Ok(Some(pending))
}

pub fn commit_sent(conn: &Connection, sent: &[PendingNotification]) -> Result<()> {
    if sent.is_empty() {
        return Ok(());
    }
    let mut dedupe = load_dedupe_state(conn)?;
    let now = Utc::now().to_rfc3339();
    for n in sent {
        if n.id == "analysis_reminder" {
            let _ = analysis_schedule::record_reminder_sent(conn);
        }
        dedupe.entries.insert(
            n.id.clone(),
            NotificationStateEntry {
                fingerprint: n.fingerprint.clone(),
                last_sent_at: now.clone(),
            },
        );
    }
    save_dedupe_state(conn, &dedupe)
}

fn notifications_from_recommendations(recos: &[RecommendationDto]) -> Vec<PendingNotification> {
    recos
        .iter()
        .filter(|r| r.severity == "important" || r.severity == "warning")
        .map(reco_to_notification)
        .collect()
}

fn reco_to_notification(rec: &RecommendationDto) -> PendingNotification {
    let fingerprint = format!(
        "{}|{}|{}",
        rec.id,
        rec.detected_value.unwrap_or(0.0),
        rec.severity
    );
    PendingNotification {
        id: rec.id.clone(),
        title: rec.title.clone(),
        body: format!("{} {}", rec.description, rec.reason),
        fingerprint,
        action_route: Some(rec.action_route.clone()),
    }
}

fn should_skip(state: &NotificationDedupeState, item: &PendingNotification) -> bool {
    let Some(prev) = state.entries.get(&item.id) else {
        return false;
    };
    prev.fingerprint == item.fingerprint
}

fn load_dedupe_state(conn: &Connection) -> Result<NotificationDedupeState> {
    let Some(raw) = settings_repo::get(conn, DEDUPE_SETTINGS_KEY)? else {
        return Ok(NotificationDedupeState::default());
    };
    serde_json::from_str(&raw).map_err(Into::into)
}

fn save_dedupe_state(conn: &Connection, state: &NotificationDedupeState) -> Result<()> {
    let json = serde_json::to_string(state)?;
    settings_repo::set(conn, DEDUPE_SETTINGS_KEY, &json)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> crate::db::Database {
        let dir = tempfile::tempdir().unwrap();
        crate::db::Database::open(dir.path().join("t.db")).unwrap()
    }

    #[test]
    fn disabled_returns_empty_plan() {
        let mut db = db();
        let conn = db.conn_mut();
        settings::update(
            conn,
            settings::UpdateSettings {
                notifications_enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        let plan = plan_notifications(conn).unwrap();
        assert!(!plan.enabled);
        assert!(plan.to_send.is_empty());
    }

    #[test]
    fn dedupe_skips_same_fingerprint() {
        let item = PendingNotification {
            id: "storage_usage_high".into(),
            title: "T".into(),
            body: "B".into(),
            fingerprint: "storage_usage_high|92|important".into(),
            action_route: None,
        };
        let mut state = NotificationDedupeState::default();
        assert!(!should_skip(&state, &item));
        state.entries.insert(
            item.id.clone(),
            NotificationStateEntry {
                fingerprint: item.fingerprint.clone(),
                last_sent_at: Utc::now().to_rfc3339(),
            },
        );
        assert!(should_skip(&state, &item));
        let mut changed = item.clone();
        changed.fingerprint = "storage_usage_high|93|important".into();
        assert!(!should_skip(&state, &changed));
    }

    #[test]
    fn scan_finished_only_on_completed() {
        let mut db = db();
        let conn = db.conn_mut();
        assert!(plan_scan_finished(conn, 1, "failed").unwrap().is_none());
        let n = plan_scan_finished(conn, 2, "completed").unwrap();
        assert!(n.is_some());
    }

    #[test]
    fn commit_persists_dedupe() {
        let mut db = db();
        let conn = db.conn_mut();
        let n = PendingNotification {
            id: "duplicates_detected".into(),
            title: "Doublons".into(),
            body: "Body".into(),
            fingerprint: "duplicates_detected|5|warning".into(),
            action_route: Some("/duplicates".into()),
        };
        commit_sent(conn, &[n]).unwrap();
        let state = load_dedupe_state(conn).unwrap();
        assert!(state.entries.contains_key("duplicates_detected"));
    }
}
