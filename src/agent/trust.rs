use anyhow::{Context, Result};
use std::path::Path;

pub fn run(agent_type: &str, project: &str, mode: crate::core::output::OutputMode) -> Result<()> {
    let _agent = crate::agent::loader::load_agent(agent_type)
        .with_context(|| format!("Agent '{}' not found", agent_type))?;

    let project_path = Path::new(project);
    if !project_path.exists() {
        anyhow::bail!("Project path does not exist: {}", project);
    }
    let canonical = std::fs::canonicalize(project_path)
        .with_context(|| format!("Failed to resolve project path: {}", project))?;
    let project_str = canonical.to_string_lossy().to_string();

    crate::workflow::trust::grant_trust(&project_str, agent_type)?;

    if !mode.is_json() {
        println!("Agent '{}' trusted for project '{}'", agent_type, project_str);
    }
    Ok(())
}
