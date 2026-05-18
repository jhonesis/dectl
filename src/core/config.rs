use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub core: CoreConfig,
    pub memory: MemoryConfig,
    pub workflow: WorkflowConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreConfig {
    pub default_editor: String,
    pub color: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    pub max_results: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowConfig {
    pub trust_path: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            core: CoreConfig {
                default_editor: "vim".to_string(),
                color: true,
            },
            memory: MemoryConfig { max_results: 20 },
            workflow: WorkflowConfig {
                trust_path: "~/.dectl/trust.toml".to_string(),
            },
        }
    }
}

impl GlobalConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let default_config = GlobalConfig::default();
            default_config.ensure_dir()?;
            default_config.save()?;
            eprintln!("Created default config at {:?}", config_path);
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {:?}", config_path))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {:?}", config_path))
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".dectl").join("config.toml"))
    }

    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".dectl"))
    }

    fn ensure_dir(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create config directory {:?}", dir))?;
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectDetails,
    pub stack: StackDetails,
    #[serde(default)]
    pub conventions: ProjectConventions,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectDetails {
    pub name: String,
    #[serde(default)]
    pub project_type: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StackDetails {
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
    #[serde(default)]
    pub databases: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProjectConventions {
    #[serde(default)]
    pub rules: Vec<String>,
}

impl ProjectConfig {
    pub fn load() -> Result<Option<Self>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read project config from {:?}", config_path))?;

        let config: ProjectConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse project config from {:?}", config_path))?;

        Ok(Some(config))
    }

    fn config_path() -> Result<PathBuf> {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        Ok(cwd.join(".dec").join("config").join("project.toml"))
    }
}
