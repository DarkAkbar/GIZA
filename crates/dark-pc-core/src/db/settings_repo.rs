use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    )
    .optional()
    .context("lecture setting")
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        (key, value),
    )
    .context("écriture setting")?;
    Ok(())
}

pub fn remove(conn: &Connection, key: &str) -> Result<bool> {
    let n = conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
    Ok(n > 0)
}
