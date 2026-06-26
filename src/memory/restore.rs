use crate::bail_app_err;
use anyhow::Result;
use serde::Serialize;

use crate::core::db::{get_db, Storage};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryRestoreOutput {
    pub id: i64,
    pub restored: bool,
}

pub fn run(id: i64, mode: OutputMode) -> Result<()> {
    let db = get_db()?;

    let exists: bool = db
        .query_row(
            "SELECT id FROM memories WHERE id = ?1 AND deleted_at IS NOT NULL",
            rusqlite::params![id],
            |row| row.get::<_, i64>(0),
        )
        .is_ok();

    if !exists {
        let already_active: bool = db
            .query_row(
                "SELECT id FROM memories WHERE id = ?1 AND deleted_at IS NULL",
                rusqlite::params![id],
                |row| row.get::<_, i64>(0),
            )
            .is_ok();

        if already_active {
            bail_app_err!(
                format!("Memory entry #{} is not deleted", id),
                "Use `dectl memory list --include-deleted` to find deleted entries"
            );
        }

        bail_app_err!(
            format!("Memory entry #{} not found", id),
            "Run `dectl memory list` to find valid IDs"
        );
    }

    let now = chrono::Utc::now().to_rfc3339();
    db.execute(
        "UPDATE memories SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    )?;

    let output = MemoryRestoreOutput { id, restored: true };

    mode.print(&output)?;

    Ok(())
}
