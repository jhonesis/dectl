use anyhow::Result;
use ignore::{DirEntry, WalkBuilder};
use indicatif::ProgressStyle;
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

    let _spinner = if !mode.is_json() && is_terminal::is_terminal(std::io::stdout()) {
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().tick_strings(&[
            "▹▹▹▹▹",
            "▸▹▹▹▹",
            "▹▸▹▹▹",
            "▹▹▸▹▹",
            "▹▹▹▸▹",
            "▹▹▹▹▸",
            "▪▪▪▪▪",
        ]));
        pb.set_message("Scanning project files...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

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

    if let Some(spinner) = &_spinner {
        spinner.finish_and_clear();
    }

    let paths: Vec<String> = entries
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    let result = serde_json::json!({
        "files": paths,
        "count": file_count
    });
    mode.print(&result)?;

    Ok(())
}

fn is_not_ignored(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !IGNORED_DIRS.contains(&name.as_ref()) && !name.starts_with('.')
}
