use serde::Serialize;

pub const SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Serialize)]
pub struct MigrateResult {
    pub from: String,
    pub to: String,
    pub status: String,
    pub steps: Vec<MigrationStep>,
}

#[derive(Debug, Serialize)]
pub struct MigrationStep {
    pub from: String,
    pub to: String,
    pub description: String,
}

pub struct MigrationEngine;

impl MigrationEngine {
    pub fn analyze(current_version: &str) -> MigrateResult {
        if current_version == SCHEMA_VERSION {
            MigrateResult {
                from: current_version.to_string(),
                to: SCHEMA_VERSION.to_string(),
                status: "up_to_date".into(),
                steps: vec![],
            }
        } else {
            MigrateResult {
                from: current_version.to_string(),
                to: SCHEMA_VERSION.to_string(),
                status: "migrations_needed".into(),
                steps: vec![MigrationStep {
                    from: current_version.to_string(),
                    to: SCHEMA_VERSION.to_string(),
                    description: format!(
                        "Migrate schema from {} to {}",
                        current_version, SCHEMA_VERSION
                    ),
                }],
            }
        }
    }
}
