use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct SddTemplates;

impl SddTemplates {
    pub const SKILL_MD: &'static str =
        include_str!("../project/templates/txt/sdd_skill.md");
    pub const TEMPLATES_MD: &'static str =
        include_str!("../project/templates/txt/sdd_templates.md");
    pub const EXAMPLES_MD: &'static str =
        include_str!("../project/templates/txt/sdd_examples.md");

    pub fn sdd_files() -> Vec<(&'static str, &'static str)> {
        vec![
            (".dec/sdd/SKILL.md", Self::SKILL_MD),
            (".dec/sdd/references/templates.md", Self::TEMPLATES_MD),
            (".dec/sdd/references/examples.md", Self::EXAMPLES_MD),
        ]
    }
}

pub fn ensure_sdd_dir(project_dir: &Path) -> Result<()> {
    let dec_sdd = project_dir.join(".dec/sdd");
    if dec_sdd.exists() {
        return Ok(());
    }

    fs::create_dir_all(dec_sdd.join("references"))
        .context("Failed to create .dec/sdd/references/")?;

    for (path, content) in SddTemplates::sdd_files() {
        let full_path = project_dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, content).with_context(|| format!("Failed to write {}", path))?;
    }

    Ok(())
}
