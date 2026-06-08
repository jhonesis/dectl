use crate::core::output::OutputMode;
use crate::workflow::runner::Runner;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn run(
    name: &str,
    var: Vec<String>,
    dry_run: bool,
    from_step: Option<usize>,
    auto: bool,
    non_interactive: bool,
    mode: OutputMode,
) -> Result<()> {
    let workflow_path = Path::new(".dec/workflows").join(format!("{}.yaml", name));

    if !workflow_path.exists() {
        anyhow::bail!(
            "Workflow '{}' not found. Run 'dectl workflow list' to see available workflows.",
            name
        );
    }

    let workflow = crate::workflow::loader::load_workflow(&workflow_path)?;
    let project_path = std::env::current_dir()
        .and_then(|p| std::fs::canonicalize(&p))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let has_action_steps = workflow
        .steps
        .iter()
        .any(|s| s.step_type == crate::workflow::StepType::Action);

    if !auto {
        let trust_decision = crate::workflow::trust::check_trust(
            &project_path,
            name,
            has_action_steps,
            non_interactive,
        )?;

        match trust_decision {
            crate::workflow::trust::TrustDecision::RequiresConfirmation => {
                anyhow::bail!(
                    "Workflow '{}' contains action steps that are not trusted.\n\
                     Trust this workflow for this project? Edit ~/.dectl/trust.toml manually.",
                    name
                );
            }
            crate::workflow::trust::TrustDecision::AskUser => {
                println!("Workflow '{}' contains action steps:", name);
                for step in &workflow.steps {
                    if step.step_type == crate::workflow::StepType::Action {
                        if let Some(cmd) = &step.cmd {
                            println!("  - {}", cmd.join(" "));
                        }
                    }
                }
                print!("\nDo you trust this workflow? (y/N): ");
                std::io::Write::flush(&mut std::io::stdout())?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    println!("Aborted.");
                    return Ok(());
                }

                crate::workflow::trust::grant_trust(&project_path, name)?;
                println!("✓ Trusted. This workflow is now trusted for this project.");
            }
            crate::workflow::trust::TrustDecision::Trusted => {}
        }
    }

    let mut provided_vars: HashMap<String, String> = HashMap::new();
    for v in var {
        if let Some((k, v)) = v.split_once('=') {
            provided_vars.insert(k.to_string(), v.to_string());
        } else {
            anyhow::bail!("Invalid --var format: '{}'. Expected 'name=value'", v);
        }
    }

    let mut vars = Runner::resolve_inputs(&workflow, &provided_vars)?;

    if !mode.is_json() {
        println!("\n🔄 Running workflow: {}", workflow.name);
        if !vars.is_empty() {
            println!("Variables:");
            for (k, v) in &vars {
                println!("  {} = {}", k, v);
            }
        }
        println!();
    }

    let result = Runner::execute(&workflow, &mut vars, dry_run, from_step, auto, &mode, true)?;

    if mode.is_json() {
        let json_result = if result.success && !result.paused {
            serde_json::json!({
                "status": "ok",
                "workflow": workflow.name,
                "steps_executed": result.steps_executed,
                "results": result.results
            })
        } else if result.paused {
            serde_json::json!({
                "status": "paused",
                "workflow": workflow.name,
                "steps_executed": result.steps_executed,
                "next_step": result.steps_executed + 1,
                "hint": format!("Resume with: dectl workflow run {} --from-step {}", name, result.steps_executed + 1),
                "results": result.results
            })
        } else {
            let failed_step = result
                .results
                .iter()
                .find(|r| !r.success)
                .map(|s| s.step_num);
            serde_json::json!({
                "status": "error",
                "workflow": workflow.name,
                "steps_executed": result.steps_executed,
                "failed_step": failed_step,
                "hint": "Use --from-step N to resume after fixing the issue",
                "results": result.results
            })
        };
        crate::core::output::Output::print(&json_result, mode);
    } else {
        if result.paused {
            println!(
                "\n⏸️  Workflow '{}' paused at step {} (researcher + coder completed)",
                workflow.name, result.steps_executed
            );
            println!(
                "   Resume after implementing: dectl workflow run {} --from-step {}",
                name,
                result.steps_executed + 1
            );
        } else if result.success {
            println!(
                "\n✓ Workflow '{}' completed successfully ({} steps)",
                workflow.name, result.steps_executed
            );
        } else {
            println!(
                "\n✗ Workflow '{}' failed at step {}",
                workflow.name, result.steps_executed
            );
            println!(
                "   Resume with: dectl workflow run {} --from-step {}",
                name, result.steps_executed
            );
        }
    }

    if !result.success {
        std::process::exit(1);
    }

    Ok(())
}
