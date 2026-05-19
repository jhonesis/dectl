use anyhow::{Context, Result};
use colored::Colorize;
use serde::Serialize;

use super::db::DbConn;
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryEditOutput {
    pub id: i64,
    pub updated_at: String,
}

fn get_editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string())
}

pub fn run(id: i64, mode: OutputMode) -> Result<()> {
    let db = DbConn::new()?;

    let (content, _tags, _project): (String, Option<String>, Option<String>) = db
        .conn()
        .query_row(
            "SELECT content, tags, project FROM memories WHERE id = ?1 AND deleted_at IS NULL",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(2)?, row.get(3)?)),
        )
        .context(format!("Memory entry #{} not found", id))?;

    let temp_content = format!(
        "# Edit memory #{}\n# Lines starting with # are comments and will be ignored\n# Empty content will cancel the edit\n\n{}",
        id, content
    );

    let temp_file = tempfile::NamedTempFile::new().context("Failed to create temporary file")?;

    std::fs::write(temp_file.path(), &temp_content).context("Failed to write to temporary file")?;

    let editor = get_editor();
    let editor_path = which::which(&editor).unwrap_or_else(|_| std::path::PathBuf::from(&editor));

    let status = std::process::Command::new(&editor_path)
        .arg(temp_file.path())
        .status()
        .context(format!("Failed to execute editor: {}", editor))?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    let edited = std::fs::read_to_string(temp_file.path()).context("Failed to read edited file")?;

    let new_content: String = edited
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    if new_content.is_empty() {
        anyhow::bail!("Empty content - edit cancelled");
    }

    if new_content == content {
        println!("{}", "No changes detected.".dimmed());
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339();

    db.conn().execute(
        "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_content, now, id],
    )?;

    let output = MemoryEditOutput {
        id,
        updated_at: now,
    };

    match mode {
        OutputMode::Json => {
            let envelope = crate::core::output::JsonEnvelope::ok(&output);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputMode::Human => {
            println!("{}", format!("Memory #{} updated.", id).green());
        }
    }

    Ok(())
}
