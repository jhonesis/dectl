use anyhow::Result;
use ignore::WalkBuilder;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use crate::core::output::OutputMode;

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

#[derive(Debug, Clone, Serialize)]
struct FileChange {
    path: String,
    status: String,
}

pub fn run(interval: u64, mode: OutputMode) -> Result<()> {
    let mut previous = scan_files()?;

    loop {
        let current = scan_files()?;
        let changes = diff_files(&previous, &current);

        if !changes.is_empty() {
            if mode.is_json() {
                let output = serde_json::json!({
                    "changes": changes,
                    "count": changes.len(),
                });
                mode.print(&output)?;
            } else {
                let now = chrono::Local::now().format("%H:%M:%S");
                println!("\n[{}] {} change(s) detected:", now, changes.len());
                for change in &changes {
                    let icon = match change.status.as_str() {
                        "added" => "\u{2795}",
                        "modified" => "\u{270F}\u{FE0F}",
                        "removed" => "\u{2796}",
                        _ => "?",
                    };
                    println!("  {} {}", icon, change.path);
                }
            }
        }

        previous = current;
        std::thread::sleep(Duration::from_secs(interval));
    }
}

fn scan_files() -> Result<HashSet<String>> {
    let walker = WalkBuilder::new(Path::new("."))
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(is_not_ignored)
        .build();

    let mut files = HashSet::new();
    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            files.insert(entry.path().to_string_lossy().to_string());
        }
    }
    Ok(files)
}

fn diff_files(previous: &HashSet<String>, current: &HashSet<String>) -> Vec<FileChange> {
    let mut changes = Vec::new();

    for path in current.difference(previous) {
        changes.push(FileChange {
            path: path.clone(),
            status: "added".to_string(),
        });
    }

    for path in previous.difference(current) {
        changes.push(FileChange {
            path: path.clone(),
            status: "removed".to_string(),
        });
    }

    for path in previous.intersection(current) {
        let prev_meta = std::fs::metadata(path);
        let curr_meta = std::fs::metadata(path);
        if let (Ok(p), Ok(c)) = (prev_meta, curr_meta) {
            if let (Ok(pt), Ok(ct)) = (p.modified(), c.modified()) {
                if ct
                    .duration_since(pt)
                    .map(|d| d.as_secs() > 1)
                    .unwrap_or(true)
                {
                    changes.push(FileChange {
                        path: path.clone(),
                        status: "modified".to_string(),
                    });
                }
            }
        }
    }

    changes
}

fn is_not_ignored(entry: &ignore::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !IGNORED_DIRS.contains(&name.as_ref()) && !name.starts_with('.')
}
