use crate::bail_app_err;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn update_project_toml(project_dir: &Path) -> Result<()> {
    let toml_path = project_dir.join(".dec/config/project.toml");
    if !toml_path.exists() {
        bail_app_err!(
            ".dec/config/project.toml not found. Run `dectl project init` first.",
            "Run `dectl project init --standard`"
        );
    }

    use crate::core::toml_util;
    toml_util::ensure_section(&toml_path, "specs")?;
    toml_util::update_field(&toml_path, "specs.dir", "specs")?;

    Ok(())
}

pub fn update_project_isa(project_dir: &Path) -> Result<()> {
    let isa_path = project_dir.join(".dec/isa/project.isa.md");
    if !isa_path.exists() {
        bail_app_err!(
            ".dec/isa/project.isa.md not found. Run `dectl project init` first.",
            "Run `dectl project init --standard`"
        );
    }

    let content = fs::read_to_string(&isa_path).context("Failed to read project.isa.md")?;

    let link_line =
        "\n\n## SDD\n\nSee `specs/` for SDD artifacts (constitution, spec, plan, tasks, etc.)";

    if content.contains("See `specs/` for SDD artifacts") {
        return Ok(());
    }

    let new_content = format!("{}{}", content.trim_end(), link_line);
    fs::write(&isa_path, new_content).context("Failed to write project.isa.md")?;

    Ok(())
}
