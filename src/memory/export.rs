use anyhow::Result;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;

use crate::core::db::{get_db, MemoryEntry, Storage};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryExportOutput {
    pub path: String,
    pub count: usize,
    pub format: String,
}

pub fn run(path: PathBuf, format: String, mode: OutputMode) -> Result<()> {
    let db = get_db()?;

    let entries: Vec<MemoryEntry> = db.query_map(
        "SELECT id, content, tags, project, created_at, updated_at, type FROM memories ORDER BY created_at",
        &[],
        MemoryEntry::from_row,
    )?;

    match format.as_str() {
        "jsonl" => {
            let mut file = std::fs::File::create(&path)?;
            for entry in &entries {
                let line = serde_json::to_string(entry)?;
                writeln!(file, "{}", line)?;
            }
        }
        _ => {
            let json = serde_json::to_string_pretty(&entries)?;
            std::fs::write(&path, json)?;
        }
    }

    let output = MemoryExportOutput {
        path: path.display().to_string(),
        count: entries.len(),
        format,
    };

    mode.print(&output)?;

    Ok(())
}
