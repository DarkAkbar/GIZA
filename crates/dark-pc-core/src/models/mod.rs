use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Option<i64>,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub accessed_at: Option<String>,
    pub mime_or_kind: Option<String>,
    pub is_system_hint: bool,
    pub content_hash: Option<String>,
    pub category: Option<String>,
    pub risk_level: Option<RiskLevel>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Safe,
    Review,
    Risky,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: Option<i64>,
    pub recommendation_type: String,
    pub title: String,
    pub description: String,
    pub affected_files: Vec<String>,
    pub recoverable_space: u64,
    pub confidence: f32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Trash,
    RestoreNote,
    Move,
    Organize,
    Scan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    pub id: Option<i64>,
    pub operation_type: OperationType,
    pub payload_json: String,
    pub recoverable_bytes: Option<u64>,
    pub reversible: bool,
    pub created_at: String,
}
