use anyhow::Result;
use colored::Colorize;
use serde::Serialize;

use super::db::DbConn;
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryDeleteOutput {
    pub id: i64,
    pub deleted_at: String,
    pub hard: bool,
}

pub fn run(id: i64, hard: bool, non_interactive: bool, mode: OutputMode) -> Result<()> {
    let db = DbConn::new()?;

    let exists = db
        .conn()
        .query_row(
            "SELECT id FROM memories WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, i64>(0),
        )
        .is_ok();

    if !exists {
        anyhow::bail!("Memory entry #{} not found", id);
    }

    let now = chrono::Utc::now().to_rfc3339();

    if hard {
        if !non_interactive {
            println!("{}", "⚠️  This will permanently delete memory #{}".yellow());
            println!("{}", "Type 'yes' to confirm:".yellow());

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "yes" {
                println!("{}", "Cancelled.".dimmed());
                return Ok(());
            }
        }

        db.conn()
            .execute("DELETE FROM memories WHERE id = ?1", rusqlite::params![id])?;

        let output = MemoryDeleteOutput {
            id,
            deleted_at: now,
            hard: true,
        };

        match mode {
            OutputMode::Json => {
                let envelope = crate::core::output::JsonEnvelope::ok(&output);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
            OutputMode::Human => {
                println!("{}", format!("Memory #{} permanently deleted.", id).green());
            }
        }
    } else {
        db.conn().execute(
            "UPDATE memories SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        )?;

        let output = MemoryDeleteOutput {
            id,
            deleted_at: now,
            hard: false,
        };

        match mode {
            OutputMode::Json => {
                let envelope = crate::core::output::JsonEnvelope::ok(&output);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            }
            OutputMode::Human => {
                println!(
                    "{}",
                    format!("Memory #{} moved to trash (soft delete).", id).green()
                );
                println!("{}", "Use --hard to permanently delete.".dimmed());
            }
        }
    }

    Ok(())
}
