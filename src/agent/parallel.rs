use crate::agent::schema::{AgentResult, AgentRunStatus};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

pub fn run_parallel(
    agent_types: &[String],
    task: &str,
    vars: &HashMap<String, String>,
    timeout_secs: Option<u64>,
    dry_run: bool,
    non_interactive: bool,
    mode: &crate::core::output::OutputMode,
) -> Result<Vec<AgentResult>> {
    let mut agents = Vec::new();
    for agent_type in agent_types {
        let agent = crate::agent::loader::load_agent(agent_type);
        match agent {
            Some((def, _)) => agents.push(def),
            None => anyhow::bail!(
                "Agent '{}' not found. Run 'dectl agent list' to see available agents.",
                agent_type
            ),
        }
    }

    let task = task.to_string();
    let vars = vars.clone();
    let mode = *mode;

    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    for (i, agent_def) in agents.into_iter().enumerate() {
        let tx = tx.clone();
        let task = task.clone();
        let vars = vars.clone();
        let agent_type = agent_types[i].clone();

        let handle = thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::agent::runner::run_agent(
                    &agent_def,
                    &task,
                    &vars,
                    None,
                    dry_run,
                    timeout_secs,
                    non_interactive,
                    &mode,
                )
            }));

            let agent_result = match result {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => AgentResult {
                    agent_type: agent_type.clone(),
                    status: AgentRunStatus::Error {
                        message: e.to_string(),
                    },
                    steps_executed: 0,
                    log_id: None,
                },
                Err(panic) => AgentResult {
                    agent_type: agent_type.clone(),
                    status: AgentRunStatus::Error {
                        message: format!(
                            "Agent '{}' panicked: {:?}",
                            agent_type,
                            panic.downcast_ref::<&str>().unwrap_or(&"unknown panic")
                        ),
                    },
                    steps_executed: 0,
                    log_id: None,
                },
            };
            let _ = tx.send((agent_type, agent_result));
        });
        handles.push(handle);
    }

    drop(tx);

    let mut results: Vec<(String, AgentResult)> = Vec::new();
    for received in rx {
        results.push(received);
    }

    for handle in handles {
        let _ = handle.join();
    }

    let mut ordered: Vec<AgentResult> = Vec::new();
    for agent_type in agent_types {
        if let Some((_, result)) = results.iter().find(|(t, _)| t == agent_type) {
            ordered.push(result.clone());
        }
    }

    Ok(ordered)
}
