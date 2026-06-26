use crate::bail_app_err;
use crate::workflow::schema::{StepType, Workflow};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::interpolate::interpolate;

fn run_cmd_with_timeout(
    mut cmd: Command,
    timeout_secs: Option<u64>,
    desc: String,
) -> Result<std::process::Output> {
    match timeout_secs {
        Some(timeout) => {
            let output = crate::core::threadpool::with_timeout(move || Ok(cmd.output()?), timeout)
                .map_err(|e| {
                    anyhow::anyhow!("Command timed out after {}s: {} ({})", timeout, desc, e)
                })?;
            Ok(output)
        }
        None => {
            let output = cmd
                .output()
                .with_context(|| format!("Failed to execute: {}", desc))?;
            Ok(output)
        }
    }
}

pub struct Runner;

impl Runner {
    pub fn resolve_inputs(
        workflow: &Workflow,
        provided_vars: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        let mut resolved: HashMap<String, String> = HashMap::new();

        for input_def in &workflow.inputs {
            if let Some(value) = provided_vars.get(&input_def.name) {
                resolved.insert(input_def.name.clone(), value.clone());
            } else if input_def.is_required {
                bail_app_err!(
                    format!(
                        "Required input '{}' not provided. Use --var {}='<value>'",
                        input_def.name, input_def.name
                    ),
                    "Run `dectl workflow list` to see available workflows"
                );
            } else if let Some(default) = &input_def.default {
                resolved.insert(input_def.name.clone(), default.clone());
            }
        }

        Ok(resolved)
    }

