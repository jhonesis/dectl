use crate::bail_app_err;
use anyhow::{Context, Result};
use serde::Serialize;
use std::io::Read;

use crate::core::db::{get_db, Storage};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryAddOutput {
    pub id: i64,
    pub content_preview: String,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub created_at: String,
}

pub fn run(
    content: Option<String>,
    tags: Option<String>,
    project: Option<String>,
    type_: String,
    mode: OutputMode,
) -> Result<()> {
    let content = match content {
        Some(c) => c,
        None => {
            let mut stdin = std::io::stdin();
            let mut input = String::new();
            stdin
                .read_to_string(&mut input)
                .context("Failed to read from stdin")?;
            if input.is_empty() {
                bail_app_err!(
                    "No content provided. Use argument or pipe content via stdin.",
                    "Use: dectl memory add \"your content\""
                );
            }
            input.trim().to_string()
        }
    };

    let tags: Vec<String> = tags
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let valid_types = crate::core::db::VALID_TYPES;
    if !valid_types.contains(&type_.as_str()) {
        bail_app_err!(
            format!(
                "Invalid type '{}'. Valid types: {}",
                type_,
                valid_types.join(", ")
            ),
            "Valid types: note, decision, research, code, session, agent, task"
        );
    }

    let db = get_db()?;
    let now = chrono::Utc::now().to_rfc3339();
    let tags_str = tags.join(",");
    let project_str = project.clone();

    db.execute(
        "INSERT INTO memories (content, tags, project, created_at, updated_at, type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![content, tags_str, project_str, now, now, type_],
    )?;

    let id = db.last_insert_rowid();

    let output = MemoryAddOutput {
        id,
        content_preview: content.chars().take(100).collect(),
        tags,
        project,
        created_at: now,
    };

    mode.print(&output)?;

    Ok(())
}
