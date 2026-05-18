use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::templates::{InitLevel, Templates};

pub fn run(level: InitLevel, interactive: bool) -> Result<()> {
    let dec_path = Path::new(".dec");

    if dec_path.exists() {
        anyhow::bail!(
            ".dec/ already exists. Remove it first if you want to reinitialize.\n\
             Hint: rm -rf .dec/"
        );
    }

    let files = Templates::files_for_level(level);

    for (path, _content) in &files {
        let full_path = Path::new(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }
    }

    for (path, content) in &files {
        if content.is_empty() {
            continue;
        }
        fs::write(path, *content).with_context(|| format!("Failed to write file {:?}", path))?;
    }

    let level_name = match level {
        InitLevel::Level1 => "level 1 (minimal)",
        InitLevel::Level2 => "level 2 (standard)",
        InitLevel::Level3 => "level 3 (full)",
    };
    println!("Created .dec/ with {} ({} files)", level_name, files.len());
    println!("\nNext steps:");
    println!("  1. Edit .dec/config/project.toml with your project details");
    println!("  2. Edit .dec/isa/project.isa.md to define your vision");
    println!("  3. Run 'dectl project info' to verify the setup");

    if interactive {
        println!("\n⚠️  Review .dec/.gitignore before committing to git.");
    }

    Ok(())
}
