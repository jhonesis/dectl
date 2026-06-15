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

        let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");

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

        if current_version < 2 {
            self.conn.execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                    content,
                    tags UNINDEXED,
                    content='memories',
                    content_rowid='id'
                );

                CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                    INSERT INTO memories_fts(rowid, content, tags) VALUES (new.id, new.content, new.tags);
                END;

                CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                    INSERT INTO memories_fts(memories_fts, rowid, content, tags) VALUES('delete', old.id, old.content, old.tags);
                END;

                CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
                    INSERT INTO memories_fts(memories_fts, rowid, content, tags) VALUES('delete', old.id, old.content, old.tags);
                    INSERT INTO memories_fts(rowid, content, tags) VALUES (new.id, new.content, new.tags);
                END;",
            )
            .context("Migration v2: failed to create FTS5 virtual table")?;

            let has_type_col: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM pragma_table_info('memories') WHERE name='type'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if !has_type_col {
                self.conn
                    .execute_batch(
                        "ALTER TABLE memories ADD COLUMN type TEXT NOT NULL DEFAULT 'note'",
                    )
                    .context("Migration v2: failed to add type column")?;
            }

            self.conn.execute(
                "INSERT INTO memories_fts(rowid, content, tags) SELECT id, content, tags FROM memories",
                [],
            )?;

            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO migrations (version, name, applied_at) VALUES (2, 'fts5_and_types', ?1)",
                params![now],
            )?;
        }

        if current_version < 3 {
            self.conn
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS agent_outputs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    agent_type TEXT NOT NULL,
                    task_id TEXT,
                    task_description TEXT,
                    output_file TEXT,
                    memory_id INTEGER,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE SET NULL
                );
                CREATE INDEX IF NOT EXISTS idx_agent_outputs_agent ON agent_outputs(agent_type);
                CREATE INDEX IF NOT EXISTS idx_agent_outputs_task ON agent_outputs(task_id);",
                )
                .context("Migration v3: failed to create agent_outputs table")?;

            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO migrations (version, name, applied_at) VALUES (3, 'agent_outputs', ?1)",
                params![now],
            )?;
        }

        if current_version < 4 {
            self.conn
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS tag_taxonomy (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    category TEXT NOT NULL DEFAULT 'general',
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE INDEX IF NOT EXISTS idx_tag_taxonomy_category ON tag_taxonomy(category);",
                )
                .context("Migration v4: failed to create tag_taxonomy table")?;

            let seed_tags = vec![
                ("decision", "Arquitectural or design decisions", "type"),
                ("note", "General notes and observations", "type"),
                ("context", "Project context and requirements", "type"),
                ("research", "Research findings and analysis", "type"),
                ("incident", "Issues, bugs, and incidents", "type"),
                ("session", "Session summaries and logs", "meta"),
                ("high-impact", "Critical information worth surfacing", "priority"),
                ("agent", "Agent-generated content", "source"),
                ("code-snippet", "Code fragments and examples", "type"),
            ];

            for (name, desc, category) in &seed_tags {
                self.conn.execute(
                    "INSERT OR IGNORE INTO tag_taxonomy (name, description, category) VALUES (?1, ?2, ?3)",
                    params![name, desc, category],
                )?;
            }

            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO migrations (version, name, applied_at) VALUES (4, 'tag_taxonomy', ?1)",
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
    #[serde(rename = "type")]
    pub type_: String,
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
            type_: row.get(6)?,
        })
    }
}

pub const VALID_TYPES: &[&str] = &[
    "note",
    "decision",
    "context",
    "research",
    "incident",
    "code-snippet",
];

pub const MEMORY_SELECT_COLS: &str = "id, content, tags, project, created_at, updated_at, type";
