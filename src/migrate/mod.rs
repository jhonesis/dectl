pub mod engine;

use crate::core::config::ProjectConfig;
use crate::core::error::AppError;
use crate::core::output::{Output, OutputMode};
use anyhow::Result;

pub fn run(dry_run: bool, mode: OutputMode) -> Result<()> {
    let current = ProjectConfig::load()?.ok_or_else(|| {
        AppError::new("Not a dectl project. Run `dectl project init` first.")
            .with_hint("Run `dectl project init --standard`")
    })?;

    let current_version = current.dec.schema_version.clone();
    let result = engine::MigrationEngine::analyze(&current_version);

    if !dry_run && result.status == "migrations_needed" {
        let mut config = ProjectConfig::load()?.ok_or_else(|| {
            AppError::new("Not a dectl project.").with_hint("Run `dectl project init --standard`")
        })?;
        config.dec.schema_version = engine::SCHEMA_VERSION.to_string();
        config.save()?;
    }

    Output::print(&result, mode);
    Ok(())
}
