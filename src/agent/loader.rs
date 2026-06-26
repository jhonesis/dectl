use std::collections::HashSet;
use std::path::PathBuf;

use crate::agent::schema::{AgentDef, AgentSource};

pub fn load_builtin(name: &str) -> Option<AgentDef> {
    let yaml_str = match name {
        "coder" => include_str!("builtins/coder.yaml"),
        "reviewer" => include_str!("builtins/reviewer.yaml"),
        "researcher" => include_str!("builtins/researcher.yaml"),
        "documenter" => include_str!("builtins/documenter.yaml"),
        _ => return None,
    };
    serde_yaml::from_str(yaml_str).ok()
}

pub fn load_custom_agents() -> Vec<(AgentDef, AgentSource)> {
    let agents_dir = PathBuf::from(".dec/agents");
    if !agents_dir.exists() {
        return Vec::new();
    }
    let mut agents = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_yaml::from_str::<AgentDef>(&content) {
                        Ok(agent) => agents.push((agent, AgentSource::Custom(path))),
                        Err(e) => {
                            log::warn!("Warning: invalid agent in {}: {}", path.display(), e)
                        }
                    },
                    Err(e) => {
                        log::warn!("Warning: could not read {}: {}", path.display(), e)
                    }
                }
            }
        }
    }
    agents
}

pub fn load_agent(name: &str) -> Option<(AgentDef, AgentSource)> {
    for (agent, source) in load_custom_agents() {
        if agent.name == name {
            return Some((agent, source));
        }
    }
    load_builtin(name).map(|a| (a, AgentSource::Builtin))
}

pub fn list_all_agents() -> Vec<(AgentDef, AgentSource)> {
    let custom = load_custom_agents();
    let custom_names: HashSet<String> = custom.iter().map(|(a, _)| a.name.clone()).collect();

    let mut all = custom;

    for name in &["coder", "reviewer", "researcher", "documenter"] {
        if !custom_names.contains(*name) {
            if let Some(agent) = load_builtin(name) {
                all.push((agent, AgentSource::Builtin));
            }
        }
    }

    all
}
