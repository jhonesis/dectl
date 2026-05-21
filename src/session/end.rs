use crate::core::output::{Output, OutputMode};
use crate::session::types::SessionEndResult;
use crate::session::{decision_capture, git_sync, session_summary};
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
        struct ResultOutput {
            steps: Vec<StepOutput>,
            decisions_saved: usize,
        }
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
        println!();
    }
}

use colored::Colorize;
