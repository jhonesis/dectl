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
    pub objectives: Option<String>,
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

    if let Some((name, desc)) =
        read_readme_context("README.md").or_else(|| read_readme_context("README.MD"))
    {
        ctx.name = ctx.name.or(name);
        ctx.description = ctx.description.or(desc);
    }

    if let Some((name, desc)) =
        read_readme_context("docs/README.md").or_else(|| read_readme_context("docs/README.MD"))
    {
        ctx.name = ctx.name.or(name);
        ctx.description = ctx.description.or(desc);
    }

    for entry in glob_readme_entries().unwrap_or_default() {
        let first_lines = read_file_lines(&entry, 20);
        let keywords = extract_keywords_from_lines(&first_lines);

        if keywords.vision.is_some() && ctx.vision.is_none() {
            ctx.vision = keywords.vision;
        }
        if keywords.objectives.is_some() && ctx.objectives.is_none() {
            ctx.objectives = keywords.objectives;
        }
    }

    if let Some(spec_content) = read_file_prefix("SPEC.md", 500)
        .or_else(|| read_file_prefix("spec.md", 500))
        .or_else(|| read_file_prefix("specs/master/spec.md", 500))
    {
        if ctx.description.is_none() {
            ctx.description = extract_description_from_spec(&spec_content);
        }
    }

    ctx
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
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "mi-proyecto".to_string())
        });

    update_project_toml(project_dir, &project_name, stack)?;
    update_project_isa(project_dir, &project_name, context)?;

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

fn read_file_lines(path: &str, max_lines: usize) -> Vec<String> {
    let full_path = Path::new(path);
    if !full_path.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(full_path).ok();
    let content = match content {
        Some(c) => c,
        None => return Vec::new(),
    };

    content
        .lines()
        .take(max_lines)
        .map(|s| s.to_string())
        .collect()
}

fn glob_readme_entries() -> Option<Vec<String>> {
    let walker = WalkBuilder::new(Path::new("docs"))
        .max_depth(Some(2))
        .hidden(false)
        .build();

    let mut entries = Vec::new();
    for entry in walker.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_string_lossy();
        if name.ends_with(".md") && name.to_lowercase().starts_with("readme") {
            entries.push(path.to_string_lossy().to_string());
        }
    }
    Some(entries)
}

fn read_readme_context(path: &str) -> Option<(Option<String>, Option<String>)> {
    let full_path = Path::new(path);
    if !full_path.exists() {
        return None;
    }

    let content = fs::read_to_string(full_path).ok()?;
    let content = content.chars().take(500).collect::<String>();

    let mut name = None;
    let mut description = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            name = Some(stripped.trim().to_string());
            break;
        }
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("```") {
            description = Some(trimmed.to_string());
            break;
        }
    }

    Some((name, description))
}

fn extract_keywords_from_lines(lines: &[String]) -> OptionalContext {
    let mut ctx = OptionalContext::default();
    let content = lines.join(" ");

    let vision_keywords = ["vision", "visi", "propósito", "qué es"];
    let objectives_keywords = ["objective", "objetivo", "meta", "meta"];

    for keyword in vision_keywords {
        if content.to_lowercase().contains(keyword) {
            ctx.vision = lines
                .iter()
                .find(|l| l.to_lowercase().contains(keyword))
                .map(|s| s.trim().to_string());
            break;
        }
    }

    for keyword in objectives_keywords {
        if content.to_lowercase().contains(keyword) {
            ctx.objectives = lines
                .iter()
                .find(|l| l.to_lowercase().contains(keyword))
                .map(|s| s.trim().to_string());
            break;
        }
    }

    ctx
}

fn extract_description_from_spec(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with("```")
            && trimmed.len() > 20
        {
            return Some(trimmed.to_string());
        }
    }
    None
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

    if !stack.languages.is_empty() || !stack.frameworks.is_empty() || !stack.tools.is_empty() {
        let mut stack_map = toml::map::Map::new();

        if !stack.languages.is_empty() {
            stack_map.insert(
                "languages".to_string(),
                toml::Value::Array(
                    stack
                        .languages
                        .iter()
                        .map(|s| toml::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }

        if !stack.frameworks.is_empty() {
            stack_map.insert(
                "frameworks".to_string(),
                toml::Value::Array(
                    stack
                        .frameworks
                        .iter()
                        .map(|s| toml::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }

        if !stack.tools.is_empty() {
            stack_map.insert(
                "tools".to_string(),
                toml::Value::Array(
                    stack
                        .tools
                        .iter()
                        .map(|s| toml::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }

        doc.insert("stack".to_string(), toml::Value::Table(stack_map));
    }

    let new_content = toml::to_string_pretty(&doc)?;
    fs::write(&toml_path, new_content)?;

    Ok(())
}

fn update_project_isa(project_dir: &Path, name: &str, context: &OptionalContext) -> Result<()> {
    use super::templates::Templates;

    let isa_path = project_dir.join(".dec/isa/project.isa.md");

    let template = if isa_path.exists() {
        fs::read_to_string(&isa_path)?
    } else {
        Templates::project_isa().to_string()
    };

    let new_content = if context.vision.is_some() || context.objectives.is_some() {
        let mut content = template.replace("[Nombre del Proyecto]", name);

        if let Some(ref vision) = context.vision {
            content = fill_section(&content, "## Visión", vision);
        }

        if let Some(ref objectives) = context.objectives {
            content = fill_section(&content, "## Objetivo Principal", objectives);
        }

        content
    } else {
        template.replace("[Nombre del Proyecto]", name)
    };

    fs::write(&isa_path, new_content)?;

    Ok(())
}

fn fill_section(content: &str, section_header: &str, value: &str) -> String {
    let mut result = content.to_string();

    if let Some(pos) = result.find(section_header) {
        let after_header = &result[pos..];
        if let Some(next_newline) = after_header[1..].find('\n') {
            let start = pos + 1 + next_newline;
            let after = &result[start..];

            let replacement = if after.trim().starts_with("<!--") || after.trim().is_empty() {
                format!(" {}", value)
            } else {
                let indent = " ".repeat(section_header.len() - 3);
                format!("\n{}\n{}", indent, value)
            };

            result = format!("{}{}", &result[..start], replacement);
        }
    }

    result
}
