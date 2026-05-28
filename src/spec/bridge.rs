use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn update_project_toml(project_dir: &Path) -> Result<()> {
    let toml_path = project_dir.join(".dec/config/project.toml");
    if !toml_path.exists() {
        anyhow::bail!(".dec/config/project.toml not found. Run `dectl project init` first.");
    }

    let content = fs::read_to_string(&toml_path).context("Failed to read project.toml")?;

    let mut doc: toml::map::Map<String, toml::Value> =
        toml::from_str(&content).unwrap_or_else(|_| toml::map::Map::new());

    let specs_entry = doc
        .entry("specs".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

    if let Some(specs_table) = specs_entry.as_table_mut() {
        specs_table.insert("dir".to_string(), toml::Value::String("specs".to_string()));
    }

    let new_content = toml::to_string_pretty(&doc).context("Failed to serialize project.toml")?;
    fs::write(&toml_path, new_content).context("Failed to write project.toml")?;

    Ok(())
}

pub fn update_project_isa(project_dir: &Path) -> Result<()> {
    let isa_path = project_dir.join(".dec/isa/project.isa.md");
    if !isa_path.exists() {
        anyhow::bail!(".dec/isa/project.isa.md not found. Run `dectl project init` first.");
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
