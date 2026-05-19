use anyhow::Result;
use colored::Colorize;
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

    let search_pattern = format!("%{}%", query.to_lowercase());

    let base_query = "SELECT id, content, tags, project, created_at, updated_at FROM memories WHERE deleted_at IS NULL";

    let entries: Vec<MemoryEntry> = if let Some(ref proj) = project {
        let full_query = format!(
            "{} AND project = ?1 AND (LOWER(content) LIKE ?2 OR LOWER(tags) LIKE ?2)",
            base_query
        );
        let mut stmt = db.conn().prepare(&full_query)?;
        let rows = stmt.query_map(
            rusqlite::params![proj, &search_pattern],
            MemoryEntry::from_row,
        )?;
        rows.filter_map(|r| r.ok()).collect()
    } else {
        let full_query = format!(
            "{} AND (LOWER(content) LIKE ?1 OR LOWER(tags) LIKE ?1)",
            base_query
        );
        let mut stmt = db.conn().prepare(&full_query)?;
        let rows = stmt.query_map(rusqlite::params![&search_pattern], MemoryEntry::from_row)?;
        rows.filter_map(|r| r.ok()).collect()
    };

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