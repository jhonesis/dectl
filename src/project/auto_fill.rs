use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct DetectedStack {
    pub name: Option<String>,
    pub project_type: String,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub tools: Vec<String>,
    pub has_env_config: bool,
    pub has_docker: bool,
    pub has_linting: bool,
}

impl Default for DetectedStack {
    fn default() -> Self {
        Self {
            name: None,
            project_type: "other".to_string(),
            languages: Vec::new(),
            frameworks: Vec::new(),
            tools: Vec::new(),
            has_env_config: false,
            has_docker: false,
            has_linting: false,
        }
    }
}

#[derive(Default)]
pub struct OptionalContext {
    pub name: Option<String>,
    pub description: Option<String>,
    pub vision: Option<String>,
}

pub fn is_project_empty() -> bool {
    let exclude_dirs: HashSet<&str> = [
        ".dec",
        "target",
        "node_modules",
        ".git",
        "vendor",
        "bin",
        "obj",
        ".venv",
        "venv",
        "__pycache__",
        ".svn",
        ".hg",
        ".idea",
        ".vscode",
    ]
    .into_iter()
    .collect();

    let exclude_files: HashSet<&str> = [
        "AGENTS.md",
        "CLAUDE.md",
        "AGENTS.MD",
        "CLAUDE.MD",
        ".gitignore",
    ]
    .into_iter()
    .collect();

    let mut count = 0;

    let walker = WalkBuilder::new(Path::new("."))
        .hidden(false)
        .filter_entry(move |entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy();

            if exclude_files.contains(name.as_ref()) {
                return false;
            }

            for component in path.components() {
                let comp_str = component.as_os_str().to_string_lossy();
                if exclude_dirs.contains(comp_str.as_ref()) {
                    return false;
                }
            }

            if name.starts_with('.') && name.as_ref() != ".dec" {
                return false;
            }

            true
        })
        .max_depth(Some(5))
        .build();

    for entry in walker.flatten() {
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            count += 1;
            if count > 0 {
                break;
            }
        }
    }

    count == 0
}

pub fn detect_stack() -> DetectedStack {
    let mut stack = DetectedStack::default();

    if let Some(name) = read_json_field("package.json", "name")
        .or_else(|| read_toml_field("Cargo.toml", "package", Some("name")))
        .or_else(|| read_toml_field("go.mod", "module", None))
        .or_else(|| read_toml_field("pyproject.toml", "project", Some("name")))
    {
        stack.name = Some(name);
    }

    if Path::new("package.json").exists() {
        let has_scripts = read_json_field("package.json", "scripts")
            .map(|s| !s.is_empty() && s != "{}")
            .unwrap_or(false);

        stack.project_type = "npm".to_string();
        stack.languages.push("javascript".to_string());

        if has_scripts {
            stack.frameworks.push("node".to_string());
        }

        if Path::new("package-lock.json").exists() || Path::new("yarn.lock").exists() {
            stack.tools.push("npm".to_string());
        }
    }

    if Path::new("Cargo.toml").exists() {
        stack.project_type = "cargo".to_string();
        stack.languages.push("rust".to_string());

        if Path::new("rust-toolchain.toml").exists() {
            stack.tools.push("rust-toolchain".to_string());
        }
    }

    if Path::new("go.mod").exists() {
        stack.project_type = "go".to_string();
        stack.languages.push("go".to_string());
    }

    if Path::new("requirements.txt").exists() {
        stack.project_type = "pip".to_string();
        stack.languages.push("python".to_string());
        if let Some(content) = read_file_prefix("requirements.txt", 2000) {
            let lower = content.to_lowercase();
            if lower.contains("fastapi") {
                stack.frameworks.push("fastapi".to_string());
            }
            if lower.contains("django") {
                stack.frameworks.push("django".to_string());
            }
            if lower.contains("flask") {
                stack.frameworks.push("flask".to_string());
            }
            if lower.contains("aiortc") {
                stack.frameworks.push("aiortc".to_string());
            }
            if lower.contains("uvicorn") {
                stack.tools.push("uvicorn".to_string());
            }
        }
    }

    if Path::new("pyproject.toml").exists() {
        if stack.project_type == "other" {
            stack.project_type = "python".to_string();
        }
        if !stack.languages.contains(&"python".to_string()) {
            stack.languages.push("python".to_string());
        }
    }

    if Path::new("pom.xml").exists() {
        stack.project_type = "maven".to_string();
        stack.languages.push("java".to_string());
    }

    if Path::new("build.gradle").exists() || Path::new("build.gradle.kts").exists() {
        stack.project_type = "gradle".to_string();
        stack.languages.push("java".to_string());
    }

    if Path::new("composer.json").exists() {
        stack.project_type = "composer".to_string();
        stack.languages.push("php".to_string());
    }

    if Path::new("Gemfile").exists() {
        stack.project_type = "bundler".to_string();
        stack.languages.push("ruby".to_string());
    }

    stack.has_env_config = Path::new(".env.example").exists()
        || Path::new(".env.template").exists()
        || Path::new(".env.sample").exists();

    stack.has_docker = Path::new("docker-compose.yml").exists()
        || Path::new("docker-compose.yaml").exists()
        || Path::new("Dockerfile").exists();

    stack.has_linting = Path::new(".eslintrc").exists()
        || Path::new(".eslintrc.js").exists()
        || Path::new(".eslintrc.json").exists()
        || Path::new(".eslintrc.yaml").exists()
        || Path::new(".eslintrc.yml").exists()
        || Path::new("eslint.config.js").exists()
        || Path::new("eslint.config.mjs").exists()
        || Path::new("eslint.config.cjs").exists();

    stack.languages.sort();
    stack.languages.dedup();
    stack.frameworks.sort();
    stack.frameworks.dedup();
    stack.tools.sort();
    stack.tools.dedup();

    stack
}

