use crate::core::output::OutputMode;
use crate::workflow::loader::load_workflow;
use anyhow::Result;
use std::path::Path;

pub fn run(name: &str, mode: OutputMode) -> Result<()> {
    let workflow_path = Path::new(".dec/workflows").join(format!("{}.yaml", name));

    if !workflow_path.exists() {
        anyhow::bail!(
            "Workflow '{}' not found. Run 'dectl workflow list' to see available workflows.",
            name
        );
    }

    let workflow = load_workflow(&workflow_path)?;

    if mode.is_json() {
        crate::core::output::Output::print(
            &serde_json::json!({
                "status": "ok",
                "workflow": {
                    "name": workflow.name,
                    "description": workflow.description,
                    "inputs": workflow.inputs.iter().map(|i| serde_json::json!({
                        "name": i.name,
                        "description": i.description,
                        "required": i.is_required,
                        "default": i.default
                    })).collect::<Vec<_>>(),
                    "steps": workflow.steps.iter().enumerate().map(|(idx, s)| {
                        let mut obj = serde_json::json!({
                            "index": idx + 1,
                            "type": s.step_type.to_string(),
                            "description": s.description
                        });
                        if let Some(c) = &s.content {
                            obj["content"] = serde_json::json!(c);
                        }
                        if let Some(c) = &s.cmd {
                            obj["cmd"] = serde_json::json!(c);
                        }
                        if let Some(p) = &s.path {
                            obj["path"] = serde_json::json!(p);
                        }
                        if let Some(shell) = &s.shell {
                            obj["shell"] = serde_json::json!(shell);
                        }
                        obj
                    }).collect::<Vec<_>>()
                }
            }),
            mode,
        );
    } else {
        println!("Workflow: {}\n", workflow.name);
        println!("Description: {}\n", workflow.description);

        if !workflow.inputs.is_empty() {
            println!("Inputs:");
            for input in &workflow.inputs {
                let req_opt = if input.is_required {
                    "required"
                } else {
                    "optional"
                };
                let default = input
                    .default
                    .as_ref()
                    .map(|d| format!(" (default: {})", d))
                    .unwrap_or_default();
                println!(
                    "  - {}: {} [{}]{}",
                    input.name, input.description, req_opt, default
                );
            }
            println!();
        }

        println!("Steps:");
        for (idx, step) in workflow.steps.iter().enumerate() {
            println!("  [{}] {} ({})", idx + 1, step.description, step.step_type);

            if let Some(content) = &step.content {
                let preview = if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content.clone()
                };
                println!(
                    "      content: {}",
                    preview.replace("\n", "\n             ")
                );
            }
            if let Some(cmd) = &step.cmd {
                println!("      cmd: {}", cmd.join(" "));
            }
            if let Some(path) = &step.path {
                println!("      path: {}", path);
            }
            if let Some(shell) = step.shell {
                if shell {
                    println!("      shell: true");
                }
            }
        }
    }

    Ok(())
}
