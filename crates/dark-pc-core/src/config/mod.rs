//! Paramètres applicatifs (exclusions de scan, seuils, etc.).

/// Clés de paramètres persistés dans `settings`.
pub mod keys {
    pub const LARGE_FILE_THRESHOLD_MB: &str = "large_file_threshold_mb";
    pub const CONFIRM_SENSITIVE_ACTIONS: &str = "confirm_sensitive_actions";
    pub const NOTIFICATIONS_ENABLED: &str = "notifications_enabled";
    pub const LAUNCH_AT_STARTUP: &str = "launch_at_startup";
    pub const DUPLICATE_KEEP_RULE: &str = "duplicate_keep_rule";
    pub const ANALYSIS_REMINDER_ENABLED: &str = "analysis_reminder_enabled";
    pub const ANALYSIS_REMINDER_INTERVAL_DAYS: &str = "analysis_reminder_interval_days";
    pub const LAST_ANALYSIS_REMINDER_SENT_AT: &str = "last_analysis_reminder_sent_at";
}

/// Valeurs par défaut conservatrices (Windows — première version).
pub fn default_scan_exclusions() -> Vec<&'static str> {
    vec![
        "System Volume Information",
        "$Recycle.Bin",
        "Windows",
        "Program Files",
        "Program Files (x86)",
        "ProgramData",
    ]
}
