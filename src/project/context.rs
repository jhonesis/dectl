use anyhow::{Context, Result};
use indicatif::ProgressStyle;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::core::output::OutputMode;

const DEFAULT_MAX_TOKENS: usize = 4000;

struct SectionDef {
    rel_path: &'static str,
    _label: &'static str,
    weight: f64,
}

const PRIORITY_SECTIONS: &[SectionDef] = &[
    SectionDef {
        rel_path: "isa/project.isa.md",
        _label: "Project Identity",
        weight: 0.20,
    },
    SectionDef {
        rel_path: "config/project.toml",
        _label: "Configuration",
        weight: 0.15,
    },
    SectionDef {
        rel_path: "state/last_session.md",
        _label: "Last Session",
        weight: 0.25,
    },
    SectionDef {
        rel_path: "state/progress.json",
        _label: "Progress",
        weight: 0.10,
    },
    SectionDef {
        rel_path: "prompts/system/integration.md",
        _label: "Integration Prompts",
        weight: 0.05,
    },
];

const DECISIONS_WEIGHT: f64 = 0.25;

const MAX_REDISTRIBUTION_ITERATIONS: usize = 5;
const SURPLUS_THRESHOLD: f64 = 0.01;

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

#[derive(Debug, Serialize)]
pub struct CompactOutput {
    pub project: String,
    pub stack: String,
    pub last_session: String,
    pub progress: String,
    pub decisions: String,
    pub memory_hits: usize,
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

fn read_file(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn count_tokens(text: &str) -> usize {
    let words = text.split_whitespace().count();
    ((words as f64) * 1.3) as usize
}

fn extract_stack(content: &str) -> String {
    let mut parts = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("stack =") {
            parts.push(value.trim().trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("technologies =") {
            parts.push(value.trim().trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("language =") {
            parts.push(value.trim().trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("framework =") {
            parts.push(value.trim().trim_matches('"').to_string());
        }
    }
    if parts.is_empty() {
        "Not detected".to_string()
    } else {
        parts.join(", ")
    }
}

fn extract_session_summary(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('>') {
            return trimmed.to_string();
        }
    }
    "No session data".to_string()
}

fn extract_progress(content: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(done) = json.get("completed") {
            if let Some(total) = json.get("total") {
                return format!("{}/{} features complete", done, total);
            }
        }
        return format!("{} fields", json.as_object().map(|o| o.len()).unwrap_or(0));
    }
    "No progress data".to_string()
}

fn extract_decisions(dec_dir: &Path) -> String {
    let decisions_dir = dec_dir.join("decisions");
    if !decisions_dir.exists() {
        return "None".to_string();
    }
    let mut names: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&decisions_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                if name.ends_with(".md") {
                    names.push(name.trim_end_matches(".md").to_string());
                }
            }
        }
    }
    names.sort();
    if names.is_empty() {
        "None".to_string()
    } else {
        names.join(", ")
    }
}

fn parse_session_date(content: &str) -> Option<chrono::NaiveDate> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(date_str) = trimmed.strip_prefix("**Fecha**: ") {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d") {
                return Some(date);
            }
        } else if let Some(date_str) = trimmed.strip_prefix("**Fecha**:") {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d") {
                return Some(date);
            }
        }
    }
    None
}

fn count_memory_hits() -> usize {
    let mem_path = dirs::home_dir()
        .map(|h| h.join(".dectl").join("memory.db"))
        .unwrap_or_default();
    if !mem_path.exists() {
        return 0;
    }
    match rusqlite::Connection::open(&mem_path) {
        Ok(conn) => conn
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE deleted_at IS NULL",
                [],
                |row| row.get::<_, usize>(0),
            )
            .unwrap_or(0),
        Err(_) => 0,
    }
}

fn truncate_to_budget(text: &str, budget: usize) -> String {
    let current_tokens = count_tokens(text);
    if current_tokens <= budget {
        return text.to_string();
    }
    let estimated_chars = (budget as f64 * 4.5) as usize;
    let truncated: String = text.chars().take(estimated_chars).collect();
    let trimmed = truncated.trim_end();
    if trimmed.is_empty() {
        return format!("[~0 tokens de {}]\n", current_tokens);
    }
    format!(
        "{}\n[~{} tokens mostrados de {}]\n",
        trimmed, budget, current_tokens
    )
}

