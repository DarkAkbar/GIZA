use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use dark_pc_core::Database;

#[derive(Clone)]
pub struct ScanRuntime {
    pub scan_id: i64,
    pub root_path: String,
    pub cancel: Arc<AtomicBool>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSnapshot {
    pub scan_id: Option<i64>,
    pub root_path: Option<String>,
    pub status: String,
    pub files_seen: u64,
    pub bytes_total: u64,
    pub current_path: Option<String>,
    pub error_message: Option<String>,
}

impl ScanSnapshot {
    pub fn idle() -> Self {
        Self {
            scan_id: None,
            root_path: None,
            status: "idle".into(),
            files_seen: 0,
            bytes_total: 0,
            current_path: None,
            error_message: None,
        }
    }
}

#[derive(Clone)]
pub struct DuplicateRuntime {
    pub scan_id: i64,
    pub cancel: Arc<AtomicBool>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSnapshot {
    pub scan_id: Option<i64>,
    pub status: String,
    pub candidates_hashed: u64,
    pub groups_found: u64,
    pub recoverable_bytes: u64,
    pub error_message: Option<String>,
}

impl DuplicateSnapshot {
    pub fn idle() -> Self {
        Self {
            scan_id: None,
            status: "idle".into(),
            candidates_hashed: 0,
            groups_found: 0,
            recoverable_bytes: 0,
            error_message: None,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub db_path: PathBuf,
    pub active_scan: Arc<Mutex<Option<ScanRuntime>>>,
    pub scan_snapshot: Arc<Mutex<ScanSnapshot>>,
    pub active_duplicate: Arc<Mutex<Option<DuplicateRuntime>>>,
    pub duplicate_snapshot: Arc<Mutex<DuplicateSnapshot>>,
    /// Volumes restants pour l'analyse automatique au démarrage (un scan à la fois).
    pub bootstrap_queue: Arc<Mutex<Vec<String>>>,
}
