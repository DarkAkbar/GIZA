//! Bibliothèque métier Dark PC — testable sans interface Tauri.

pub mod analysis_schedule;
pub mod apps;
pub mod cleanup;
pub mod config;
pub mod dashboard;
pub mod db;
pub mod disk;
pub mod duplicates;
pub mod exclusions;
pub mod large_files;
pub mod models;
pub mod notifications;
pub mod organize;
pub mod recommendations;
pub mod safe_delete;
pub mod scanner;
pub mod settings;
pub mod storage_monitor;

pub use db::Database;
