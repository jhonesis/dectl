use anyhow::Result;
use colored::Colorize;
use ignore::{DirEntry, WalkBuilder};
use std::path::Path;

use crate::core::output::OutputMode;

const DEFAULT_DEPTH: usize = 10;
const MAX_DISPLAY_DEPTH: usize = 20;

const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".cache",
    "__pycache__",
    ".pytest_cache",
    "vendor",
    "bin",
    "obj",
];

pub fn run(depth: Option<usize>, mode: OutputMode) -> Result<()> {
    let max_depth = depth.unwrap_or(DEFAULT_DEPTH).min(MAX_DISPLAY_DEPTH);

    let walker = WalkBuilder::new(Path::new("."))
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(is_not_ignored)
        .max_depth(Some(max_depth))
        .build();

    let mut entries: Vec<DirEntry> = walker
        .into_iter()
        .filter_map(std::result::Result::ok)
        .collect();

    entries.sort_by(|a, b| a.path().cmp(b.path()));

    let file_count = entries
        .iter()
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .count();

    match mode {
        OutputMode::Json => {
            let paths: Vec<String> = entries
                .iter()
                .map(|e| e.path().to_string_lossy().to_string())
                .collect();
            let result = serde_json::json!({
                "files": paths,
                "count": file_count
            });
            let envelope = crate::core::output::JsonEnvelope::ok(&result);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputMode::Human => {
            for entry in &entries {
                let path = entry.path();
                let depth = path.components().count().saturating_sub(1);
                let prefix = "  ".repeat(depth.min(max_depth));

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let display = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    format!("{}{}", prefix, name.blue().bold())
                } else {
                    format!("{}{}", prefix, name.green())
                };

                println!("{}", display);
            }
            println!();
            println!("Total: {} files", file_count);
        }
    }

    Ok(())
}

fn is_not_ignored(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !IGNORED_DIRS.contains(&name.as_ref()) && !name.starts_with('.')
}
