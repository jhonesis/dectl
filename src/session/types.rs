use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub date: String,
    pub actions: Vec<String>,
    pub pending: Vec<String>,
    pub decisions: Vec<String>,
    pub next_step: String,
}

impl Default for SessionSummary {
    fn default() -> Self {
        SessionSummary {
            date: chrono::Utc::now().to_rfc3339(),
            actions: Vec::new(),
            pending: Vec::new(),
            decisions: Vec::new(),
            next_step: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitChanges {
    pub modified_files: Vec<(String, String)>,
    pub new_commits: Vec<String>,
    pub detected_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedDecision {
    pub text: String,
    pub tags: Vec<String>,
    pub already_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub name: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndResult {
    pub steps: Vec<StepResult>,
    pub decisions_saved: usize,
}

impl SessionEndResult {
    pub fn new() -> Self {
        SessionEndResult {
            steps: Vec::new(),
            decisions_saved: 0,
        }
    }

    pub fn add_step(&mut self, name: &str, success: bool, message: &str) {
        self.steps.push(StepResult {
            name: name.to_string(),
            success,
            message: message.to_string(),
        });
    }

    pub fn any_success(&self) -> bool {
        self.steps.iter().any(|s| s.success)
    }
}
