use crate::agent::schema::AgentRunStatus;
use crate::core::output::{Output, OutputMode};
use anyhow::Result;
use std::collections::HashMap;

fn interpolate_hint(template: &str, task: &str, vars: &HashMap<String, String>) -> String {
    let mut all_vars = vars.clone();
    all_vars.insert("task".to_string(), task.to_string());
    match crate::workflow::interpolate::interpolate(template, &all_vars) {
        Ok(s) => s,
        Err(_) => template.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    agent_type: &str,
    task: Option<&str>,
    file: Option<&str>,
    var: &[String],
    timeout: Option<u64>,
    dry_run: bool,
    parallel: bool,
    non_interactive: bool,
    mode: OutputMode,
) -> Result<()> {
    let task = match task {
        Some(t) => t.to_string(),
        None => {
            if non_interactive {
                let msg = "Task description required. Use --task \"<description>\"";
                Output::print_error(msg, None, mode);
                std::process::exit(1);
            }
            print!("Enter task description: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    let mut vars: HashMap<String, String> = HashMap::new();
    for v in var {
        if let Some((k, val)) = v.split_once('=') {
            vars.insert(k.to_string(), val.to_string());
        } else {
            let msg = format!("Invalid --var format: '{}'. Expected 'name=value'", v);
            Output::print_error(&msg, None, mode);
            std::process::exit(1);
        }
    }

    if parallel {
        return run_parallel_agents(
            agent_type,
            &task,
            &vars,
            timeout,
            dry_run,
            non_interactive,
            mode,
        );
    }

    run_single_agent(
        agent_type,
        &task,
        file,
        &vars,
        timeout,
        dry_run,
        non_interactive,
        mode,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_single_agent(
    agent_type: &str,
    task: &str,
    file: Option<&str>,
    vars: &HashMap<String, String>,
    timeout: Option<u64>,
    dry_run: bool,
    non_interactive: bool,
    mode: OutputMode,
) -> Result<()> {
    let agent = crate::agent::loader::load_agent(agent_type);
    let (agent_def, _source) = match agent {
        Some(a) => a,
        None => {
            let msg = format!(
                "Agent '{}' not found. Run 'dectl agent list' to see available agents.",
                agent_type
            );
            Output::print_error(&msg, None, mode);
            std::process::exit(1);
        }
    };

    if !mode.is_json() && !dry_run {
        println!("🔄 Running agent: {} ({})", agent_def.name, agent_def.role);
        println!("   Task: {}", task);
        if let Some(f) = file {
            println!("   File: {}", f);
        }
        println!();
    }

    let result = crate::agent::runner::run_agent(
        &agent_def,
        task,
        vars,
        file,
        dry_run,
        timeout,
        non_interactive,
        &mode,
        false,
    )?;

    if mode.is_json() {
        let json_result = match &result.status {
            AgentRunStatus::Ok => {
                serde_json::json!({
                    "status": "ok",
                    "agent": agent_def.name,
                    "task": task,
                    "steps_executed": result.steps_executed,
                    "log_id": result.log_id,
                })
            }
            AgentRunStatus::Error { message } => {
                serde_json::json!({
                    "status": "error",
                    "agent": agent_def.name,
                    "task": task,
                    "error": message,
                    "steps_executed": result.steps_executed,
                    "log_id": result.log_id,
                })
            }
            AgentRunStatus::Timeout => {
                serde_json::json!({
                    "status": "timeout",
                    "agent": agent_def.name,
                    "task": task,
                    "steps_executed": 0,
                })
            }
        };
        Output::print(&json_result, mode);
    } else {
        match &result.status {
            AgentRunStatus::Ok => {
                println!(
                    "✓ Agent '{}' completed successfully ({} steps) (log_id: {})",
                    agent_def.name,
                    result.steps_executed,
                    result.log_id.unwrap_or(0)
                );
            }
            AgentRunStatus::Error { message } => {
                println!("✗ Agent '{}' failed: {}", agent_def.name, message);
            }
            AgentRunStatus::Timeout => {
                println!("⏱ Agent '{}' timed out", agent_def.name);
            }
        }
    }

    if matches!(result.status, AgentRunStatus::Ok) {
        if let Some(hint) = &agent_def.next_step_hint {
            let msg = interpolate_hint(hint, task, vars);
            if !mode.is_json() {
                println!("\n→ {}", msg);
            }
        }
    }

    if !matches!(result.status, AgentRunStatus::Ok) {
        std::process::exit(1);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_parallel_agents(
    agent_types_str: &str,
    task: &str,
    vars: &HashMap<String, String>,
    timeout: Option<u64>,
    dry_run: bool,
    non_interactive: bool,
    mode: OutputMode,
) -> Result<()> {
    let agent_types: Vec<String> = agent_types_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if agent_types.is_empty() {
        let msg =
            "No agent types specified for parallel execution. Use --parallel reviewer,documenter";
        Output::print_error(msg, None, mode);
        std::process::exit(1);
    }

    if !mode.is_json() {
        println!("🔄 Running agents in parallel: {}", agent_types.join(", "));
        println!("   Task: {}", task);
        println!();
    }

    let results = crate::agent::parallel::run_parallel(
        &agent_types,
        task,
        vars,
        timeout,
        dry_run,
        non_interactive,
        &mode,
    )?;

    let all_ok = results
        .iter()
        .all(|r| matches!(r.status, AgentRunStatus::Ok));
    let any_error = results
        .iter()
        .any(|r| matches!(r.status, AgentRunStatus::Error { .. }));
    let _any_timeout = results
        .iter()
        .any(|r| matches!(r.status, AgentRunStatus::Timeout));

    if mode.is_json() {
        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                let (status_str, error_msg) = match &r.status {
                    AgentRunStatus::Ok => ("ok", None),
                    AgentRunStatus::Error { message } => ("error", Some(message.as_str())),
                    AgentRunStatus::Timeout => ("timeout", None),
                };
                serde_json::json!({
                    "agent": r.agent_type,
                    "status": status_str,
                    "log_id": r.log_id,
                    "steps_executed": r.steps_executed,
                    "error": error_msg,
                })
            })
            .collect();

        Output::print(
            &serde_json::json!({
                "status": if all_ok { "ok" } else if any_error { "partial" } else { "error" },
                "results": json_results,
            }),
            mode,
        );
    } else {
        for result in &results {
            let (icon, msg) = match &result.status {
                AgentRunStatus::Ok => (
                    "✓",
                    format!(
                        "completed ({} steps, log_id: {})",
                        result.steps_executed,
                        result.log_id.unwrap_or(0)
                    ),
                ),
                AgentRunStatus::Error { message } => ("✗", format!("failed: {}", message)),
                AgentRunStatus::Timeout => ("⏱", "timed out".to_string()),
            };
            println!("  {} {} {}", icon, result.agent_type, msg);
        }
        println!();
        if all_ok {
            println!("✓ All agents completed successfully");
        } else {
            println!("✗ Some agents failed");
        }
    }

    if !all_ok {
        std::process::exit(1);
    }

    Ok(())
}
