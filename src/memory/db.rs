use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct DbConn {
    conn: Connection,
}

impl DbConn {
    pub fn new() -> Result<Self> {
        let db_path = Self::db_path()?;
        Self::open(&db_path)
    }

    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = DbConn { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn open(path: &PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create {:?}", parent))?;
            }
        }

        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .context("Failed to enable WAL mode")?;

        let db = DbConn { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    fn db_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".dectl").join("memory.db"))
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )",
        )?;

        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                tags TEXT,
                project TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            )",
        )?;

        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_memories_project ON memories(project)",
        )?;

        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC)",
        )?;

        let current_version: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version < 1 {
            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO migrations (version, name, applied_at) VALUES (1, 'initial', ?1)",
                params![now],
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryEntry {
    pub id: i64,
    pub content: String,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl MemoryEntry {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let tags_str: String = row.get(2)?;
        let tags: Vec<String> = if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split(',').map(|s| s.trim().to_string()).collect()
        };

        Ok(MemoryEntry {
            id: row.get(0)?,
            content: row.get(1)?,
            tags,
            project: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}
