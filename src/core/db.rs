use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

pub trait Storage: Send + Sync {
    fn execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<usize>;
    fn execute_batch(&self, sql: &str) -> Result<()>;
    fn last_insert_rowid(&self) -> i64;
    fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], f: F) -> Result<T>
    where
        T: Send,
        F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T>;
    fn query_map<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
        f: F,
    ) -> Result<Vec<T>>
    where
        T: Send,
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>;
}

struct RealStorage {
    conn: Mutex<Connection>,
}

impl Storage for RealStorage {
    fn execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.execute(sql, params)?)
    }

    fn execute_batch(&self, sql: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.execute_batch(sql)?)
    }

    fn last_insert_rowid(&self) -> i64 {
        let conn = self.conn.lock().unwrap();
        conn.last_insert_rowid()
    }

    fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], f: F) -> Result<T>
    where
        T: Send,
        F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row(sql, params, f)?)
    }

    fn query_map<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
        mut f: F,
    ) -> Result<Vec<T>>
    where
        T: Send,
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| f(row))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

static DB: OnceCell<RealStorage> = OnceCell::new();

pub fn init_db() -> Result<()> {
    get_db()?;
    Ok(())
}

pub fn get_db() -> Result<&'static impl Storage> {
    DB.get_or_try_init(|| {
        let db_path = db_path()?;
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create {:?}", parent))?;
            }
        }
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;
        let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");
        run_migrations(&conn)?;
        Ok(RealStorage {
            conn: Mutex::new(conn),
        })
    })
}

fn db_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home).join(".dectl").join("memory.db"))
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
    )?;

    conn.execute_batch(
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

    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_memories_project ON memories(project)")?;

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC)",
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO migrations (version, name, applied_at) VALUES (1, 'initial', ?1)",
            params![now],
        )?;
    }

    if current_version < 2 {
        conn.execute_batch(
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

        let has_type_col: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('memories') WHERE name='type'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_type_col {
            conn.execute_batch("ALTER TABLE memories ADD COLUMN type TEXT NOT NULL DEFAULT 'note'")
                .context("Migration v2: failed to add type column")?;
        }

        conn.execute(
            "INSERT INTO memories_fts(rowid, content, tags) SELECT id, content, tags FROM memories",
            [],
        )?;

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO migrations (version, name, applied_at) VALUES (2, 'fts5_and_types', ?1)",
            params![now],
        )?;
    }

    if current_version < 3 {
        conn.execute_batch(
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
        conn.execute(
            "INSERT INTO migrations (version, name, applied_at) VALUES (3, 'agent_outputs', ?1)",
            params![now],
        )?;
    }

    if current_version < 4 {
        conn.execute_batch(
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
            (
                "high-impact",
                "Critical information worth surfacing",
                "priority",
            ),
            ("agent", "Agent-generated content", "source"),
            ("code-snippet", "Code fragments and examples", "type"),
        ];

        for (name, desc, category) in &seed_tags {
            conn.execute(
                "INSERT OR IGNORE INTO tag_taxonomy (name, description, category) VALUES (?1, ?2, ?3)",
                params![name, desc, category],
            )?;
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO migrations (version, name, applied_at) VALUES (4, 'tag_taxonomy', ?1)",
            params![now],
        )?;
    }

    Ok(())
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use rusqlite::Connection;

    pub struct InMemoryStorage {
        conn: Mutex<Connection>,
    }

    impl InMemoryStorage {
        pub fn new() -> Result<Self> {
            let conn = Connection::open_in_memory()?;
            conn.execute_batch(
                "CREATE TABLE memories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content TEXT NOT NULL,
                    tags TEXT,
                    project TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    deleted_at TEXT,
                    type TEXT NOT NULL DEFAULT 'note'
                )",
            )?;
            Ok(Self {
                conn: Mutex::new(conn),
            })
        }
    }

    impl Storage for InMemoryStorage {
        fn execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<usize> {
            let conn = self.conn.lock().unwrap();
            Ok(conn.execute(sql, params)?)
        }

        fn execute_batch(&self, sql: &str) -> Result<()> {
            let conn = self.conn.lock().unwrap();
            Ok(conn.execute_batch(sql)?)
        }

        fn last_insert_rowid(&self) -> i64 {
            let conn = self.conn.lock().unwrap();
            conn.last_insert_rowid()
        }

        fn query_row<T, F>(
            &self,
            sql: &str,
            params: &[&dyn rusqlite::types::ToSql],
            f: F,
        ) -> Result<T>
        where
            T: Send,
            F: FnOnce(&rusqlite::Row) -> rusqlite::Result<T>,
        {
            let conn = self.conn.lock().unwrap();
            Ok(conn.query_row(sql, params, f)?)
        }

        fn query_map<T, F>(
            &self,
            sql: &str,
            params: &[&dyn rusqlite::types::ToSql],
            mut f: F,
        ) -> Result<Vec<T>>
        where
            T: Send,
            F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
        {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(params, |row| f(row))?;
            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok(result)
        }
    }

    #[test]
    fn test_in_memory_storage_execute_and_query() {
        let storage = InMemoryStorage::new().unwrap();
        storage
            .execute(
                "INSERT INTO memories (content, tags, project, created_at, updated_at, type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params!["test content", "test", "testproj", "2024-01-01", "2024-01-01", "note"],
            )
            .unwrap();

        let rows: Vec<(i64, String)> = storage
            .query_map(
                "SELECT id, content FROM memories WHERE project = ?1",
                params!["testproj"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, "test content");
    }

    #[test]
    fn test_in_memory_storage_empty_query() {
        let storage = InMemoryStorage::new().unwrap();
        let rows: Vec<i64> = storage
            .query_map(
                "SELECT id FROM memories WHERE project = ?1",
                params!["nonexistent"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_in_memory_storage_last_insert_rowid() {
        let storage = InMemoryStorage::new().unwrap();
        storage
            .execute(
                "INSERT INTO memories (content, tags, project, created_at, updated_at, type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params!["test", "", "", "2024-01-01", "2024-01-01", "note"],
            )
            .unwrap();
        assert_eq!(storage.last_insert_rowid(), 1);

        storage
            .execute(
                "INSERT INTO memories (content, tags, project, created_at, updated_at, type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params!["test2", "", "", "2024-01-01", "2024-01-01", "note"],
            )
            .unwrap();
        assert_eq!(storage.last_insert_rowid(), 2);
    }
}
