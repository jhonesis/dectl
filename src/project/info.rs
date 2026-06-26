use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

use crate::core::config::ProjectConfig;
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct ProjectInfoOutput {
    pub name: Option<String>,
    pub project_type: Option<String>,
    pub description: Option<String>,
    pub stack: StackInfo,
    pub conventions: Vec<String>,
    pub warnings: Vec<String>,
    pub isa: Option<IsaExcerpt>,
}

#[derive(Debug, Serialize)]
pub struct StackInfo {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IsaExcerpt {
    pub vision: Option<String>,
    pub objective: Option<String>,
    pub path: String,
}

pub fn run(mode: OutputMode) -> Result<()> {
    let mut warnings = Vec::new();
    let mut stack = StackInfo {
        languages: Vec::new(),
        frameworks: Vec::new(),
        databases: Vec::new(),
        tools: Vec::new(),
    };
    let mut conventions = Vec::new();
    let mut name = None;
    let mut project_type = None;
    let mut description = None;
    let mut isa = None;

    match ProjectConfig::load()? {
        Some(config) => {
            name = Some(config.project.name.clone());
            project_type = Some(config.project.project_type.clone());
            description = Some(config.project.description.clone());

            stack.languages = config.stack.languages.clone();
            stack.frameworks = config.stack.frameworks.clone();
            stack.databases = config.stack.databases.clone();
            stack.tools = config.stack.tools.clone();

            conventions = config.conventions.rules.clone();
        }
        None => {
            warnings.push("Missing .dec/config/project.toml".to_string());
        }
    }

    let isa_path = PathBuf::from(".dec/isa/project.isa.md");
    if isa_path.exists() {
        match extract_isa_excerpt(&isa_path) {
            Ok(excerpt) => {
                isa = Some(excerpt);
            }
            Err(e) => {
                warnings.push(format!("Failed to parse .dec/isa/project.isa.md: {}", e));
            }
        }
    } else {
        warnings.push("Missing .dec/isa/project.isa.md".to_string());
    }

    let output = ProjectInfoOutput {
        name,
        project_type,
        description,
        stack,
        conventions,
        warnings,
        isa,
    };

    mode.print(&output)?;

    Ok(())
}

fn extract_isa_excerpt(path: &PathBuf) -> Result<IsaExcerpt> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;

    let vision =
        extract_section(&content, "## Visión").or_else(|| extract_section(&content, "## Vision"));
    let objective = extract_section(&content, "## Objetivo Principal")
        .or_else(|| extract_section(&content, "## Objetivo"));

    Ok(IsaExcerpt {
        vision,
        objective,
        path: path.to_string_lossy().to_string(),
    })
}

fn extract_section(content: &str, header: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == header {
            let mut result = Vec::new();
            for next_line in lines.iter().skip(i + 1) {
                let trimmed = next_line.trim();
                if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                    break;
                }
                if !trimmed.is_empty() && !trimmed.starts_with("<!--") {
                    result.push(trimmed.to_string());
                }
            }
            if result.is_empty() {
                return Some(String::new());
            }
            let text = result.join(" ").trim().to_string();
            if text.is_empty() || text.starts_with('(') {
                return None;
            }
            return Some(text);
        }
    }
    None
}
