use dark_pc_core::notifications::{self, PendingNotification};
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;

use crate::state::AppState;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessNotificationsResult {
    pub enabled: bool,
    pub sent_count: u32,
    pub sent_ids: Vec<String>,
    pub skipped_duplicate: u32,
}

fn show_notification(app: &AppHandle, item: &PendingNotification) -> Result<(), String> {
    app.notification()
        .builder()
        .title(&item.title)
        .body(&item.body)
        .show()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_notification_state(
    state: State<'_, AppState>,
) -> Result<notifications::NotificationStateOverview, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    notifications::get_notification_state(db.conn()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn process_notifications(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProcessNotificationsResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let plan = notifications::plan_notifications(db.conn()).map_err(|e| e.to_string())?;
    if !plan.enabled {
        return Ok(ProcessNotificationsResult {
            enabled: false,
            sent_count: 0,
            sent_ids: Vec::new(),
            skipped_duplicate: plan.skipped_duplicate,
        });
    }

    let mut sent = Vec::new();
    for item in &plan.to_send {
        show_notification(&app, item)?;
        sent.push(item.clone());
    }
    notifications::commit_sent(db.conn(), &sent).map_err(|e| e.to_string())?;

    Ok(ProcessNotificationsResult {
        enabled: true,
        sent_count: sent.len() as u32,
        sent_ids: sent.iter().map(|s| s.id.clone()).collect(),
        skipped_duplicate: plan.skipped_duplicate,
    })
}

pub fn try_notify_scan_finished(app: &AppHandle, state: &AppState, scan_id: i64, status: &str) {
    let Ok(db) = state.db.lock() else {
        return;
    };
    let Ok(Some(item)) = notifications::plan_scan_finished(db.conn(), scan_id, status) else {
        return;
    };
    if show_notification(app, &item).is_err() {
        return;
    }
    let _ = notifications::commit_sent(db.conn(), &[item]);
}