    pub fn execute(
        workflow: &Workflow,
        vars: &mut HashMap<String, String>,
        dry_run: bool,
        from_step: Option<usize>,
        auto: bool,
        output: &crate::core::output::OutputMode,
        pause_on_prompt: bool,
    ) -> Result<ExecutionResult> {
        let start_idx = from_step.map(|s| s.saturating_sub(1)).unwrap_or(0);

        if start_idx >= workflow.steps.len() {
            bail_app_err!(
                format!(
                    "from_step {} is out of bounds (workflow has {} steps)",
                    start_idx,
                    workflow.steps.len()
                ),
                "Check the workflow definition in .dec/workflows/ for required inputs"
            );
        }

        let mut results: Vec<StepResult> = Vec::new();
        let mut all_success = true;
        let mut paused = false;

        let has_remaining_run_always = |current_idx: usize| -> bool {
            workflow.steps[current_idx + 1..]
                .iter()
                .any(|s| s.run_always.unwrap_or(false))
        };

        for (idx, step) in workflow.steps.iter().enumerate().skip(start_idx) {
            let step_num = idx + 1;

            if let Some(ref cond) = step.skip_if {
                let should_skip = interpolate(cond, vars)
                    .map(|r| r.trim() == "true" || r.trim() == "1" || r.trim() == "yes")
                    .unwrap_or(false);
                if should_skip {
                    log::info!(
                        "Step {} skipped (condition: {} resolved to true)",
                        step_num,
                        cond
                    );
                    results.push(StepResult {
                        step_num,
                        step_type: format!("{:?}", step.step_type).to_lowercase(),
                        success: true,
                        output: Some(format!("Skipped via skip_if: {}", cond)),
                        stderr: None,
                    });
                    continue;
                }
            }

            if !all_success && !step.run_always.unwrap_or(false) {
                results.push(StepResult {
                    step_num,
                    step_type: format!("{:?}", step.step_type).to_lowercase(),
                    success: false,
                    output: Some("Skipped due to previous failure".to_string()),
                    stderr: None,
                });
                continue;
            }

            if dry_run {
                println!("[DRY-RUN] Step {}: {}", step_num, step.description);
                if let Some(content) = &step.content {
                    let interp = interpolate(content, vars).unwrap_or_else(|_| content.clone());
                    for line in interp.lines() {
                        println!("  > {}", line);
                    }
                }
                if let Some(cmd) = &step.cmd {
                    let mut interp_cmd: Vec<String> = Vec::new();
                    for c in cmd {
                        interp_cmd.push(interpolate(c, vars).unwrap_or_else(|_| c.clone()));
                    }
                    println!("  > CMD: {}", interp_cmd.join(" "));
                }
                if let Some(path) = &step.path {
                    let interp_path = interpolate(path, vars).unwrap_or_else(|_| path.clone());
                    println!("  > WRITE: {}", interp_path);
                }
                println!();
                continue;
            }

            print_step_header(step_num, &step.description);

            match step.step_type {
                StepType::Prompt => {
                    let content = step.content.as_ref().unwrap();
                    let interp = interpolate(content, vars).context("Interpolation error")?;
                    println!("\n{}", interp);
                    results.push(StepResult {
                        step_num,
                        step_type: "prompt".to_string(),
                        success: true,
                        output: None,
                        stderr: None,
                    });
                    if pause_on_prompt {
                        paused = true;
                        println!(
                            "\n⏸️  Workflow paused at step {}. Resume with:\n   dectl workflow run {} --from-step {}",
                            step_num,
                            workflow.name,
                            step_num + 1
                        );
                        break;
                    }
                }
                StepType::Action => {
                    let cmd = step.cmd.as_ref().unwrap();
                    let shell = step.shell.unwrap_or(false);

                    let mut interp_cmd: Vec<String> = Vec::new();
                    for c in cmd {
                        interp_cmd.push(interpolate(c, vars).context("Interpolation error")?);
                    }

                    let output_str = if shell {
                        let full_cmd = interp_cmd.join(" ");
                        let mut sh_cmd = Command::new("sh");
                        sh_cmd.args(["-c", &full_cmd]);
                        let shell_output = run_cmd_with_timeout(
                            sh_cmd,
                            step.timeout_secs,
                            format!("sh -c '{}'", full_cmd),
                        )?;

                        let captured_out =
                            String::from_utf8_lossy(&shell_output.stdout).to_string();
                        let captured_err =
                            String::from_utf8_lossy(&shell_output.stderr).to_string();

                        if !captured_out.is_empty() {
                            print!("{}", captured_out);
                        }
                        if !captured_err.is_empty() {
                            eprint!("{}", captured_err);
                        }

                        if !shell_output.status.success() {
                            all_success = false;
                            results.push(StepResult {
                                step_num,
                                step_type: "action".to_string(),
                                success: false,
                                output: Some(captured_out),
                                stderr: Some(captured_err),
                            });
                            eprintln!(
                                "\n⚠️  Step {} failed. Resume with --from-step {}",
                                step_num, step_num
                            );
                            if !has_remaining_run_always(idx) {
                                break;
                            }
                            continue;
                        }

                        Some((captured_out, captured_err))
                    } else {
                        let program = &interp_cmd[0];
                        let args: Vec<&String> = interp_cmd[1..].iter().collect();

                        let mut prog_cmd = Command::new(program);
                        prog_cmd.args(args.clone());
                        let output = run_cmd_with_timeout(
                            prog_cmd,
                            step.timeout_secs,
                            format!(
                                "{} {}",
                                program,
                                args.iter()
                                    .map(|s| s.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            ),
                        )?;

                        let cmd_stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        if !output.status.success() {
                            all_success = false;
                            results.push(StepResult {
                                step_num,
                                step_type: "action".to_string(),
                                success: false,
                                output: Some(String::from_utf8_lossy(&output.stdout).to_string()),
                                stderr: Some(cmd_stderr),
                            });
                            eprintln!(
                                "\n⚠️  Step {} failed. Resume with --from-step {}",
                                step_num, step_num
                            );
                            if !has_remaining_run_always(idx) {
                                break;
                            }
                            continue;
                        }

                        Some((
                            String::from_utf8_lossy(&output.stdout).to_string(),
                            cmd_stderr,
                        ))
                    };

                    let captured = output_str.clone().map(|(out, _)| out);

                    let action_stderr = output_str.clone().map(|(_, err)| err);

                    results.push(StepResult {
                        step_num,
                        step_type: "action".to_string(),
                        success: true,
                        output: captured.clone(),
                        stderr: action_stderr,
                    });

                    if let Some(ref out) = captured {
                        vars.insert(format!("step_{}_output", step_num), out.clone());
                        vars.insert("last_output".to_string(), out.clone());
                    }
                }
                StepType::Write => {
                    let path = step.path.as_ref().unwrap();
                    let content = step.content.as_ref().unwrap();

                    let interp_path = interpolate(path, vars).context("Interpolation error")?;
                    let interp_content =
                        interpolate(content, vars).context("Interpolation error")?;

                    let parent = Path::new(&interp_path).parent().unwrap_or(Path::new("."));
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {:?}", parent))?;

                    fs::write(&interp_path, &interp_content)
                        .with_context(|| format!("Failed to write file: {}", interp_path))?;

                    println!("✓ Wrote: {}", interp_path);
                    results.push(StepResult {
                        step_num,
                        step_type: "write".to_string(),
                        success: true,
                        output: None,
                        stderr: None,
                    });
                }
                StepType::Agent => {
                    let agent_types: Vec<String> = if step.parallel.unwrap_or(false) {
                        step.agent_types.clone().unwrap_or_default()
                    } else {
                        if let Some(ref agent_type) = step.agent_type {
                            vec![agent_type.clone()]
                        } else {
                            results.push(StepResult {
                                step_num,
                                step_type: "agent".to_string(),
                                success: false,
                                output: Some("Agent step missing agent_type".to_string()),
                                stderr: None,
                            });
                            all_success = false;
                            eprintln!(
                                "\nStep {} failed: agent step requires agent_type or agent_types",
                                step_num
                            );
                            if !has_remaining_run_always(idx) {
                                break;
                            }
                            continue;
                        }
                    };

                    if agent_types.is_empty() {
                        results.push(StepResult {
                            step_num,
                            step_type: "agent".to_string(),
                            success: false,
                            output: Some("No agent types specified".to_string()),
                            stderr: None,
                        });
                        all_success = false;
                        log::error!("\nStep {} failed: no agent types specified", step_num);
                        if !has_remaining_run_always(idx) {
                            break;
                        }
                        continue;
                    }

                    let task_raw = step.task.as_deref().unwrap_or("");
                    let task = interpolate(task_raw, vars).context("Interpolation error")?;

                    let agent_results = if step.parallel.unwrap_or(false) {
                        crate::agent::parallel::run_parallel(
                            &agent_types,
                            &task,
                            vars,
                            None,
                            dry_run,
                            auto,
                            output,
                        )
                    } else {
                        let agent_def = match crate::agent::loader::load_agent(&agent_types[0]) {
                            Some((def, _)) => def,
                            None => {
                                let msg = format!("Agent '{}' not found", agent_types[0]);
                                results.push(StepResult {
                                    step_num,
                                    step_type: "agent".to_string(),
                                    success: false,
                                    output: Some(msg.clone()),
                                    stderr: None,
                                });
                                all_success = false;
                                eprintln!("\n\u{26a0}\u{fe0f}  Step {} failed. Resume with --from-step {}", step_num, step_num);
                                if !has_remaining_run_always(idx) {
                                    break;
                                }
                                continue;
                            }
                        };

                        crate::agent::runner::run_agent(
                            &agent_def, &task, vars, None, dry_run, None, auto, output, auto, true,
                        )
                        .map(|r| vec![r])
                    };

                    match agent_results {
                        Ok(results_list) => {
                            let all_agent_ok = results_list.iter().all(|r| {
                                matches!(r.status, crate::agent::schema::AgentRunStatus::Ok)
                            });
                            if all_agent_ok {
                                println!("\u{2713} Agent step completed");
                                results.push(StepResult {
                                    step_num,
                                    step_type: "agent".to_string(),
                                    success: true,
                                    output: Some(format!(
                                        "{} agent(s) completed",
                                        results_list.len()
                                    )),
                                    stderr: None,
                                });
                            } else {
                                let failed_details: Vec<String> = results_list
                                    .iter()
                                    .filter(|r| {
                                        !matches!(
                                            r.status,
                                            crate::agent::schema::AgentRunStatus::Ok
                                        )
                                    })
                                    .map(|r| {
                                        let detail = match &r.status {
                                            crate::agent::schema::AgentRunStatus::Error {
                                                message,
                                            } => {
                                                format!("{}: {:?}", r.agent_type, message)
                                            }
                                            crate::agent::schema::AgentRunStatus::Timeout => {
                                                format!("{}: timeout", r.agent_type)
                                            }
                                            _ => r.agent_type.clone(),
                                        };
                                        detail
                                    })
                                    .collect();
                                for detail in &failed_details {
                                    eprintln!("  ✗ {}", detail);
                                }
                                let failed: Vec<&str> = results_list
                                    .iter()
                                    .filter(|r| {
                                        !matches!(
                                            r.status,
                                            crate::agent::schema::AgentRunStatus::Ok
                                        )
                                    })
                                    .map(|r| r.agent_type.as_str())
                                    .collect();
                                let msg = format!("Agent(s) failed: {}", failed.join(", "));
                                results.push(StepResult {
                                    step_num,
                                    step_type: "agent".to_string(),
                                    success: false,
                                    output: Some(msg.clone()),
                                    stderr: None,
                                });
                                all_success = false;
                                eprintln!("\n\u{26a0}\u{fe0f}  Step {} failed. Resume with --from-step {}", step_num, step_num);
                                if !has_remaining_run_always(idx) {
                                    break;
                                }
                                continue;
                            }
                        }
                        Err(e) => {
                            results.push(StepResult {
                                step_num,
                                step_type: "agent".to_string(),
                                success: false,
                                output: Some(e.to_string()),
                                stderr: None,
                            });
                            all_success = false;
                            eprintln!(
                                "\n\u{26a0}\u{fe0f}  Step {} failed. Resume with --from-step {}",
                                step_num, step_num
                            );
                            if !has_remaining_run_always(idx) {
                                break;
                            }
                            continue;
                        }
                    }
                }
            }

            let _ = output;
        }

        let fail_logs: Vec<String> = results
            .iter()
            .filter_map(|r| if !r.success { r.stderr.clone() } else { None })
            .collect();

        Ok(ExecutionResult {
            workflow_name: workflow.name.clone(),
            success: all_success,
            steps_executed: results.len(),
            paused,
            results,
            logs: fail_logs,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StepResult {
    #[serde(rename = "step_num")]
    pub step_num: usize,
    #[serde(rename = "step_type")]
    pub step_type: String,
    pub success: bool,
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionResult {
    #[serde(rename = "workflow_name")]
    pub workflow_name: String,
    pub success: bool,
    #[serde(rename = "steps_executed")]
    pub steps_executed: usize,
    #[serde(default)]
    pub paused: bool,
    pub results: Vec<StepResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub logs: Vec<String>,
}

fn print_step_header(step_num: usize, description: &str) {
    println!("\n╔══ Step {}: {} ══╗", step_num, description);
}
