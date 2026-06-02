pub mod engine;

use crate::core::config::ProjectConfig;
use crate::core::output::{Output, OutputMode};
use anyhow::{anyhow, Result};

pub fn run(dry_run: bool, mode: OutputMode) -> Result<()> {
    let current = ProjectConfig::load()?
        .ok_or_else(|| anyhow!("Not a dectl project. Run `dectl project init` first."))?;

    let current_version = current.dec.schema_version.clone();
    let result = engine::MigrationEngine::analyze(&current_version);

    if !dry_run && result.status == "migrations_needed" {
        let mut config = ProjectConfig::load()?.ok_or_else(|| anyhow!("Not a dectl project."))?;
        config.dec.schema_version = engine::SCHEMA_VERSION.to_string();
        config.save()?;
    }

    Output::print(&result, mode);
    Ok(())
}
