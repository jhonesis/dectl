use anyhow::{Context, Result};
use serde::Serialize;
use std::path::PathBuf;

use crate::core::output::OutputMode;

const DEFAULT_MAX_TOKENS: usize = 4000;

#[derive(Debug, Serialize)]
pub struct ContextOutput {
    pub project: String,
    pub tokens_used: usize,
    pub tokens_limit: usize,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ContextJsonOutput {
    pub project: String,
    pub tokens_used: usize,
    pub tokens_limit: usize,
    pub files: Vec<FileContent>,
}

#[derive(Debug, Serialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub tokens: usize,
}

fn find_dec_dir() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    let home = dirs::home_dir()?;

    let mut path = current.as_path();
    while path != home {
        let dec_path = path.join(".dec");
        if dec_path.exists() && dec_path.is_dir() {
            return Some(dec_path);
        }
        path = path.parent()?;
    }

    None
}

fn read_file(path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn count_tokens(text: &str) -> usize {
    let words = text.split_whitespace().count();
    ((words as f64) * 1.3) as usize
}

pub fn run(max_tokens: Option<usize>, format: String, mode: OutputMode) -> Result<()> {
    let dec_dir = find_dec_dir().context(".dec/ directory not found")?;

    let project_name = dec_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let max_tokens = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

    let priority_files = vec![
        ("isa/project.isa.md", "Project Identity"),
        ("config/project.toml", "Configuration"),
        ("state/last_session.md", "Last Session"),
        ("state/progress.json", "Progress"),
        ("prompts/system/integration.md", "Integration Prompts"),
    ];

    let mut all_content = String::new();
    let mut files_data: Vec<FileContent> = Vec::new();

    for (rel_path, label) in &priority_files {
        let full_path = dec_dir.join(rel_path);
        if let Some(content) = read_file(&full_path) {
            let tokens = count_tokens(&content);
            all_content.push_str(&format!("\n## {} ({})\n\n{}\n", label, rel_path, content));

            files_data.push(FileContent {
                path: rel_path.to_string(),
                content: content.clone(),
                tokens,
            });
        }
    }

    let decisions_dir = dec_dir.join("decisions");
    if decisions_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&decisions_dir) {
            let mut recent_decisions: Vec<(String, String)> = Vec::new();
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str() {
                    if name.ends_with(".md") {
                        if let Some(content) = read_file(&entry.path()) {
                            recent_decisions.push((name.to_string(), content));
                        }
                    }
                }
            }
            recent_decisions.sort_by(|a, b| b.0.cmp(&a.0));
            for (name, content) in recent_decisions.iter().take(3) {
                let tokens = count_tokens(content);
                all_content.push_str(&format!("\n## Decision: {}\n\n{}\n", name, content));
                files_data.push(FileContent {
                    path: format!("decisions/{}", name),
                    content: content.clone(),
                    tokens,
                });
            }
        }
    }

    let tokens_used = count_tokens(&all_content);
    let truncated = if tokens_used > max_tokens {
        let mut truncated_content = String::new();
        let mut current_tokens = 0;

        for file in &files_data {
            if current_tokens + file.tokens <= max_tokens {
                truncated_content.push_str(&format!("\n## {}\n\n{}\n", file.path, file.content));
                current_tokens += file.tokens;
            }
        }
        truncated_content
    } else {
        all_content
    };

    let final_tokens = count_tokens(&truncated);

    match (format.as_str(), mode) {
        ("json", OutputMode::Json) => {
            let output = ContextJsonOutput {
                project: project_name,
                tokens_used: final_tokens,
                tokens_limit: max_tokens,
                files: files_data,
            };
            let envelope = crate::core::output::JsonEnvelope::ok(&output);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        ("json", OutputMode::Human) => {
            let output = ContextJsonOutput {
                project: project_name,
                tokens_used: final_tokens,
                tokens_limit: max_tokens,
                files: files_data,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            let output = ContextOutput {
                project: project_name,
                tokens_used: final_tokens,
                tokens_limit: max_tokens,
                content: truncated,
            };
            println!("{}", output.content);
            println!("\n---\ntokens: {}/{}", final_tokens, max_tokens);
        }
    }

    Ok(())
}
