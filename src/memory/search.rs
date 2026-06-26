use anyhow::Result;
use serde::Serialize;

use crate::core::db::{get_db, MemoryEntry, Storage};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemorySearchOutput {
    pub query: String,
    pub entries: Vec<MemoryEntry>,
    pub count: usize,
}

pub fn run(query: String, project: Option<String>, mode: OutputMode) -> Result<()> {
    let db = get_db()?;

    let select_cols = crate::core::db::MEMORY_SELECT_COLS;

    let fts_query = query.split_whitespace().collect::<Vec<_>>().join(" AND ");
    let entries: Vec<MemoryEntry> =
        search_entries(db, select_cols, &query, &fts_query, project.as_deref());

    let count = entries.len();
    let output = MemorySearchOutput {
        query,
        entries,
        count,
    };

    mode.print(&output)?;

    Ok(())
}

fn search_entries(
    db: &impl Storage,
    cols: &str,
    raw_query: &str,
    fts_query: &str,
    project: Option<&str>,
) -> Vec<MemoryEntry> {
    if let Ok(result) = fts_search(db, cols, fts_query, project) {
        return result;
    }
    like_search(db, cols, raw_query, project)
}

fn fts_search(
    db: &impl Storage,
    cols: &str,
    fts_query: &str,
    project: Option<&str>,
) -> Result<Vec<MemoryEntry>> {
    if let Some(proj) = project {
        let sql = format!(
            "SELECT {} FROM memories m
             JOIN memories_fts fts ON m.id = fts.rowid
             WHERE m.deleted_at IS NULL AND memories_fts MATCH ?1 AND m.project = ?2
             ORDER BY rank",
            cols
        );
        db.query_map(
            &sql,
            rusqlite::params![fts_query, proj],
            MemoryEntry::from_row,
        )
    } else {
        let sql = format!(
            "SELECT {} FROM memories m
             JOIN memories_fts fts ON m.id = fts.rowid
             WHERE m.deleted_at IS NULL AND memories_fts MATCH ?1
             ORDER BY rank",
            cols
        );
        db.query_map(&sql, rusqlite::params![fts_query], MemoryEntry::from_row)
    }
}

fn like_search(
    db: &impl Storage,
    cols: &str,
    raw_query: &str,
    project: Option<&str>,
) -> Vec<MemoryEntry> {
    let search_pattern = format!("%{}%", raw_query.to_lowercase());
    if let Some(proj) = project {
        let sql = format!(
            "SELECT {} FROM memories WHERE deleted_at IS NULL AND project = ?1 AND (LOWER(content) LIKE ?2 OR LOWER(tags) LIKE ?2) ORDER BY created_at DESC",
            cols
        );
        db.query_map(
            &sql,
            rusqlite::params![proj, &search_pattern],
            MemoryEntry::from_row,
        )
        .unwrap_or_default()
    } else {
        let sql = format!(
            "SELECT {} FROM memories WHERE deleted_at IS NULL AND (LOWER(content) LIKE ?1 OR LOWER(tags) LIKE ?1) ORDER BY created_at DESC",
            cols
        );
        db.query_map(
            &sql,
            rusqlite::params![&search_pattern],
            MemoryEntry::from_row,
        )
        .unwrap_or_default()
    }
}
