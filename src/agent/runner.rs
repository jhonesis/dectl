use crate::agent::schema::{AgentDef, AgentResult, AgentRunStatus};
use crate::bail_app_err;
use crate::core::db::Storage;
use crate::workflow::runner::Runner;
use crate::workflow::schema::{StepType, Workflow};
use anyhow::Result;
use std::collections::HashMap;

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

    let timeout = timeout_secs.unwrap_or(300);

    match crate::core::threadpool::with_timeout(
        move || {
            execute_agent_inner(
                &agent_def,
                &task_clone,
                &vars,
                file_context.as_deref(),
                dry_run,
                non_interactive,
                auto,
                &mode,
                called_from_workflow,
            )
        },
        timeout,
    ) {
        Ok(agent_result) => Ok(agent_result),
        Err(crate::core::threadpool::PoolError::Timeout { timeout_secs: secs }) => {
            let error_msg = format!("Agent '{}' timed out after {}s", agent_name, secs);
            if let Ok(db) = crate::core::db::get_db() {
                let _ = crate::agent::log::record_agent_execution(
                    db,
                    &agent_name,
                    &task,
                    "timeout",
                    0,
                    (secs * 1000) as i64,
                    Some(&error_msg),
                );
            }
            Ok(AgentResult {
                agent_type: agent_name,
                status: AgentRunStatus::Timeout,
                steps_executed: 0,
                log_id: None,
            })
        }
        Err(crate::core::threadpool::PoolError::Execute(msg)) => Ok(AgentResult {
            agent_type: agent_name,
            status: AgentRunStatus::Error { message: msg },
            steps_executed: 0,
            log_id: None,
        }),
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
    let db = crate::core::db::get_db().ok();

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
                bail_app_err!(
                    format!("Agent '{}' is not trusted for this project.\nRun without --non-interactive to trust interactively, or use:\ndectl agent trust {} --project .", agent_def.name, agent_def.name),
                    "Check the agent definition with `dectl agent describe <type>`"
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
                    bail_app_err!(
                        format!("Agent '{}' contains action steps that are not trusted.\nRun interactively to trust, or edit ~/.dectl/trust.toml manually.", agent_def.name),
                        "Check the command output above for details"
                    );
                }
            }
            crate::workflow::trust::TrustDecision::Trusted => {}
        }
    }

    let resolved = Runner::resolve_inputs(&workflow, &all_vars)?;
    let mut merged = all_vars.clone();
    merged.extend(resolved);

    let execution_result =
        Runner::execute(&workflow, &mut merged, dry_run, None, auto, mode, false)?;
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

    let log_id = match db {
        Some(c) => crate::agent::log::record_agent_execution(
            c,
            &agent_def.name,
            task,
            status_str,
            execution_result.steps_executed,
            duration_ms,
            error_msg_owned.as_deref(),
        )?,
        None => 0,
    };

    if agent_ok && !dry_run {
        if let Some(c) = db {
            let mem_type = if agent_def.name.to_lowercase() == "researcher" {
                "research"
            } else {
                "note"
            };
            let summary = format!(
                "[Agent: {}] Task: {}. Executed {} steps in {}ms. {}",
                agent_def.name, task, execution_result.steps_executed, duration_ms, agent_def.role,
            );
            let now = chrono::Utc::now().to_rfc3339();
            let tags = format!("agent,{}", agent_def.name);
            if c.execute(
                "INSERT INTO memories (content, tags, project, created_at, updated_at, type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![summary, tags, Option::<&str>::None, now, now, mem_type],
            )
            .is_ok()
            {
                let memory_id = c.last_insert_rowid();
                let _ = c.execute(
                    "INSERT INTO agent_outputs (agent_type, task_id, task_description, memory_id, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        agent_def.name,
                        vars.get("task_id").map(|s| s.as_str()),
                        task,
                        memory_id,
                        now,
                    ],
                );
            }
        }
    }

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
    use crate::workflow::schema::StepType;

    fn make_agent(name: &str, step_type: StepType) -> AgentDef {
        AgentDef {
            name: name.to_string(),
            role: "test role".to_string(),
            description: "test".to_string(),
            requires: vec![],
            context_files: vec![],
            inputs: vec![],
            next_step_hint: None,
            steps: vec![crate::workflow::schema::Step {
                step_type,
                description: "test step".to_string(),
                content: Some("hello".to_string()),
                cmd: None,
                path: Some("/tmp/dectl-test-write.txt".to_string()),
                agent_type: None,
                agent_types: None,
                parallel: None,
                shell: None,
                task: None,
                run_always: None,
                skip_if: None,
                timeout_secs: None,
            }],
        }
    }

    fn make_vars(task: &str, task_id: &str) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("task".to_string(), task.to_string());
        vars.insert("task_id".to_string(), task_id.to_string());
        vars
    }

    #[test]
    fn test_agent_auto_insert_memory_research() {
        let agent = make_agent("researcher", StepType::Write);
        let vars = make_vars("test research", "T-UT-001");
        let result = execute_agent_inner(
            &agent,
            "test research",
            &vars,
            None,
            false,
            true,
            true,
            &crate::core::output::OutputMode::Human,
            true,
        );
        assert!(result.is_ok(), "agent execution failed: {:?}", result.err());

        let db = crate::core::db::get_db().expect("failed to open db");
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE content LIKE ?1 AND type = 'research'",
                rusqlite::params!["[Agent: researcher]%"],
                |row| row.get(0),
            )
            .unwrap_or(0);
        assert!(
            count > 0,
            "Expected at least one memory entry with type='research' for researcher agent"
        );
    }

    #[test]
    fn test_agent_auto_insert_memory_note() {
        let agent = make_agent("coder", StepType::Write);
        let vars = make_vars("test coding", "T-UT-002");
        let result = execute_agent_inner(
            &agent,
            "test coding",
            &vars,
            None,
            false,
            true,
            true,
            &crate::core::output::OutputMode::Human,
            true,
        );
        assert!(result.is_ok(), "agent execution failed: {:?}", result.err());

        let db = crate::core::db::get_db().expect("failed to open db");
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE content LIKE ?1 AND type = 'note'",
                rusqlite::params!["[Agent: coder]%"],
                |row| row.get(0),
            )
            .unwrap_or(0);
        assert!(
            count > 0,
            "Expected at least one memory entry with type='note' for coder agent"
        );
    }

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
