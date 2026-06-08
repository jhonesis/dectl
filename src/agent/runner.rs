use crate::agent::schema::{AgentDef, AgentResult, AgentRunStatus};
use crate::workflow::runner::Runner;
use crate::workflow::schema::{StepType, Workflow};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub fn run_agent(
    agent_def: &AgentDef,
    task: &str,
    vars: &HashMap<String, String>,
    file_context: Option<&str>,
    dry_run: bool,
    timeout_secs: Option<u64>,
    non_interactive: bool,
    mode: &crate::core::output::OutputMode,
    auto: bool,
    called_from_workflow: bool,
) -> Result<AgentResult> {
    let agent_def = agent_def.clone();
    let agent_name = agent_def.name.clone();
    let task = task.to_string();
    let task_clone = task.clone();
    let vars = vars.clone();
    let file_context = file_context.map(|s| s.to_string());
    let mode = *mode;

    let (tx, rx) = mpsc::channel();
    let timeout = timeout_secs.unwrap_or(300);

    std::thread::spawn(move || {
        let result = execute_agent_inner(
            &agent_def,
            &task_clone,
            &vars,
            file_context.as_deref(),
            dry_run,
            non_interactive,
            auto,
            &mode,
            called_from_workflow,
        );
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(timeout)) {
        Ok(Ok(agent_result)) => Ok(agent_result),
        Ok(Err(e)) => Ok(AgentResult {
            agent_type: agent_name,
            status: AgentRunStatus::Error {
                message: e.to_string(),
            },
            steps_executed: 0,
            log_id: None,
        }),
        Err(_) => {
            let error_msg = format!("Agent '{}' timed out after {}s", agent_name, timeout);
            let _ = crate::agent::log::record_agent_execution(
                &agent_name,
                &task,
                "timeout",
                0,
                (timeout * 1000) as i64,
                Some(&error_msg),
            );
            Ok(AgentResult {
                agent_type: agent_name,
                status: AgentRunStatus::Timeout,
                steps_executed: 0,
                log_id: None,
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_agent_inner(
    agent_def: &AgentDef,
    task: &str,
    vars: &HashMap<String, String>,
    file_context: Option<&str>,
    dry_run: bool,
    non_interactive: bool,
    auto: bool,
    mode: &crate::core::output::OutputMode,
    called_from_workflow: bool,
) -> Result<AgentResult> {
    let start = std::time::Instant::now();

    let workflow = Workflow {
        name: agent_def.name.clone(),
        description: agent_def.description.clone(),
        inputs: agent_def.inputs.clone(),
        steps: agent_def.steps.clone(),
    };

    let mut all_vars: HashMap<String, String> = HashMap::new();
    all_vars.insert("task".to_string(), task.to_string());
    if let Some(file_path) = file_context {
        all_vars.insert("file".to_string(), file_path.to_string());
    }
    for (k, v) in vars {
        all_vars.insert(k.clone(), v.clone());
    }

    // Load context files automatically
    for context_file in &agent_def.context_files {
        if let Ok(content) = std::fs::read_to_string(context_file) {
            let var_name = normalize_context_filename(context_file);
            all_vars.insert(format!("context_{}", var_name), content);
        }
    }

    // When called from a workflow step, trust is checked at the workflow level
    // (src/workflow/run.rs). Skip the per-agent trust check to avoid redundant prompts.
    let has_action = agent_def
        .steps
        .iter()
        .any(|s| s.step_type == StepType::Action);
    if has_action && !dry_run && !auto && !called_from_workflow {
        let project_path = std::env::current_dir()
            .and_then(|p| std::fs::canonicalize(&p))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let trust_decision = crate::workflow::trust::check_trust(
            &project_path,
            &agent_def.name,
            true,
            non_interactive,
        )?;
        match trust_decision {
            crate::workflow::trust::TrustDecision::RequiresConfirmation => {
                anyhow::bail!(
                    "Agent '{}' is not trusted for this project.\n\
                     Run without --non-interactive to trust interactively, or use:\n\
                     dectl agent trust {} --project .",
                    agent_def.name,
                    agent_def.name
                );
            }
            crate::workflow::trust::TrustDecision::AskUser => {
                if !mode.is_json() {
                    println!("Agent '{}' contains action steps:", agent_def.name);
                    for step in &agent_def.steps {
                        if step.step_type == StepType::Action {
                            if let Some(cmd) = &step.cmd {
                                println!("  - {}", cmd.join(" "));
                            }
                        }
                    }
                    print!("\nDo you trust this agent? (y/N): ");
                    std::io::Write::flush(&mut std::io::stdout())?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if input.trim().to_lowercase() != "y" {
                        anyhow::bail!("Agent execution aborted by user.");
                    }
                    crate::workflow::trust::grant_trust(&project_path, &agent_def.name)?;
                    if !mode.is_json() {
                        println!("Trusted. This agent is now trusted for this project.");
                    }
                } else {
                    anyhow::bail!(
                        "Agent '{}' contains action steps that are not trusted.\n\
                         Run interactively to trust, or edit ~/.dectl/trust.toml manually.",
                        agent_def.name
                    );
                }
            }
            crate::workflow::trust::TrustDecision::Trusted => {}
        }
    }

    let resolved = Runner::resolve_inputs(&workflow, &all_vars)?;
    let mut merged = all_vars.clone();
    merged.extend(resolved);

    let execution_result = Runner::execute(&workflow, &mut merged, dry_run, None, auto, mode, false)?;
    let duration_ms = start.elapsed().as_millis() as i64;

    let agent_ok = if !execution_result.success {
        let any_write_ok = execution_result
            .results
            .iter()
            .any(|r| r.success && r.step_type == "write");
        if any_write_ok {
            for failed in execution_result.results.iter().filter(|r| !r.success) {
                eprintln!(
                    "  ⚠ Non-critical step {} ({}) failed — main output was produced",
                    failed.step_num, failed.step_type
                );
            }
            true
        } else {
            false
        }
    } else {
        true
    };

    let error_msg_owned: Option<String> = if agent_ok {
        None
    } else {
        execution_result
            .results
            .iter()
            .find(|r| !r.success)
            .map(|r| format!("Step {} ({}) failed", r.step_num, r.step_type))
    };

    let status_str = if error_msg_owned.is_some() {
        "error"
    } else {
        "ok"
    };

    let log_id = crate::agent::log::record_agent_execution(
        &agent_def.name,
        task,
        status_str,
        execution_result.steps_executed,
        duration_ms,
        error_msg_owned.as_deref(),
    )?;

    Ok(AgentResult {
        agent_type: agent_def.name.clone(),
        status: if agent_ok {
            AgentRunStatus::Ok
        } else {
            AgentRunStatus::Error {
                message: error_msg_owned.unwrap_or_default(),
            }
        },
        steps_executed: execution_result.steps_executed,
        log_id: Some(log_id),
    })
}

/// Normalize a context file path to a valid variable name.
/// Examples:
///   ".dec/config/project.toml" → "dec_config_project_toml"
///   ".dec/isa/project.isa.md" → "dec_isa_project_isa_md"
fn normalize_context_filename(path: &str) -> String {
    path.chars()
        .map(|c| match c {
            '.' | '/' | '-' => '_',
            c => c,
        })
        .collect::<String>()
        .trim_start_matches('_')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_context_filename_basic() {
        assert_eq!(
            normalize_context_filename(".dec/config/project.toml"),
            "dec_config_project_toml"
        );
    }

    #[test]
    fn test_normalize_context_filename_with_isa() {
        assert_eq!(
            normalize_context_filename(".dec/isa/project.isa.md"),
            "dec_isa_project_isa_md"
        );
    }

    #[test]
    fn test_normalize_context_filename_with_hyphen() {
        assert_eq!(
            normalize_context_filename("my-config-file.json"),
            "my_config_file_json"
        );
    }

    #[test]
    fn test_normalize_context_filename_no_leading_dot() {
        assert_eq!(
            normalize_context_filename("config/project.toml"),
            "config_project_toml"
        );
    }
}
