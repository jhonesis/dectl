use anyhow::Result;
use std::path::Path;

use super::bridge;
use super::templates;

pub fn run(json: bool, _non_interactive: bool) -> Result<()> {
    let project_dir = Path::new(".");

    let dec_exists = project_dir.join(".dec").exists();
    if !dec_exists {
        anyhow::bail!(".dec/ not found. Run `dectl project init` first.");
    }

    templates::ensure_sdd_dir(project_dir)?;

    bridge::update_project_toml(project_dir)?;
    bridge::update_project_isa(project_dir)?;

    let message = r#".dec/sdd/ ready
.dec/config/project.toml updated
.dec/isa/project.isa.md updated
Agent: interview the user and create specs/ with real content
  - Read .dec/sdd/references/templates.md for document templates
  - Read .dec/sdd/SKILL.md for the SDD workflow
  - Create specs/ in the project root"#
        .to_string();

    if json {
        let envelope = serde_json::json!({
            "status": "ok",
            "data": {
                "message": ".dec/sdd/ ready",
                "bridge": {
                    "project_toml": true,
                    "project_isa": true
                },
                "next": "Interview the user and create specs/ with SDD documents"
            }
        });
        println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
    } else {
        println!("{}", message);
    }

    Ok(())
}
