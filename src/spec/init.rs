use crate::bail_app_err;
use anyhow::{Context, Result};
use std::path::Path;

use super::bridge;
use super::templates;

pub fn run(json: bool, _non_interactive: bool, from: Option<&Path>) -> Result<()> {
    let project_dir = Path::new(".");

    let dec_exists = project_dir.join(".dec").exists();
    if !dec_exists {
        bail_app_err!(
            ".dec/ not found. Run `dectl project init` first.",
            "Run `dectl project init` from a dectl project directory"
        );
    }

    templates::ensure_sdd_dir(project_dir)?;

    bridge::update_project_toml(project_dir)?;
    bridge::update_project_isa(project_dir)?;

    let from_content = if let Some(from_path) = from {
        if !from_path.exists() {
            bail_app_err!(
                format!("File not found: {}", from_path.display()),
                format!("The file '{}' does not exist", from_path.display())
            );
        }
        let content = std::fs::read_to_string(from_path)
            .with_context(|| format!("Failed to read file: {}", from_path.display()))?;
        if content.trim().is_empty() {
            bail_app_err!(
                format!("File is empty: {}", from_path.display()),
                format!("The file '{}' is empty", from_path.display())
            );
        }
        Some(content)
    } else {
        None
    };

    let mut message = r#".dec/sdd/ ready
.dec/config/project.toml updated
.dec/isa/project.isa.md updated
Agent: interview the user and create specs/ with real content
  - Read .dec/sdd/references/templates.md for document templates
  - Read .dec/sdd/SKILL.md for the SDD workflow
  - Create specs/ in the project root"#
        .to_string();

    if let Some(ref content) = from_content {
        message.push_str(&format!(
            "\n\n[SOURCE FILE CONTENT]\n{}\n[/SOURCE FILE CONTENT]\nUse the above content as input to create specs following SKILL.md rules.",
            content
        ));
    }

    if json {
        let envelope = serde_json::json!({
            "status": "ok",
            "data": {
                "message": ".dec/sdd/ ready",
                "bridge": {
                    "project_toml": true,
                    "project_isa": true
                },
                "next": "Interview the user and create specs/ with SDD documents",
                "from_file": from.map(|p| p.display().to_string())
            }
        });
        println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
    } else {
        println!("{}", message);
    }

    Ok(())
}
