use crate::session::types::SessionHook;
use crate::workflow::loader;
use crate::workflow::runner::Runner;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn load_hooks() -> Result<Vec<SessionHook>> {
    let project_toml = Path::new(".dec/config/project.toml");
    if !project_toml.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(project_toml)?;
    let table: toml::Value = toml::from_str(&content)?;

    if let Some(session) = table.get("session").and_then(|s| s.as_table()) {
        if let Some(hooks) = session.get("hooks").and_then(|h| h.as_array()) {
            let mut result = Vec::new();
            for hook_val in hooks {
                if let Some(hook_name) = hook_val.as_str() {
                    result.push(SessionHook {
                        name: hook_name.to_string(),
                        workflow: hook_name.to_string(),
                        vars: HashMap::new(),
                        run_always: true,
                    });
                }
            }
            return Ok(result);
        }
    }

    Ok(Vec::new())
}

pub fn execute_hooks(hooks: &[SessionHook], dry_run: bool) -> Vec<(String, bool, String)> {
    let mut results = Vec::new();
    let mode = crate::core::output::OutputMode::Human;

    for hook in hooks {
        if dry_run {
            results.push((
                format!("hook:{}", hook.name),
                true,
                format!("would run workflow '{}'", hook.workflow),
            ));
            continue;
        }

        match run_hook_workflow_direct(hook, &mode) {
            Ok(()) => {
                results.push((
                    format!("hook:{}", hook.name),
                    true,
                    format!("hook '{}' completed", hook.name),
                ));
            }
            Err(e) => {
                let msg = format!("hook '{}' failed: {}", hook.name, e);
                results.push((format!("hook:{}", hook.name), hook.run_always, msg));
            }
        }
    }

    results
}

fn run_hook_workflow_direct(
    hook: &SessionHook,
    _mode: &crate::core::output::OutputMode,
) -> Result<()> {
    let workflow_path = Path::new(".dec/workflows").join(format!("{}.yaml", hook.workflow));
    if !workflow_path.exists() {
        anyhow::bail!("Workflow '{}' not found for hook", hook.workflow);
    }

    let workflow = loader::load_workflow(&workflow_path)?;
    let mut vars = hook.vars.clone();

    let _result = Runner::execute(&workflow, &mut vars, false, None, true, _mode, false)?;
    Ok(())
}
