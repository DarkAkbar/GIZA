-- V2.5 — historique d'utilisation des volumes

CREATE TABLE IF NOT EXISTS storage_snapshots (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  mount_path      TEXT NOT NULL,
  total_bytes     INTEGER NOT NULL,
  used_bytes      INTEGER NOT NULL,
  free_bytes      INTEGER NOT NULL,
  usage_percent   REAL NOT NULL,
  created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_storage_snapshots_mount_created
  ON storage_snapshots(mount_path, datetime(created_at) DESC);

CREATE INDEX IF NOT EXISTS idx_storage_snapshots_created
  ON storage_snapshots(datetime(created_at) DESC);
