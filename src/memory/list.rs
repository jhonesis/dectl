use anyhow::Result;
use colored::Colorize;
use serde::Serialize;

use super::db::{DbConn, MemoryEntry};
use crate::core::config::GlobalConfig;
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryListOutput {
    pub entries: Vec<MemoryEntry>,
    pub count: usize,
}

pub fn run(project: Option<String>, limit: Option<usize>, mode: OutputMode) -> Result<()> {
    let config = GlobalConfig::load()?;
    let limit = limit.unwrap_or(config.memory.max_results);

    let db = DbConn::new()?;

    let cols = super::db::MEMORY_SELECT_COLS;
    let query = if project.is_some() {
        format!("SELECT {} FROM memories WHERE project = ?1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT ?2", cols)
    } else {
        format!(
            "SELECT {} FROM memories WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT ?1",
            cols
        )
    };

    let entries: Vec<MemoryEntry> = if let Some(ref proj) = project {
        let mut stmt = db.conn().prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params![proj, limit], MemoryEntry::from_row)?;
        rows.filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = db.conn().prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params![limit], MemoryEntry::from_row)?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let output = MemoryListOutput {
        count: entries.len(),
        entries,
    };

    match mode {
        OutputMode::Json => {
            let envelope = crate::core::output::JsonEnvelope::ok(&output);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputMode::Human => {
            if output.entries.is_empty() {
                println!("No memory entries found.");
                return Ok(());
            }
            for entry in &output.entries {
                println!(
                    "[{}] {} ({})",
                    entry.id,
                    entry.content.chars().take(60).collect::<String>().green(),
                    entry.type_.dimmed()
                );
                if !entry.tags.is_empty() {
                    println!("  Tags: {}", entry.tags.join(", ").cyan());
                }
                if let Some(ref p) = entry.project {
                    println!("  Project: {}", p);
                }
                println!("  Created: {}", entry.created_at);
                println!();
            }
            println!("Total: {} entries", output.count);
        }
    }

    Ok(())
}
