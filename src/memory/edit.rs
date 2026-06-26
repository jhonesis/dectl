use crate::bail_app_err;
use anyhow::{Context, Result};
use serde::Serialize;

use crate::core::db::{get_db, Storage};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryEditOutput {
    pub id: i64,
    pub updated_at: String,
}

fn get_editor() -> Result<String> {
    for var in &["EDITOR", "VISUAL"] {
        if let Ok(val) = std::env::var(var) {
            if !val.trim().is_empty() {
                return Ok(val);
            }
        }
    }
    for editor in &["vi", "nano", "vim"] {
        if which::which(editor).is_ok() {
            return Ok(editor.to_string());
        }
    }
    anyhow::bail!("No editor found")
}

pub fn run(id: i64, mode: OutputMode) -> Result<()> {
    let db = get_db()?;

    let (content, _tags, _project): (String, Option<String>, Option<String>) = db
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

    let editor = match get_editor() {
        Ok(e) => e,
        Err(_) => bail_app_err!("No editor found", "Install nano or set $EDITOR"),
    };
    let editor_path = which::which(&editor).unwrap_or_else(|_| std::path::PathBuf::from(&editor));

    let status = std::process::Command::new(&editor_path)
        .arg(temp_file.path())
        .status()
        .context(format!("Failed to execute editor: {}", editor))?;

    if !status.success() {
        bail_app_err!(
            "Editor exited with non-zero status",
            "Check your $EDITOR configuration"
        );
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
        bail_app_err!(
            "Empty content - edit cancelled",
            "Write content before saving"
        );
    }

    if new_content == content {
        if !mode.is_json() {
            println!("No changes detected.");
        }
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339();

    db.execute(
        "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_content, now, id],
    )?;

    let output = MemoryEditOutput {
        id,
        updated_at: now,
    };

    mode.print(&output)?;

    Ok(())
}
