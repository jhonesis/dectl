use crate::workflow::schema::{InputDefinition, Step};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub name: String,
    pub role: String,
    pub description: String,
    /// Optional list of predecessor agents (informational only, does not block execution).
    /// Use to document the recommended pipeline order. Agents are fault-tolerant
    /// and will execute even if predecessors fail.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Context files to automatically load and pass as variables.
    /// Each file is read and made available as {{context_<filename>}} in steps.
    /// Example: ".dec/config/project.toml" → {{context_dec_config_project_toml}}
    #[serde(default)]
    pub context_files: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<InputDefinition>,
    #[serde(default)]
    pub next_step_hint: Option<String>,
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
