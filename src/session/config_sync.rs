use crate::project::auto_fill::{detect_stack, DetectedStack};
use crate::session::types::{ConfigDiff, ConfigSyncResult};
use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

struct TomlStack {
    languages: Vec<String>,
    frameworks: Vec<String>,
    tools: Vec<String>,
    project_type: String,
}

impl Default for TomlStack {
    fn default() -> Self {
        Self {
            languages: Vec::new(),
            frameworks: Vec::new(),
            tools: Vec::new(),
            project_type: "other".to_string(),
        }
    }
}

pub fn sync_config(dry_run: bool) -> Result<ConfigSyncResult> {
    let detected = detect_stack();
    let registered = read_project_toml()?;

    let diff = compare_stacks(&detected, &registered);

    if diff.is_empty() {
        return Ok(ConfigSyncResult {
            diff,
            toml_updated: false,
            isa_incoherent: false,
            isa_warnings: Vec::new(),
        });
    }

    let isa_warnings = check_isa_coherence(&detected)?;

    let result = if dry_run {
        ConfigSyncResult {
            diff,
            toml_updated: false,
            isa_incoherent: !isa_warnings.is_empty(),
            isa_warnings,
        }
    } else {
        merge_stack_into_toml(&diff, &registered)?;
        ConfigSyncResult {
            diff,
            toml_updated: true,
            isa_incoherent: !isa_warnings.is_empty(),
            isa_warnings,
        }
    };

    Ok(result)
}

impl ConfigDiff {
    pub fn is_empty(&self) -> bool {
        self.new_languages.is_empty()
            && self.new_frameworks.is_empty()
            && self.new_tools.is_empty()
            && self.type_changed.is_none()
    }
}

