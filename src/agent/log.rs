use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

fn ensure_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS agent_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            project TEXT,
            agent_type TEXT NOT NULL,
            task TEXT NOT NULL,
            status TEXT NOT NULL,
            steps_executed INTEGER DEFAULT 0,
            duration_ms INTEGER DEFAULT 0,
            error TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_agent_log_timestamp ON agent_log(timestamp);
        CREATE INDEX IF NOT EXISTS idx_agent_log_type ON agent_log(agent_type);",
    )?;
    Ok(())
}

fn open_db() -> Result<Connection> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("HOME not set"))?;
    let db_path = PathBuf::from(home).join(".dectl").join("memory.db");

    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {:?}", parent))?;
        }
    }

    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open database at {:?}", db_path))?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .context("Failed to enable WAL mode")?;
    Ok(conn)
}

pub fn record_agent_execution(
    agent_type: &str,
    task: &str,
    status: &str,
    steps_executed: usize,
    duration_ms: i64,
    error: Option<&str>,
) -> Result<i64> {
    let conn = open_db()?;
    ensure_table(&conn)?;

    conn.execute(
        "INSERT INTO agent_log (agent_type, task, status, steps_executed, duration_ms, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![agent_type, task, status, steps_executed, duration_ms, error],
    )?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

pub fn query_agent_sessions_since(timestamp: &str) -> Result<usize> {
    let conn = open_db()?;
    let count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_log WHERE timestamp > ?1",
            rusqlite::params![timestamp],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(count)
}
