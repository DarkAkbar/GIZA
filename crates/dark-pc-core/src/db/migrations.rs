use anyhow::{Context, Result};
use rusqlite::Connection;

const MIGRATION_001: &str = include_str!("../../../../migrations/001_initial.sql");
const MIGRATION_002: &str = include_str!("../../../../migrations/002_user_exclusions.sql");
const MIGRATION_003: &str = include_str!("../../../../migrations/003_storage_snapshots.sql");

pub fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(MIGRATION_001).context("migration 001")?;
    record_if_missing(conn, 1)?;
    conn.execute_batch(MIGRATION_002).context("migration 002")?;
    record_if_missing(conn, 2)?;
    conn.execute_batch(MIGRATION_003).context("migration 003")?;
    record_if_missing(conn, 3)?;
    Ok(())
}

fn record_if_missing(conn: &Connection, version: i64) -> Result<()> {
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
            [version],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if exists == 0 {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            (version, now),
        )
        .context("schema_migrations")?;
    }
    Ok(())
}