pub fn scan_docs_for_context() -> OptionalContext {
    let mut ctx = OptionalContext::default();

    // Minimal: only try to read project name from README title as basic fallback
    if let Some(content) = read_file_prefix("README.md", 200) {
        for line in content.lines() {
            if let Some(stripped) = line.trim().strip_prefix("# ") {
                ctx.name = Some(stripped.trim().to_string());
                break;
            }
        }
    }

    ctx
}

pub fn generate_auto_fill_prompt(project_dir: &Path) -> Result<()> {
    use crate::project::templates::Templates;

    let prompt_dir = project_dir.join(".dec/prompts/tasks");
    fs::create_dir_all(&prompt_dir)?;
    let prompt_path = prompt_dir.join("auto-fill.md");
    fs::write(&prompt_path, Templates::auto_fill_task())?;
    Ok(())
}

pub fn fill_project_files(
    stack: &DetectedStack,
    context: &OptionalContext,
    project_dir: &Path,
) -> Result<()> {
    let project_name = stack
        .name
        .clone()
        .or_else(|| context.name.clone())
        .unwrap_or_else(|| {
            project_dir
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
                .or_else(|| {
                    std::env::current_dir()
                        .ok()
                        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
                })
                .unwrap_or_else(|| "mi-proyecto".to_string())
        });

    update_project_toml(project_dir, &project_name, stack)?;
    update_project_isa(project_dir, &project_name)?;
    generate_auto_fill_prompt(project_dir)?;

    Ok(())
}

fn update_project_toml(project_dir: &Path, name: &str, stack: &DetectedStack) -> Result<()> {
    use super::templates::Templates;

    let toml_path = project_dir.join(".dec/config/project.toml");

    let content = if toml_path.exists() {
        fs::read_to_string(&toml_path)?
    } else {
        Templates::project_toml_l1().to_string()
    };

    let mut doc: toml::map::Map<String, toml::Value> =
        toml::from_str(&content).unwrap_or_else(|_| toml::map::Map::new());

    if let Some(project) = doc.get_mut("project") {
        if let Some(map) = project.as_table_mut() {
            if let Some(n) = map.get_mut("name") {
                *n = toml::Value::String(name.to_string());
            }
            if let Some(t) = map.get_mut("type") {
                *t = toml::Value::String(stack.project_type.clone());
            }
        }
    } else {
        let mut project_map = toml::map::Map::new();
        project_map.insert("name".to_string(), toml::Value::String(name.to_string()));
        project_map.insert(
            "type".to_string(),
            toml::Value::String(stack.project_type.clone()),
        );
        doc.insert("project".to_string(), toml::Value::Table(project_map));
    }

    // Merge languages into existing stack, preserving frameworks, tools, databases
    if !stack.languages.is_empty() {
        let existing_languages: Vec<String> = doc
            .get("stack")
            .and_then(|s| s.get("languages"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut merged: Vec<String> = existing_languages.clone();
        for lang in &stack.languages {
            if !merged.contains(lang) {
                merged.push(lang.clone());
            }
        }
        merged.sort();

        let stack_entry = doc
            .entry("stack".to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

        if let Some(stack_table) = stack_entry.as_table_mut() {
            stack_table.insert(
                "languages".to_string(),
                toml::Value::Array(merged.into_iter().map(toml::Value::String).collect()),
            );
        }
    }

    let new_content = toml::to_string_pretty(&doc)?;
    fs::write(&toml_path, new_content)?;

    Ok(())
}

fn update_project_isa(project_dir: &Path, name: &str) -> Result<()> {
    use super::templates::Templates;

    let isa_path = project_dir.join(".dec/isa/project.isa.md");

    let template = if isa_path.exists() {
        fs::read_to_string(&isa_path)?
    } else {
        Templates::project_isa().to_string()
    };

    let new_content = template.replace("[Project Name]", name);

    fs::write(&isa_path, new_content)?;

    Ok(())
}

fn read_json_field(file_path: &str, field: &str) -> Option<String> {
    let content = read_file_prefix(file_path, 5000)?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let value = json.get(field)?.as_str()?.to_string();
    Some(value)
}

fn read_toml_field(file_path: &str, section: &str, field: Option<&str>) -> Option<String> {
    let content = read_file_prefix(file_path, 5000)?;
    let toml_val: toml::Value = toml::from_str(&content).ok()?;

    let section_val = toml_val.get(section)?;

    let value = if let Some(f) = field {
        section_val.get(f)?.as_str()?.to_string()
    } else {
        section_val.as_str()?.to_string()
    };

    Some(value)
}

fn read_file_prefix(path: &str, max_chars: usize) -> Option<String> {
    let full_path = Path::new(path);
    if !full_path.exists() {
        return None;
    }

    let content = fs::read_to_string(full_path).ok()?;
    if content.len() <= max_chars {
        Some(content)
    } else {
        Some(content.chars().take(max_chars).collect())
    }
}
