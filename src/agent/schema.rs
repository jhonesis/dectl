use crate::workflow::schema::{InputDefinition, Step};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub name: String,
    pub role: String,
    pub description: String,
    #[serde(default)]
    pub context_files: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<InputDefinition>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentSource {
    Builtin,
    Custom(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent_type: String,
    pub status: AgentRunStatus,
    pub steps_executed: usize,
    pub log_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunStatus {
    Ok,
    Error { message: String },
    Timeout,
}
