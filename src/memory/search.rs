use anyhow::Result;
use colored::Colorize;
use rusqlite::Connection;
use serde::Serialize;

use super::db::{DbConn, MemoryEntry};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemorySearchOutput {
    pub query: String,
    pub entries: Vec<MemoryEntry>,
    pub count: usize,
}

pub fn run(query: String, project: Option<String>, mode: OutputMode) -> Result<()> {
    let db = DbConn::new()?;

    let select_cols = super::db::MEMORY_SELECT_COLS;

    let fts_query = query.split_whitespace().collect::<Vec<_>>().join(" AND ");
    let entries: Vec<MemoryEntry> = search_entries(
        db.conn(),
        select_cols,
        &query,
        &fts_query,
        project.as_deref(),
    );

    let count = entries.len();
    let output = MemorySearchOutput {
        query,
        entries,
        count,
    };

    match mode {
        OutputMode::Json => {
            let envelope = crate::core::output::JsonEnvelope::ok(&output);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputMode::Human => {
            if output.count == 0 {
                println!("No results found.");
                return Ok(());
            }
            println!("Found {} result(s):\n", count);
            for entry in &output.entries {
                println!(
                    "[{}] {}",
                    entry.id,
                    entry.content.chars().take(80).collect::<String>().green()
                );
                if !entry.tags.is_empty() {
                    println!("  Tags: {}", entry.tags.join(", ").cyan());
                }
                if let Some(ref p) = entry.project {
                    println!("  Project: {}", p);
                }
                println!();
            }
        }
    }

    Ok(())
}

fn search_entries(
    conn: &Connection,
    cols: &str,
    raw_query: &str,
    fts_query: &str,
    project: Option<&str>,
) -> Vec<MemoryEntry> {
    if let Ok(result) = fts_search(conn, cols, fts_query, project) {
        return result;
    }
    like_search(conn, cols, raw_query, project)
}

fn fts_search(
    conn: &Connection,
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
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![fts_query, proj], MemoryEntry::from_row)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    } else {
        let sql = format!(
            "SELECT {} FROM memories m
             JOIN memories_fts fts ON m.id = fts.rowid
             WHERE m.deleted_at IS NULL AND memories_fts MATCH ?1
             ORDER BY rank",
            cols
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![fts_query], MemoryEntry::from_row)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

fn like_search(
    conn: &Connection,
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
        let mut stmt = conn.prepare(&sql).expect("Failed to prepare LIKE query");
        let rows = stmt
            .query_map(
                rusqlite::params![proj, &search_pattern],
                MemoryEntry::from_row,
            )
            .expect("Failed to execute LIKE query");
        rows.filter_map(|r| r.ok()).collect()
    } else {
        let sql = format!(
            "SELECT {} FROM memories WHERE deleted_at IS NULL AND (LOWER(content) LIKE ?1 OR LOWER(tags) LIKE ?1) ORDER BY created_at DESC",
            cols
        );
        let mut stmt = conn.prepare(&sql).expect("Failed to prepare LIKE query");
        let rows = stmt
            .query_map(rusqlite::params![&search_pattern], MemoryEntry::from_row)
            .expect("Failed to execute LIKE query");
        rows.filter_map(|r| r.ok()).collect()
    }
}