fn calculate_budgets(
    files_data: &[FileContent],
    max_tokens: usize,
    section_weights: &[f64],
    weight_multipliers: Option<&[f64]>,
) -> Vec<usize> {
    if files_data.is_empty() {
        return Vec::new();
    }

    let total_tokens: usize = files_data.iter().map(|f| count_tokens(&f.content)).sum();
    if total_tokens <= max_tokens {
        return files_data
            .iter()
            .map(|f| count_tokens(&f.content))
            .collect();
    }

    let n_priority = PRIORITY_SECTIONS.len();
    let effective_weights: Vec<f64> = section_weights
        .iter()
        .enumerate()
        .map(|(i, &w)| {
            if let Some(multipliers) = weight_multipliers {
                if i < n_priority {
                    w * multipliers[i]
                } else {
                    w
                }
            } else {
                w
            }
        })
        .collect();

    let total_weight: f64 = effective_weights.iter().sum();

    let mut budgets: Vec<usize> = Vec::with_capacity(effective_weights.len());
    for &weight in &effective_weights {
        let raw_budget = (weight / total_weight * max_tokens as f64) as usize;
        budgets.push(raw_budget.max(1));
    }

    let mut current_budgets = budgets;
    for _iteration in 0..MAX_REDISTRIBUTION_ITERATIONS {
        let mut surplus = 0usize;
        let mut total_adjusted_weight = 0.0f64;

        for (i, file) in files_data.iter().enumerate() {
            let file_tokens = count_tokens(&file.content);
            if file_tokens <= current_budgets[i] {
                surplus += current_budgets[i] - file_tokens;
                current_budgets[i] = file_tokens;
            } else {
                total_adjusted_weight += effective_weights[i];
            }
        }

        if surplus == 0 || (surplus as f64 / max_tokens as f64) < SURPLUS_THRESHOLD {
            break;
        }

        if total_adjusted_weight == 0.0 {
            break;
        }

        for (i, file) in files_data.iter().enumerate() {
            let file_tokens = count_tokens(&file.content);
            if file_tokens > current_budgets[i] {
                let extra =
                    (surplus as f64 * effective_weights[i] / total_adjusted_weight) as usize;
                current_budgets[i] += extra;
            }
        }
    }

    current_budgets
}

pub fn run(max_tokens: Option<usize>, format: String, mode: OutputMode) -> Result<()> {
    let dec_dir = find_dec_dir().context(".dec/ directory not found")?;

    let project_name = dec_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let max_tokens = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS).max(1);

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
        pb.set_message("Scanning project context...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    let mut files_data: Vec<FileContent> = Vec::new();
    let mut section_weights: Vec<f64> = Vec::new();

    for section in PRIORITY_SECTIONS {
        let full_path = dec_dir.join(section.rel_path);
        if let Some(content) = read_file(&full_path) {
            files_data.push(FileContent {
                path: section.rel_path.to_string(),
                content,
                tokens: 0,
            });
            section_weights.push(section.weight);
        }
    }

    const MAX_DECISIONS: usize = 3;
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
            let n_decisions = recent_decisions.len().min(MAX_DECISIONS);
            if n_decisions > 0 {
                let decision_weight = DECISIONS_WEIGHT / n_decisions as f64;
                for (name, content) in recent_decisions.iter().take(n_decisions) {
                    files_data.push(FileContent {
                        path: format!("decisions/{}", name),
                        content: content.clone(),
                        tokens: 0,
                    });
                    section_weights.push(decision_weight);
                }
            }
        }
    }

    let weight_multipliers =
        if let Some(session_content) = read_file(&dec_dir.join("state/last_session.md")) {
            if let Some(session_date) = parse_session_date(&session_content) {
                let mut multipliers = Vec::with_capacity(PRIORITY_SECTIONS.len());
                for section in PRIORITY_SECTIONS {
                    let full_path = dec_dir.join(section.rel_path);
                    let mtime = std::fs::metadata(&full_path).and_then(|m| m.modified());
                    let naive_dt = session_date.and_hms_opt(0, 0, 0).unwrap();
                    let session_system: std::time::SystemTime = naive_dt.and_utc().into();
                    let is_changed = mtime.map(|m| m > session_system).unwrap_or(false);
                    multipliers.push(if is_changed { 2.0 } else { 0.5 });
                }
                Some(multipliers)
            } else {
                None
            }
        } else {
            None
        };

    let budgets = calculate_budgets(
        &files_data,
        max_tokens,
        &section_weights,
        weight_multipliers.as_deref(),
    );

    let mut truncated_content = String::new();
    for (i, file) in files_data.iter_mut().enumerate() {
        let truncated = truncate_to_budget(&file.content, budgets[i]);
        file.content = truncated;
        let used = count_tokens(&file.content);
        file.tokens = used;
        truncated_content.push_str(&format!("\n## {}\n\n{}\n", file.path, file.content));
    }

    if let Some(spinner) = &_spinner {
        spinner.finish_and_clear();
    }

    let final_tokens = count_tokens(&truncated_content);

    if format == "compact" {
        let project_toml = read_file(&dec_dir.join("config/project.toml")).unwrap_or_default();
        let last_session = read_file(&dec_dir.join("state/last_session.md")).unwrap_or_default();
        let progress_json = read_file(&dec_dir.join("state/progress.json")).unwrap_or_default();

        let compact = CompactOutput {
            project: project_name,
            stack: extract_stack(&project_toml),
            last_session: extract_session_summary(&last_session),
            progress: extract_progress(&progress_json),
            decisions: extract_decisions(&dec_dir),
            memory_hits: count_memory_hits(),
        };

        mode.print(&compact)?;
        return Ok(());
    }

    if format == "json" {
        let output = ContextJsonOutput {
            project: project_name,
            tokens_used: final_tokens,
            tokens_limit: max_tokens,
            files: files_data,
        };
        mode.print(&output)?;
    } else {
        let output = ContextOutput {
            project: project_name,
            tokens_used: final_tokens,
            tokens_limit: max_tokens,
            content: truncated_content,
        };
        println!("{}", output.content);
        println!("\n---\ntokens: {}/{}", final_tokens, max_tokens);
    }

    Ok(())
}
