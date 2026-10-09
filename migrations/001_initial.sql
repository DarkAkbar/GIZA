-- Schéma V1 — métadonnées uniquement

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scans (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  root_path     TEXT NOT NULL,
  status        TEXT NOT NULL CHECK (status IN ('pending','running','completed','cancelled','failed')),
  started_at    TEXT NOT NULL,
  finished_at   TEXT,
  files_seen    INTEGER NOT NULL DEFAULT 0,
  bytes_total   INTEGER NOT NULL DEFAULT 0,
  error_message TEXT
);

CREATE TABLE IF NOT EXISTS files (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  scan_id        INTEGER NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
  path           TEXT NOT NULL,
  name           TEXT NOT NULL,
  extension      TEXT,
  size_bytes     INTEGER NOT NULL,
  created_at     TEXT,
  modified_at    TEXT,
  accessed_at    TEXT,
  mime_or_kind   TEXT,
  is_system_hint INTEGER NOT NULL DEFAULT 0,
  content_hash   TEXT,
  UNIQUE (scan_id, path)
);

CREATE INDEX IF NOT EXISTS idx_files_scan_size ON files(scan_id, size_bytes DESC);
CREATE INDEX IF NOT EXISTS idx_files_scan_hash ON files(scan_id, content_hash)
  WHERE content_hash IS NOT NULL;

CREATE TABLE IF NOT EXISTS duplicate_groups (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  scan_id           INTEGER NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
  content_hash      TEXT NOT NULL,
  file_count        INTEGER NOT NULL,
  total_bytes       INTEGER NOT NULL,
  recoverable_bytes INTEGER NOT NULL,
  UNIQUE (scan_id, content_hash)
);

CREATE TABLE IF NOT EXISTS duplicate_group_members (
  group_id INTEGER NOT NULL REFERENCES duplicate_groups(id) ON DELETE CASCADE,
  file_id  INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  PRIMARY KEY (group_id, file_id)
);

CREATE TABLE IF NOT EXISTS operations (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  type              TEXT NOT NULL CHECK (type IN ('trash','restore_note','move','organize','scan')),
  payload_json      TEXT NOT NULL,
  recoverable_bytes INTEGER,
  reversible        INTEGER NOT NULL DEFAULT 1,
  created_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_operations_created ON operations(created_at DESC);

CREATE TABLE IF NOT EXISTS schema_migrations (
  version    INTEGER PRIMARY KEY NOT NULL,
  applied_at TEXT NOT NULL
);
