use anyhow::Result;
use serde::Serialize;

use crate::core::config::GlobalConfig;
use crate::core::db::{get_db, MemoryEntry, Storage};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryListOutput {
    pub entries: Vec<MemoryEntry>,
    pub count: usize,
}

pub fn run(
    project: Option<String>,
    limit: Option<usize>,
    include_deleted: bool,
    mode: OutputMode,
) -> Result<()> {
    let config = GlobalConfig::load()?;
    let limit = limit.unwrap_or(config.memory.max_results);

    let db = get_db()?;

    let deleted_filter = if include_deleted {
        String::new()
    } else {
        " AND deleted_at IS NULL".to_string()
    };

    let cols = crate::core::db::MEMORY_SELECT_COLS;
    let entries: Vec<MemoryEntry> = if let Some(ref proj) = project {
        let sql = format!(
            "SELECT {} FROM memories WHERE project = ?1{} ORDER BY created_at DESC LIMIT ?2",
            cols, deleted_filter
        );
        db.query_map(&sql, rusqlite::params![proj, limit], MemoryEntry::from_row)?
    } else {
        let sql = format!(
            "SELECT {} FROM memories WHERE 1=1{} ORDER BY created_at DESC LIMIT ?1",
            cols, deleted_filter
        );
        db.query_map(&sql, rusqlite::params![limit], MemoryEntry::from_row)?
    };

    let output = MemoryListOutput {
        count: entries.len(),
        entries,
    };

    mode.print(&output)?;

    Ok(())
}
