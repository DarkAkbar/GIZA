-- V2.5 — exclusions utilisateur (chemins absolus)

CREATE TABLE IF NOT EXISTS user_exclusions (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  path       TEXT NOT NULL COLLATE NOCASE,
  created_at TEXT NOT NULL,
  UNIQUE (path)
);

CREATE INDEX IF NOT EXISTS idx_user_exclusions_path ON user_exclusions(path);
