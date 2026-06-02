use crate::workflow::schema::{StepType, Workflow};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::interpolate::interpolate;

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
                anyhow::bail!(
                    "Required input '{}' not provided. Use --var {}='<value>'",
                    input_def.name,
                    input_def.name
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
        output: &crate::core::output::OutputMode,
    ) -> Result<ExecutionResult> {
        let start_idx = from_step.unwrap_or(0);

        if start_idx >= workflow.steps.len() {
            anyhow::bail!(
                "from_step {} is out of bounds (workflow has {} steps)",
                start_idx,
                workflow.steps.len()
            );
        }

        let mut results: Vec<StepResult> = Vec::new();
        let mut all_success = true;

        for (idx, step) in workflow.steps.iter().enumerate().skip(start_idx) {
            let step_num = idx + 1;

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
                    });
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
                        let shell_output = Command::new("sh")
                            .args(["-c", &full_cmd])
                            .output()
                            .with_context(|| format!("Failed to execute: {}", full_cmd))?;

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
                                output: Some(format!(
                                    "Exit code: {:?}",
                                    shell_output.status.code()
                                )),
                            });
                            eprintln!(
                                "\n⚠️  Step {} failed. Resume with --from-step {}",
                                step_num, step_num
                            );
                            break;
                        }

                        Some(captured_out)
                    } else {
                        let program = &interp_cmd[0];
                        let args = &interp_cmd[1..];

                        let output = Command::new(program)
                            .args(args)
                            .output()
                            .with_context(|| format!("Failed to execute: {}", program))?;

                        if !output.status.success() {
                            all_success = false;
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            results.push(StepResult {
                                step_num,
                                step_type: "action".to_string(),
                                success: false,
                                output: Some(stderr.to_string()),
                            });
                            eprintln!(
                                "\n⚠️  Step {} failed. Resume with --from-step {}",
                                step_num, step_num
                            );
                            break;
                        }

                        Some(String::from_utf8_lossy(&output.stdout).to_string())
                    };

                    let captured = output_str.clone();

                    results.push(StepResult {
                        step_num,
                        step_type: "action".to_string(),
                        success: true,
                        output: output_str,
                    });

                    if let Some(ref out) = captured {
                        if !out.is_empty() {
                            vars.insert(format!("step_{}_output", step_num), out.clone());
                            vars.insert("last_output".to_string(), out.clone());
                        }
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
                            });
                            all_success = false;
                            eprintln!(
                                "\nStep {} failed: agent step requires agent_type or agent_types",
                                step_num
                            );
                            break;
                        }
                    };

                    if agent_types.is_empty() {
                        results.push(StepResult {
                            step_num,
                            step_type: "agent".to_string(),
                            success: false,
                            output: Some("No agent types specified".to_string()),
                        });
                        all_success = false;
                        eprintln!("\nStep {} failed: no agent types specified", step_num);
                        break;
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
                            false,
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
                                });
                                all_success = false;
                                eprintln!("\n\u{26a0}\u{fe0f}  Step {} failed. Resume with --from-step {}", step_num, step_num);
                                break;
                            }
                        };

                        crate::agent::runner::run_agent(
                            &agent_def, &task, vars, None, dry_run, None, false, output,
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
                                });
                            } else {
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
                                });
                                all_success = false;
                                eprintln!("\n\u{26a0}\u{fe0f}  Step {} failed. Resume with --from-step {}", step_num, step_num);
                                break;
                            }
                        }
                        Err(e) => {
                            results.push(StepResult {
                                step_num,
                                step_type: "agent".to_string(),
                                success: false,
                                output: Some(e.to_string()),
                            });
                            all_success = false;
                            eprintln!(
                                "\n\u{26a0}\u{fe0f}  Step {} failed. Resume with --from-step {}",
                                step_num, step_num
                            );
                            break;
                        }
                    }
                }
            }

            let _ = output;
        }

        Ok(ExecutionResult {
            workflow_name: workflow.name.clone(),
            success: all_success,
            steps_executed: results.len(),
            results,
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
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionResult {
    #[serde(rename = "workflow_name")]
    pub workflow_name: String,
    pub success: bool,
    #[serde(rename = "steps_executed")]
    pub steps_executed: usize,
    pub results: Vec<StepResult>,
}

fn print_step_header(step_num: usize, description: &str) {
    println!("\n╔══ Step {}: {} ══╗", step_num, description);
}
