use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::core::db::{get_db, Storage};
use crate::core::output::OutputMode;

#[derive(Debug, Clone, serde::Deserialize)]
struct ImportEntry {
    content: String,
    #[serde(default)]
    tags: Option<serde_json::Value>,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(alias = "type")]
    r#type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MemoryImportOutput {
    pub path: String,
    pub imported: usize,
    pub skipped: usize,
}

pub fn run(path: PathBuf, mode: OutputMode) -> Result<()> {
    let db = get_db()?;
    let content = std::fs::read_to_string(&path)?;

    let entries: Vec<ImportEntry> = if path.extension().is_some_and(|e| e == "jsonl") {
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        serde_json::from_str(&content)?
    };

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let now = chrono::Utc::now().to_rfc3339();

    for entry in &entries {
        let tags_str = match &entry.tags {
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(","),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let project = entry.project.clone();
        let created_at = entry.created_at.clone().unwrap_or_else(|| now.clone());
        let updated_at = entry.updated_at.clone().unwrap_or_else(|| now.clone());
        let type_ = entry.r#type.clone().unwrap_or_else(|| "note".to_string());

        let exists: bool = db
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE content = ?1 AND created_at = ?2",
                rusqlite::params![&entry.content, &created_at],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if exists {
            skipped += 1;
            continue;
        }

        db.execute(
            "INSERT INTO memories (content, tags, project, created_at, updated_at, type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![&entry.content, &tags_str, &project, &created_at, &updated_at, &type_],
        )?;
        imported += 1;
    }

    let output = MemoryImportOutput {
        path: path.display().to_string(),
        imported,
        skipped,
    };

    mode.print(&output)?;

    Ok(())
}