fn read_project_toml() -> Result<TomlStack> {
    let toml_path = Path::new(".dec/config/project.toml");
    if !toml_path.exists() {
        return Ok(TomlStack::default());
    }

    let content = fs::read_to_string(toml_path)?;
    let doc: toml::Value =
        toml::from_str(&content).unwrap_or(toml::Value::Table(Default::default()));

    let mut stack = TomlStack::default();

    if let Some(project) = doc.get("project") {
        if let Some(type_val) = project.get("type") {
            if let Some(s) = type_val.as_str() {
                stack.project_type = s.to_string();
            }
        }
    }

    if let Some(stack_section) = doc.get("stack") {
        if let Some(table) = stack_section.as_table() {
            if let Some(languages) = table.get("languages") {
                if let Some(arr) = languages.as_array() {
                    stack.languages = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
            if let Some(frameworks) = table.get("frameworks") {
                if let Some(arr) = frameworks.as_array() {
                    stack.frameworks = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
            if let Some(tools) = table.get("tools") {
                if let Some(arr) = tools.as_array() {
                    stack.tools = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
        }
    }

    Ok(stack)
}

fn compare_stacks(
    detected: &crate::project::auto_fill::DetectedStack,
    registered: &TomlStack,
) -> ConfigDiff {
    let mut diff = ConfigDiff::default();

    let registered_languages: HashSet<&str> =
        registered.languages.iter().map(|s| s.as_str()).collect();
    for lang in &detected.languages {
        if !registered_languages.contains(lang.as_str()) {
            diff.new_languages.push(lang.clone());
        }
    }

    let registered_frameworks: HashSet<&str> =
        registered.frameworks.iter().map(|s| s.as_str()).collect();
    for fw in &detected.frameworks {
        if !registered_frameworks.contains(fw.as_str()) {
            diff.new_frameworks.push(fw.clone());
        }
    }

    let registered_tools: HashSet<&str> = registered.tools.iter().map(|s| s.as_str()).collect();
    for tool in &detected.tools {
        if !registered_tools.contains(tool.as_str()) {
            diff.new_tools.push(tool.clone());
        }
    }

    if detected.project_type != registered.project_type
        && detected.project_type != "other"
        && registered.project_type != "other"
    {
        diff.type_changed = Some((
            registered.project_type.clone(),
            detected.project_type.clone(),
        ));
    }

    diff
}

fn merge_stack_into_toml(diff: &ConfigDiff, registered: &TomlStack) -> Result<()> {
    let toml_path = Path::new(".dec/config/project.toml");
    let content = if toml_path.exists() {
        fs::read_to_string(toml_path)?
    } else {
        return Ok(());
    };

    let mut doc: toml::map::Map<String, toml::Value> =
        toml::from_str(&content).unwrap_or_else(|_| toml::map::Map::new());

    // Merge languages
    if !diff.new_languages.is_empty() {
        let mut existing: Vec<String> = registered.languages.clone();
        for new in &diff.new_languages {
            if !existing.contains(new) {
                existing.push(new.clone());
            }
        }
        existing.sort();

        if let Some(stack_val) = doc.get_mut("stack") {
            if let Some(stack_table) = stack_val.as_table_mut() {
                stack_table.insert(
                    "languages".to_string(),
                    toml::Value::Array(existing.into_iter().map(toml::Value::String).collect()),
                );
            }
        } else {
            let mut stack_map = toml::map::Map::new();
            stack_map.insert(
                "languages".to_string(),
                toml::Value::Array(existing.into_iter().map(toml::Value::String).collect()),
            );
            doc.insert("stack".to_string(), toml::Value::Table(stack_map));
        }
    }

    // Merge frameworks
    if !diff.new_frameworks.is_empty() {
        let mut existing: Vec<String> = registered.frameworks.clone();
        for new in &diff.new_frameworks {
            if !existing.contains(new) {
                existing.push(new.clone());
            }
        }
        existing.sort();

        if let Some(stack_val) = doc.get_mut("stack") {
            if let Some(stack_table) = stack_val.as_table_mut() {
                stack_table.insert(
                    "frameworks".to_string(),
                    toml::Value::Array(existing.into_iter().map(toml::Value::String).collect()),
                );
            }
        }
    }

    // Merge tools
    if !diff.new_tools.is_empty() {
        let mut existing: Vec<String> = registered.tools.clone();
        for new in &diff.new_tools {
            if !existing.contains(new) {
                existing.push(new.clone());
            }
        }
        existing.sort();

        if let Some(stack_val) = doc.get_mut("stack") {
            if let Some(stack_table) = stack_val.as_table_mut() {
                stack_table.insert(
                    "tools".to_string(),
                    toml::Value::Array(existing.into_iter().map(toml::Value::String).collect()),
                );
            }
        }
    }

    // Update project type if changed
    if let Some((_, new_type)) = &diff.type_changed {
        if let Some(project) = doc.get_mut("project") {
            if let Some(map) = project.as_table_mut() {
                map.insert("type".to_string(), toml::Value::String(new_type.clone()));
            }
        }
    }

    let new_content = toml::to_string_pretty(&doc)?;
    fs::write(toml_path, new_content)?;

    Ok(())
}

fn check_isa_coherence(detected: &DetectedStack) -> Result<Vec<String>> {
    let isa_path = Path::new(".dec/isa/project.isa.md");
    if !isa_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(isa_path)?;
    let mut warnings = Vec::new();

    for lang in &detected.languages {
        let lang_lower = lang.to_lowercase();
        if !content.to_lowercase().contains(&lang_lower) {
            warnings.push(format!(
                "Tech Stack section may be outdated: detected '{}' but not mentioned in project.isa.md",
                lang
            ));
        }
    }

    for fw in &detected.frameworks {
        let fw_lower = fw.to_lowercase();
        if !content.to_lowercase().contains(&fw_lower) {
            warnings.push(format!(
                "Framework '{}' detected but not mentioned in project.isa.md",
                fw
            ));
        }
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::auto_fill::DetectedStack;

    fn make_detected(
        languages: Vec<&str>,
        frameworks: Vec<&str>,
        tools: Vec<&str>,
        project_type: &str,
    ) -> DetectedStack {
        DetectedStack {
            name: None,
            project_type: project_type.to_string(),
            languages: languages.into_iter().map(String::from).collect(),
            frameworks: frameworks.into_iter().map(String::from).collect(),
            tools: tools.into_iter().map(String::from).collect(),
            has_env_config: false,
            has_docker: false,
            has_linting: false,
        }
    }

    fn make_registered(
        languages: Vec<&str>,
        frameworks: Vec<&str>,
        tools: Vec<&str>,
        project_type: &str,
    ) -> TomlStack {
        TomlStack {
            languages: languages.into_iter().map(String::from).collect(),
            frameworks: frameworks.into_iter().map(String::from).collect(),
            tools: tools.into_iter().map(String::from).collect(),
            project_type: project_type.to_string(),
        }
    }

    #[test]
    fn test_compare_stacks_no_diff() {
        let detected = make_detected(vec!["rust"], vec!["actix"], vec!["cargo"], "cargo");
        let registered = make_registered(vec!["rust"], vec!["actix"], vec!["cargo"], "cargo");
        let diff = compare_stacks(&detected, &registered);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_compare_stacks_new_language() {
        let detected = make_detected(vec!["javascript", "rust"], vec![], vec![], "cargo");
        let registered = make_registered(vec!["javascript"], vec![], vec![], "npm");
        let diff = compare_stacks(&detected, &registered);
        assert_eq!(diff.new_languages, vec!["rust"]);
        assert!(diff.new_frameworks.is_empty());
    }

    #[test]
    fn test_compare_stacks_new_tool() {
        let detected = make_detected(vec!["rust"], vec![], vec!["docker", "cargo"], "cargo");
        let registered = make_registered(vec!["rust"], vec![], vec!["cargo"], "cargo");
        let diff = compare_stacks(&detected, &registered);
        assert_eq!(diff.new_tools, vec!["docker"]);
    }

    #[test]
    fn test_compare_stacks_type_changed() {
        let detected = make_detected(vec!["rust"], vec![], vec![], "cargo");
        let registered = make_registered(vec!["javascript"], vec!["node"], vec!["npm"], "npm");
        let diff = compare_stacks(&detected, &registered);
        assert_eq!(
            diff.type_changed,
            Some(("npm".to_string(), "cargo".to_string()))
        );
    }

    #[test]
    fn test_compare_stacks_empty_registered() {
        let detected = make_detected(
            vec!["rust", "python"],
            vec!["actix"],
            vec!["docker"],
            "cargo",
        );
        let registered = TomlStack::default();
        let diff = compare_stacks(&detected, &registered);
        assert_eq!(diff.new_languages.len(), 2);
        assert_eq!(diff.new_frameworks, vec!["actix"]);
        assert_eq!(diff.new_tools, vec!["docker"]);
    }
}
