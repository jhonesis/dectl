use crate::workflow::Workflow;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::interpolate::extract_variables;

pub fn load_workflow(workflow_path: &Path) -> Result<Workflow> {
    let yaml_content = fs::read_to_string(workflow_path)
        .with_context(|| format!("Failed to read workflow file: {:?}", workflow_path))?;

    let workflow: Workflow = serde_yaml::from_str(&yaml_content)
        .with_context(|| format!("Failed to parse workflow YAML: {:?}", workflow_path))?;

    validate_workflow(&workflow)?;

    Ok(workflow)
}

pub fn validate_workflow(workflow: &Workflow) -> Result<()> {
    if workflow.name.is_empty() {
        anyhow::bail!("Workflow name cannot be empty");
    }

    if workflow.steps.is_empty() {
        anyhow::bail!("Workflow '{}' must have at least one step", workflow.name);
    }

    let declared_inputs: HashSet<&str> = workflow.inputs.iter().map(|i| i.name.as_str()).collect();

    let mut referenced_vars: Vec<String> = Vec::new();
    for (idx, step) in workflow.steps.iter().enumerate() {
        if let Some(content) = &step.content {
            referenced_vars.extend(extract_variables(content));
        }
        if let Some(path) = &step.path {
            referenced_vars.extend(extract_variables(path));
        }
        if let Some(cmd) = &step.cmd {
            for arg in cmd {
                referenced_vars.extend(extract_variables(arg));
            }
        }

        match step.step_type {
            crate::workflow::StepType::Prompt => {
                if step.content.is_none() {
                    anyhow::bail!(
                        "Step {} ('{}'): 'prompt' type requires 'content' field",
                        idx + 1,
                        step.description
                    );
                }
            }
            crate::workflow::StepType::Action => {
                if step.cmd.is_none() {
                    anyhow::bail!(
                        "Step {} ('{}'): 'action' type requires 'cmd' field",
                        idx + 1,
                        step.description
                    );
                }
            }
            crate::workflow::StepType::Write => {
                if step.path.is_none() || step.content.is_none() {
                    anyhow::bail!(
                        "Step {} ('{}'): 'write' type requires 'path' and 'content' fields",
                        idx + 1,
                        step.description
                    );
                }
            }
            crate::workflow::StepType::Agent => {
                if step.task.is_none() {
                    anyhow::bail!(
                        "Step {} ('{}'): 'agent' type requires 'task' field",
                        idx + 1,
                        step.description
                    );
                }
            }
        }
    }

    for var_name in &referenced_vars {
        if !declared_inputs.contains(var_name.as_str()) {
            anyhow::bail!(
                "Variable '{{{{{}}}}}' referenced in workflow but not declared in inputs. Declare it in the 'inputs' section.",
                var_name
            );
        }
    }

    for input in &workflow.inputs {
        if !input.is_required && input.default.is_none() {
            anyhow::bail!(
                "Input '{}' is optional but has no default value",
                input.name
            );
        }
    }

    Ok(())
}

pub fn list_workflows_in_dir(dir_path: &Path) -> Vec<(String, Result<Workflow>)> {
    let mut results = Vec::new();

    if !dir_path.exists() {
        return results;
    }

    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let workflow = load_workflow(&path);
            results.push((name, workflow));
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::schema::{InputDefinition, Step, StepType};

    fn create_test_workflow() -> Workflow {
        Workflow {
            name: "test_workflow".to_string(),
            description: "Test workflow".to_string(),
            inputs: vec![
                InputDefinition {
                    name: "feature_name".to_string(),
                    description: "The feature name".to_string(),
                    is_required: true,
                    default: None,
                },
                InputDefinition {
                    name: "module".to_string(),
                    description: "The module".to_string(),
                    is_required: false,
                    default: Some("src".to_string()),
                },
            ],
            steps: vec![Step {
                step_type: StepType::Prompt,
                description: "Test step".to_string(),
                content: Some("Feature: {{feature_name}}".to_string()),
                cmd: None,
                path: None,
                shell: None,
                agent_type: None,
                agent_types: None,
                task: None,
                parallel: None,
            }],
        }
    }

    #[test]
    fn test_validate_workflow_valid() {
        let workflow = create_test_workflow();
        assert!(validate_workflow(&workflow).is_ok());
    }

    #[test]
    fn test_validate_workflow_empty_name() {
        let mut workflow = create_test_workflow();
        workflow.name = "".to_string();
        assert!(validate_workflow(&workflow).is_err());
    }

    #[test]
    fn test_validate_workflow_no_steps() {
        let mut workflow = create_test_workflow();
        workflow.steps = vec![];
        assert!(validate_workflow(&workflow).is_err());
    }

    #[test]
    fn test_validate_workflow_undeclared_variable() {
        let mut workflow = create_test_workflow();
        workflow.steps[0].content = Some("{{undeclared}}".to_string());
        let err = validate_workflow(&workflow).unwrap_err();
        assert!(err.to_string().contains("undeclared"));
    }

    #[test]
    fn test_validate_workflow_optional_without_default() {
        let mut workflow = create_test_workflow();
        workflow.inputs[1].default = None;
        let err = validate_workflow(&workflow).unwrap_err();
        assert!(err.to_string().contains("optional"));
    }

    #[test]
    fn test_validate_prompt_requires_content() {
        let mut workflow = create_test_workflow();
        workflow.steps[0].content = None;
        let err = validate_workflow(&workflow).unwrap_err();
        assert!(err.to_string().contains("content"));
    }

    #[test]
    fn test_validate_action_requires_cmd() {
        let mut workflow = create_test_workflow();
        workflow.steps[0] = Step {
            step_type: StepType::Action,
            description: "Action step".to_string(),
            content: None,
            cmd: None,
            path: None,
            shell: None,
            agent_type: None,
            agent_types: None,
            task: None,
            parallel: None,
        };
        let err = validate_workflow(&workflow).unwrap_err();
        assert!(err.to_string().contains("cmd"));
    }

    #[test]
    fn test_validate_write_requires_path_and_content() {
        let mut workflow = create_test_workflow();
        workflow.steps[0] = Step {
            step_type: StepType::Write,
            description: "Write step".to_string(),
            content: None,
            cmd: None,
            path: None,
            shell: None,
            agent_type: None,
            agent_types: None,
            task: None,
            parallel: None,
        };
        let err = validate_workflow(&workflow).unwrap_err();
        assert!(err.to_string().contains("path") && err.to_string().contains("content"));
    }
}
