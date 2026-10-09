//! V2.5 — instantanés périodiques de l'espace disque (sans scan de fichiers).

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::storage_snapshot_repo;
use crate::disk::{self, VolumeStats};

/// Intervalle minimal entre deux snapshots pour un même volume (anti-spam).
pub const MIN_SNAPSHOT_INTERVAL_SECS: i64 = 900;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordSnapshotsResult {
    pub recorded_count: u32,
    pub skipped_cooldown_count: u32,
    pub volumes_seen: u32,
    pub snapshots: Vec<StorageSnapshotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSnapshotDto {
    pub id: i64,
    pub mount_path: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f32,
    pub created_at: String,
}

pub fn record_snapshots(conn: &Connection, force: bool) -> Result<RecordSnapshotsResult> {
    let volumes = disk::list_mounted_volumes();
    let mut recorded_count = 0u32;
    let mut skipped_cooldown_count = 0u32;
    let mut snapshots = Vec::new();

    for vol in &volumes {
        if !force && within_cooldown(conn, &vol.mount_path)? {
            skipped_cooldown_count += 1;
            continue;
        }
        let id = storage_snapshot_repo::insert(
            conn,
            &vol.mount_path,
            vol.total_bytes,
            vol.used_bytes,
            vol.available_bytes,
            vol.usage_percent,
        )?;
        recorded_count += 1;
        snapshots.push(snapshot_dto(id, vol));
    }

    Ok(RecordSnapshotsResult {
        recorded_count,
        skipped_cooldown_count,
        volumes_seen: volumes.len() as u32,
        snapshots,
    })
}

fn within_cooldown(conn: &Connection, mount_path: &str) -> Result<bool> {
    let Some(last) = storage_snapshot_repo::latest_created_at(conn, mount_path)? else {
        return Ok(false);
    };
    let parsed = DateTime::parse_from_rfc3339(&last)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let elapsed = Utc::now().signed_duration_since(parsed);
    Ok(elapsed.num_seconds() < MIN_SNAPSHOT_INTERVAL_SECS)
}

fn snapshot_dto(id: i64, vol: &VolumeStats) -> StorageSnapshotDto {
    StorageSnapshotDto {
        id,
        mount_path: vol.mount_path.clone(),
        total_bytes: vol.total_bytes,
        used_bytes: vol.used_bytes,
        free_bytes: vol.available_bytes,
        usage_percent: vol.usage_percent,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

pub const DEFAULT_HISTORY_LIMIT: u32 = 400;
pub const CHART_MAX_POINTS: usize = 80;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageHistoryQuery {
    pub mount_path: Option<String>,
    pub period_days: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageHistoryResult {
    pub mount_path: String,
    pub available_mounts: Vec<String>,
    pub period_days: u32,
    pub snapshots: Vec<StorageSnapshotDto>,
}

pub fn get_storage_history(conn: &Connection, query: StorageHistoryQuery) -> Result<StorageHistoryResult> {
    let period_days = query.period_days.unwrap_or(7).clamp(1, 90);
    let fetch_limit = query.limit.unwrap_or(DEFAULT_HISTORY_LIMIT).clamp(10, 2_000);
    let available_mounts = available_mount_paths(conn);
    let mount_path = resolve_mount_path(conn, query.mount_path.as_deref(), &available_mounts);

    let since = (Utc::now() - chrono::Duration::days(period_days as i64)).to_rfc3339();
    let rows = storage_snapshot_repo::list_history(conn, &mount_path, &since, fetch_limit)?;
    let snapshots = downsample_for_chart(
        rows.into_iter().map(row_to_dto).collect(),
        CHART_MAX_POINTS,
    );

    Ok(StorageHistoryResult {
        mount_path,
        available_mounts,
        period_days,
        snapshots,
    })
}

fn available_mount_paths(conn: &Connection) -> Vec<String> {
    let mut paths: Vec<String> = disk::list_mounted_volumes()
        .into_iter()
        .map(|v| v.mount_path)
        .collect();
    if let Ok(stored) = storage_snapshot_repo::list_distinct_mount_paths(conn) {
        for p in stored {
            if !paths.iter().any(|x| disk::normalize_mount(x) == disk::normalize_mount(&p)) {
                paths.push(p);
            }
        }
    }
    paths.sort_by(|a, b| disk::normalize_mount(a).cmp(&disk::normalize_mount(b)));
    paths
}

fn resolve_mount_path(_conn: &Connection, hint: Option<&str>, available: &[String]) -> String {
    if let Some(h) = hint.filter(|s| !s.is_empty()) {
        let norm = disk::normalize_mount(h);
        if let Some(found) = available
            .iter()
            .find(|p| disk::normalize_mount(p) == norm)
        {
            return found.clone();
        }
        return h.to_string();
    }
    if let Some(first) = available.first() {
        return first.clone();
    }
    disk::default_system_volume().to_string()
}

fn row_to_dto(row: storage_snapshot_repo::StorageSnapshotRow) -> StorageSnapshotDto {
    StorageSnapshotDto {
        id: row.id,
        mount_path: row.mount_path,
        total_bytes: row.total_bytes.max(0) as u64,
        used_bytes: row.used_bytes.max(0) as u64,
        free_bytes: row.free_bytes.max(0) as u64,
        usage_percent: row.usage_percent as f32,
        created_at: row.created_at,
    }
}

fn downsample_for_chart(points: Vec<StorageSnapshotDto>, max_points: usize) -> Vec<StorageSnapshotDto> {
    if points.len() <= max_points {
        return points;
    }
    let last_index = points.len() - 1;
    let mut out = Vec::with_capacity(max_points);
    for i in 0..max_points {
        let t = if max_points <= 1 {
            0.0
        } else {
            i as f64 / (max_points - 1) as f64
        };
        let idx = (t * last_index as f64).round() as usize;
        out.push(points[idx].clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn records_and_respects_cooldown() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn();

        let first = record_snapshots(conn, true).unwrap();
        assert!(first.recorded_count >= 1);

        let second = record_snapshots(conn, false).unwrap();
        assert_eq!(second.recorded_count, 0);
        assert!(second.skipped_cooldown_count >= 1);

        let forced = record_snapshots(conn, true).unwrap();
        assert!(forced.recorded_count >= 1);
    }

    #[test]
    fn history_filters_by_mount_and_period() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn();

        storage_snapshot_repo::insert(conn, "C:\\", 1000, 600, 400, 60.0).unwrap();
        let old = (Utc::now() - chrono::Duration::days(10)).to_rfc3339();
        conn.execute(
            "UPDATE storage_snapshots SET created_at = ?1 WHERE mount_path = 'C:\\'",
            [&old],
        )
        .unwrap();
        storage_snapshot_repo::insert(conn, "C:\\", 1000, 650, 350, 65.0).unwrap();
        storage_snapshot_repo::insert(conn, "D:\\", 2000, 1000, 1000, 50.0).unwrap();

        let history = get_storage_history(
            conn,
            StorageHistoryQuery {
                mount_path: Some("C:\\".into()),
                period_days: Some(7),
                limit: None,
            },
        )
        .unwrap();
        assert_eq!(disk::normalize_mount(&history.mount_path), "C:\\");
        assert_eq!(history.snapshots.len(), 1);
        assert!(history
            .available_mounts
            .iter()
            .any(|p| p.starts_with("C:") || p.starts_with("c:") || p.starts_with("D:")));
    }

    #[test]
    fn history_empty_when_no_rows_in_period() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn();
        let history = get_storage_history(
            conn,
            StorageHistoryQuery {
                mount_path: Some("Z:\\".into()),
                period_days: Some(7),
                limit: None,
            },
        )
        .unwrap();
        assert!(history.snapshots.is_empty());
    }

    #[test]
    fn persisted_rows_match_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path().join("t.db")).unwrap();
        let conn = db.conn();
        record_snapshots(conn, true).unwrap();
        let rows = storage_snapshot_repo::list_recent(conn, 10, None).unwrap();
        assert!(!rows.is_empty());
        for row in rows {
            assert!(row.total_bytes >= 0);
            assert!(row.usage_percent >= 0.0 && row.usage_percent <= 100.0);
            assert_eq!(
                row.used_bytes + row.free_bytes,
                row.total_bytes as i64
            );
        }
    }
}
