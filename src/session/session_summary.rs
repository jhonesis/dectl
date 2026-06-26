use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;

use crate::session::types::SessionSummary;

pub fn generate_session_summary() -> Result<SessionSummary> {
    let mut summary = SessionSummary::default();

    let project_root = Path::new(".");

    // Read existing last_session.md for pending items and context
    let last_session_path = project_root.join(".dec/state/last_session.md");
    if last_session_path.exists() {
        let content = fs::read_to_string(&last_session_path)
            .context("Failed to read .dec/state/last_session.md")?;
        parse_pending_from_last_session(&content, &mut summary);
    }

    // Extract git information if in a git repo
    if crate::core::git::is_git_repo() {
        extract_git_actions(project_root, &mut summary)?;
    }

    // If no actions were found, add a default placeholder
    if summary.actions.is_empty() {
        summary
            .actions
            .push("No changes detected since last session".to_string());
    }

    // If no next step, derive from pending items
    if summary.next_step.is_empty() && !summary.pending.is_empty() {
        summary.next_step = format!("Continue with: {}", summary.pending[0]);
    }

    Ok(summary)
}

pub fn write_last_session(summary: &SessionSummary, dry_run: bool) -> Result<()> {
    let formatted = format_summary(summary);

    if dry_run {
        println!("{}", formatted);
        return Ok(());
    }

    let state_dir = Path::new(".dec/state");
    if !state_dir.exists() {
        fs::create_dir_all(state_dir).context("Failed to create .dec/state/ directory")?;
    }

    let output_path = state_dir.join("last_session.md");
    fs::write(&output_path, &formatted)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    Ok(())
}

fn format_summary(summary: &SessionSummary) -> String {
    let mut output = String::new();

    // Header with timestamp
    let now = Utc::now();
    output.push_str(&format!(
        "# Session End — {}\n",
        now.format("%Y-%m-%d %H:%M")
    ));
    output.push('\n');

    // Actions
    output.push_str("## Qué se hizo\n");
    if summary.actions.is_empty() {
        output.push_str("- No se detectaron cambios\n");
    } else {
        for action in &summary.actions {
            output.push_str(&format!("- {}\n", action));
        }
    }
    output.push('\n');

    // Pending
    output.push_str("## Qué quedó pendiente\n");
    if summary.pending.is_empty() {
        output.push_str("- Nada pendiente\n");
    } else {
        for item in &summary.pending {
            output.push_str(&format!("- {}\n", item));
        }
    }
    output.push('\n');

    // Decisions
    output.push_str("## Decisiones tomadas\n");
    if summary.decisions.is_empty() {
        output.push_str("- Ninguna\n");
    } else {
        for decision in &summary.decisions {
            output.push_str(&format!("- {}\n", decision));
        }
    }
    output.push('\n');

    // Next step
    output.push_str("## Próximo paso recomendado\n");
    if summary.next_step.is_empty() {
        output.push_str("- Revisar el estado del proyecto\n");
    } else {
        output.push_str(&format!("- {}\n", summary.next_step));
    }

    output
}

fn extract_git_actions(_project_root: &Path, summary: &mut SessionSummary) -> Result<()> {
    use crate::core::git;

    for commit in git::recent_commits(20)? {
        summary.actions.push(commit.message);
    }

    let modified = git::diff_since("HEAD~10")?;
    if !modified.is_empty() {
        let file_summary = format!(
            "{} archivo(s) modificado(s): {}",
            modified.len(),
            modified
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
        if modified.len() > 5 {
            summary
                .decisions
                .push(format!("{} (y {} más)", file_summary, modified.len() - 5));
        } else {
            summary.decisions.push(file_summary);
        }
    }

    Ok(())
}

fn parse_pending_from_last_session(content: &str, summary: &mut SessionSummary) {
    let mut in_pending = false;
    let mut in_next_step = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect section headers
        if trimmed.starts_with("## Qué quedó pendiente") {
            in_pending = true;
            in_next_step = false;
            continue;
        } else if trimmed.starts_with("## Próximo paso recomendado") {
            in_pending = false;
            in_next_step = true;
            continue;
        } else if trimmed.starts_with("## ") {
            in_pending = false;
            in_next_step = false;
            continue;
        }

        // Parse bullet points
        if in_pending {
            if let Some(item) = trimmed.strip_prefix("- ") {
                if !item.is_empty() && item != "Nada pendiente" {
                    summary.pending.push(item.to_string());
                }
            }
        }

        if in_next_step {
            if let Some(item) = trimmed.strip_prefix("- ") {
                if !item.is_empty() && item != "Revisar el estado del proyecto" {
                    summary.next_step = item.to_string();
                }
            }
        }
    }
}
