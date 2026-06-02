use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const TRUST_FILE: &str = "trust.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEntry {
    #[serde(rename = "project_path")]
    pub project_path: String,
    #[serde(rename = "workflow_name")]
    pub workflow_name: String,
    #[serde(rename = "trusted_at")]
    pub trusted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrustConfig {
    #[serde(rename = "trusted", default)]
    pub trusted_entries: Vec<TrustEntry>,
}

impl TrustConfig {
    fn config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_default();
        path.push(".dectl");
        fs::create_dir_all(&path).ok();
        path.push(TRUST_FILE);
        path
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(TrustConfig::default());
        }

        let content =
            fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;

        toml::from_str(&content).with_context(|| format!("Failed to parse {:?}", path))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        let content =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize trust config")?;

        fs::write(&path, content).with_context(|| format!("Failed to write {:?}", path))?;

        Ok(())
    }

    pub fn is_trusted(&self, project_path: &str, workflow_name: &str) -> bool {
        self.trusted_entries
            .iter()
            .any(|e| e.project_path == project_path && e.workflow_name == workflow_name)
    }

    pub fn trust(&mut self, project_path: String, workflow_name: String) {
        if !self.is_trusted(&project_path, &workflow_name) {
            self.trusted_entries.push(TrustEntry {
                project_path,
                workflow_name,
                trusted_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    #[allow(dead_code)]
    pub fn untrust(&mut self, project_path: &str, workflow_name: &str) {
        self.trusted_entries
            .retain(|e| !(e.project_path == project_path && e.workflow_name == workflow_name));
    }
}

pub fn check_trust(
    project_path: &str,
    workflow_name: &str,
    has_action_steps: bool,
    non_interactive: bool,
) -> Result<TrustDecision> {
    if !has_action_steps {
        return Ok(TrustDecision::Trusted);
    }

    let config = TrustConfig::load()?;

    if config.is_trusted(project_path, workflow_name) {
        return Ok(TrustDecision::Trusted);
    }

    if non_interactive {
        return Ok(TrustDecision::RequiresConfirmation);
    }

    Ok(TrustDecision::AskUser)
}

pub fn grant_trust(project_path: &str, workflow_name: &str) -> Result<()> {
    let mut config = TrustConfig::load()?;
    config.trust(project_path.to_string(), workflow_name.to_string());
    config.save()
}

#[derive(Debug, Clone)]
pub enum TrustDecision {
    Trusted,
    AskUser,
    RequiresConfirmation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_config_default() {
        let config = TrustConfig::default();
        assert!(config.trusted_entries.is_empty());
    }

    #[test]
    fn test_trust_and_check() {
        let mut config = TrustConfig::default();
        config.trust("/test/project".to_string(), "test_workflow".to_string());

        assert!(config.is_trusted("/test/project", "test_workflow"));
        assert!(!config.is_trusted("/test/project", "other_workflow"));
        assert!(!config.is_trusted("/other/project", "test_workflow"));
    }

    #[test]
    fn test_untrust() {
        let mut config = TrustConfig::default();
        config.trust("/test/project".to_string(), "test_workflow".to_string());
        assert!(config.is_trusted("/test/project", "test_workflow"));

        config.untrust("/test/project", "test_workflow");
        assert!(!config.is_trusted("/test/project", "test_workflow"));
    }

    #[test]
    fn test_no_duplicate_trust() {
        let mut config = TrustConfig::default();
        config.trust("/test/project".to_string(), "test_workflow".to_string());
        config.trust("/test/project".to_string(), "test_workflow".to_string());

        assert_eq!(config.trusted_entries.len(), 1);
    }
}
