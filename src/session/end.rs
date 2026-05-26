use crate::core::output::{Output, OutputMode};
use crate::session::types::SessionEndResult;
use crate::session::{agent_sync, config_sync, decision_capture, git_sync, session_summary};
use anyhow::Result;

pub fn run(dry_run: bool, skip_git: bool, _non_interactive: bool, mode: OutputMode) -> Result<()> {
    let mut result = SessionEndResult::new();

    // Step 1: Update last_session.md
    match session_summary::generate_session_summary() {
        Ok(summary) => {
            match session_summary::write_last_session(&summary, dry_run) {
                Ok(()) => {
                    result.add_step("last_session.md", true, "updated");
                    if mode.is_json() {
                        // decisions_saved will be set later
                    }
                }
                Err(e) => {
                    result.add_step("last_session.md", false, &e.to_string());
                }
            }
        }
        Err(e) => {
            result.add_step(
                "last_session.md",
                false,
                &format!("failed to generate summary: {}", e),
            );
        }
    }

    // Step 2: Sync with git (skip if requested)
    if skip_git {
        result.add_step("progress.json", true, "skipped (--skip-git)");
    } else {
        match git_sync::detect_git_changes() {
            Ok(Some(changes)) => match git_sync::sync_progress(&changes, dry_run) {
                Ok(()) => {
                    result.add_step("progress.json", true, "synced with git");
                }
                Err(e) => {
                    result.add_step("progress.json", false, &e.to_string());
                }
            },
            Ok(None) => {
                result.add_step("progress.json", true, "no git repo found (skipped)");
            }
            Err(e) => {
                result.add_step("progress.json", false, &e.to_string());
            }
        }
    }

    // Step 3: Capture decisions and save to memory
    match decision_capture::capture_decisions() {
        Ok(decisions) => match decision_capture::save_decisions(&decisions) {
            Ok(count) => {
                result.decisions_saved = count;
                if count > 0 {
                    result.add_step("memory", true, &format!("{} decisions saved", count));
                } else {
                    result.add_step("memory", true, "no new decisions to save");
                }
            }
            Err(e) => {
                result.add_step("memory", false, &e.to_string());
            }
        },
        Err(e) => {
            result.add_step(
                "memory",
                false,
                &format!("failed to capture decisions: {}", e),
            );
        }
    }

    // Step 4: Sync project config (detect stack changes)
    match config_sync::sync_config(dry_run) {
        Ok(sync_result) => {
            result.config_changes = Some(sync_result.clone());
            if sync_result.toml_updated {
                let changes = &sync_result.diff;
                let mut parts = Vec::new();
                if !changes.new_languages.is_empty() {
                    parts.push(format!("{} new languages", changes.new_languages.len()));
                }
                if !changes.new_frameworks.is_empty() {
                    parts.push(format!("{} new frameworks", changes.new_frameworks.len()));
                }
                if !changes.new_tools.is_empty() {
                    parts.push(format!("{} new tools", changes.new_tools.len()));
                }
                if changes.type_changed.is_some() {
                    parts.push("project type changed".to_string());
                }
                let msg = if parts.is_empty() {
                    "config up to date".to_string()
                } else {
                    format!("project.toml updated ({})", parts.join(", "))
                };
                result.add_step("config_sync", true, &msg);
            } else if !sync_result.diff.is_empty() {
                result.add_step(
                    "config_sync",
                    true,
                    "dry-run: changes detected (not applied)",
                );
            } else {
                result.add_step("config_sync", true, "config up to date");
            }
            if !sync_result.isa_warnings.is_empty() {
                result.add_step(
                    "isa_coherence",
                    true,
                    &format!(
                        "{} warnings: project.isa.md may be outdated",
                        sync_result.isa_warnings.len()
                    ),
                );
            }
        }
        Err(e) => {
            result.add_step("config_sync", false, &e.to_string());
        }
    }

    // Step 5: Query agent sessions
    match agent_sync::query_agent_sessions() {
        Ok(count) => {
            result.agent_sessions = count;
            if count > 0 {
                result.add_step(
                    "agent_sync",
                    true,
                    &format!("{} agent sessions this cycle", count),
                );
            }
        }
        Err(e) => {
            result.add_step("agent_sync", false, &e.to_string());
        }
    }

    // Output results
    print_result(&result, mode);

    // Exit code: 0 if at least one step succeeded, 1 if all failed
    if !result.any_success() {
        anyhow::bail!("All session end steps failed");
    }

    Ok(())
}

fn print_result(result: &SessionEndResult, mode: OutputMode) {
    if mode.is_json() {
        #[derive(serde::Serialize)]
        struct StepOutput {
            name: String,
            success: bool,
            message: String,
        }
        #[derive(serde::Serialize)]
        struct ConfigDiffOutput {
            new_languages: Vec<String>,
            new_frameworks: Vec<String>,
            new_tools: Vec<String>,
            type_changed: Option<(String, String)>,
        }
        #[derive(serde::Serialize)]
        struct ConfigSyncOutput {
            toml_updated: bool,
            isa_incoherent: bool,
            isa_warnings: Vec<String>,
            diff: ConfigDiffOutput,
        }
        #[derive(serde::Serialize)]
        struct ResultOutput {
            steps: Vec<StepOutput>,
            decisions_saved: usize,
            config_changes: Option<ConfigSyncOutput>,
            agent_sessions: usize,
        }
        let config_out = result.config_changes.as_ref().map(|c| ConfigSyncOutput {
            toml_updated: c.toml_updated,
            isa_incoherent: c.isa_incoherent,
            isa_warnings: c.isa_warnings.clone(),
            diff: ConfigDiffOutput {
                new_languages: c.diff.new_languages.clone(),
                new_frameworks: c.diff.new_frameworks.clone(),
                new_tools: c.diff.new_tools.clone(),
                type_changed: c.diff.type_changed.clone(),
            },
        });
        let output = ResultOutput {
            steps: result
                .steps
                .iter()
                .map(|s| StepOutput {
                    name: s.name.clone(),
                    success: s.success,
                    message: s.message.clone(),
                })
                .collect(),
            decisions_saved: result.decisions_saved,
            config_changes: config_out,
            agent_sessions: result.agent_sessions,
        };
        Output::print(&output, mode);
    } else {
        println!("\n{}", "Session ended.".bold());
        println!();
        for step in &result.steps {
            let icon = if step.success { "✅" } else { "❌" };
            println!("{} {}: {}", icon, step.name, step.message);
        }
        if result.decisions_saved > 0 {
            println!("\n{} decisions saved to memory", result.decisions_saved);
        }
        if let Some(config) = &result.config_changes {
            if !config.isa_warnings.is_empty() {
                println!("\n{}", "ISA coherence warnings:".yellow().bold());
                for warning in &config.isa_warnings {
                    println!("  ⚠ {}", warning);
                }
            }
        }
        println!();
    }
}

use colored::Colorize;
