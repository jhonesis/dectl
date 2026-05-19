use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Prompt,
    Action,
    Write,
}

impl fmt::Display for StepType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepType::Prompt => write!(f, "prompt"),
            StepType::Action => write!(f, "action"),
            StepType::Write => write!(f, "write"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "required")]
    pub is_required: bool,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Step {
    #[serde(rename = "type")]
    pub step_type: StepType,
    pub description: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub cmd: Option<Vec<String>>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub shell: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub inputs: Vec<InputDefinition>,
    pub steps: Vec<Step>,
}

impl Workflow {
    pub fn required_input_names(&self) -> HashSet<&str> {
        self.inputs
            .iter()
            .filter(|i| i.is_required)
            .map(|i| i.name.as_str())
            .collect()
    }

    pub fn optional_input_names(&self) -> HashSet<&str> {
        self.inputs
            .iter()
            .filter(|i| !i.is_required)
            .map(|i| i.name.as_str())
            .collect()
    }

    pub fn all_input_names(&self) -> HashSet<&str> {
        self.inputs.iter().map(|i| i.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_input_names() {
        let workflow = Workflow {
            name: "test".to_string(),
            description: "Test workflow".to_string(),
            inputs: vec![
                InputDefinition {
                    name: "feature_name".to_string(),
                    description: "Feature name".to_string(),
                    is_required: true,
                    default: None,
                },
                InputDefinition {
                    name: "include_tests".to_string(),
                    description: "Include tests".to_string(),
                    is_required: false,
                    default: Some("true".to_string()),
                },
            ],
            steps: vec![],
        };

        let required: HashSet<_> = workflow.required_input_names();
        assert!(required.contains("feature_name"));
        assert!(!required.contains("include_tests"));
    }
}
