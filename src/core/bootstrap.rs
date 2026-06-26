use anyhow::Result;

use super::output::OutputMode;

pub fn initialize(mode: OutputMode) -> Result<()> {
    if !mode.is_json() {
        log::debug!("Initializing dectl...");
    }

    crate::core::db::init_db()?;

    if !mode.is_json() {
        log::debug!("Initialization complete");
    }

    Ok(())
}
