use crate::core::output::OutputMode;
use anyhow::Result;
use std::path::Path;

pub fn run(mode: OutputMode) -> Result<()> {
    let workflows_dir = Path::new(".dec/workflows");

    if !workflows_dir.exists() {
        if mode.is_json() {
            crate::core::output::Output::print(
                &serde_json::json!({
                    "status": "ok",
                    "workflows": [],
                    "message": "No .dec/workflows/ directory found"
                }),
                mode,
            );
        } else {
            println!(
                "No workflows found. Run 'dectl project init --standard' to create workflows."
            );
        }
        return Ok(());
    }

    let workflows = crate::workflow::loader::list_workflows_in_dir(workflows_dir);

    if mode.is_json() {
        let results: Vec<serde_json::Value> = workflows
            .into_iter()
            .map(|(name, result)| match result {
                Ok(w) => serde_json::json!({
                    "name": name,
                    "description": w.description,
                    "inputs": w.inputs.iter().map(|i| serde_json::json!({
                        "name": i.name,
                        "description": i.description,
                        "required": i.is_required,
                        "default": i.default
                    })).collect::<Vec<_>>()
                }),
                Err(e) => serde_json::json!({
                    "name": name,
                    "error": e.to_string()
                }),
            })
            .collect();

        crate::core::output::Output::print(
            &serde_json::json!({
                "status": "ok",
                "workflows": results
            }),
            mode,
        );
    } else {
        println!("Available workflows in .dec/workflows/:\n");

        let mut has_valid = false;
        for (name, result) in &workflows {
            match result {
                Ok(w) => {
                    has_valid = true;
                    println!("  {:20} {}", name, w.description);
                    let required: Vec<_> = w.inputs.iter().filter(|i| i.is_required).collect();
                    if !required.is_empty() {
                        println!(
                            "    Required: {}",
                            required
                                .iter()
                                .map(|i| i.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }
                Err(e) => {
                    eprintln!("  {} (ERROR: {})", name, e);
                }
            }
        }

        if !has_valid {
            println!("  (no valid workflows)");
        }
    }

    Ok(())
}
