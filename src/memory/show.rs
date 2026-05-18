use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;

use super::db::{DbConn, MemoryEntry};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryShowOutput {
    pub entry: MemoryEntry,
}

pub fn run(id: i64, mode: OutputMode) -> Result<()> {
    let db = DbConn::new()?;

    let entry = db.conn().query_row(
        "SELECT id, content, tags, project, created_at, updated_at FROM memories WHERE id = ?1 AND deleted_at IS NULL",
        rusqlite::params![id],
        MemoryEntry::from_row,
    ).context(format!("Memory entry #{} not found", id))?;

    let output = MemoryShowOutput { entry };

    match mode {
        OutputMode::Json => {
            let envelope = crate::core::output::JsonEnvelope::ok(&output);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputMode::Human => {
            println!("{}", format!("#{}", output.entry.id).bold().cyan());
            println!("Content:");
            println!("{}", format!("{}\n", output.entry.content).green());

            if !output.entry.tags.is_empty() {
                println!("Tags: {}", output.entry.tags.join(", ").cyan());
            }

            if let Some(ref p) = output.entry.project {
                println!("Project: {}", p);
            }

            println!("Created: {}", output.entry.created_at);
            println!("Updated: {}", output.entry.updated_at);
        }
    }

    Ok(())
}
