use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::bail_app_err;
use crate::project::templates::Templates;
use crate::spec::add::ExtractedReqs;

static ROOT_SPEC_REQ: &str = r#"
### REQ-{NUMBER}: [{MODULE_NAME}] Module
**User Story**:
> As a developer, I want a {MODULE_NAME} module so that {DESCRIPTION}

**Implementation Notes**: Implemented in specs/{MODULE_NAME}/

---
"#;

static ROOT_TASK: &str = r#"- [ ] [T{NUMBER}] [Module] Implement {MODULE_NAME} module — L (REQ-{REQ_NUM})
  **Build**: `[build command]`
  **Verify**: specs/{MODULE_NAME}/ tests pass
  **Gate**: must pass before next task
"#;

pub fn run(name: &str, reqs: &ExtractedReqs) -> Result<()> {
    let module_dir = Path::new("specs").join(name);

    if module_dir.exists() {
        bail_app_err!(
            format!("Module directory already exists: specs/{}/", name),
            "Choose a different name or remove the existing directory"
        );
    }

    fs::create_dir_all(&module_dir).context("Failed to create module directory")?;

    let template_files = Templates::module_spec_files();

    for (filename, template_content) in &template_files {
        let file_path = module_dir.join(filename);
        let content = match *filename {
            "spec.md" => render_spec_template(template_content, reqs),
            _ => render_simple_template(template_content, name),
        };
        fs::write(&file_path, content.as_bytes())
            .with_context(|| format!("Failed to write {}", file_path.display()))?;
    }

    append_to_root_spec(name, reqs)?;
    append_to_root_tasks(name, reqs)?;

    println!(
        "✅ Created module '{}' at specs/{}/ ({} REQ(s))",
        name,
        name,
        reqs.requirements.len()
    );
    Ok(())
}

fn render_simple_template(template: &str, name: &str) -> String {
    template.replace("[MODULE_NAME]", name)
}

fn render_spec_template(template: &str, reqs: &ExtractedReqs) -> String {
    let module_prefix = reqs.name.to_uppercase().replace('-', "_");
    let content = template
        .replace("[MODULE_NAME]", &reqs.name)
        .replace("[MODULE]", &module_prefix);

    let start_marker = "<!--REQ_START-->";
    let end_marker = "<!--REQ_END-->";

    let start_pos = match content.find(start_marker) {
        Some(p) => p,
        None => return content,
    };
    let end_pos = match content.find(end_marker) {
        Some(p) => p + end_marker.len(),
        None => return content,
    };

    let before = &content[..start_pos];
    let after = &content[end_pos..];

    if reqs.requirements.is_empty() {
        if let Some(empty_marker) = after.find("## ") {
            return format!("{}{}", before, &after[empty_marker..]);
        }
        return format!("{}{}", before, after);
    }

    let mut reqs_block = String::new();
    for (i, req) in reqs.requirements.iter().enumerate() {
        let number = i + 1;
        let criteria_lines = if req.acceptance_criteria.is_empty() {
            "  - WHEN (condition) THEN the system SHALL (behavior)".to_string()
        } else {
            req.acceptance_criteria
                .iter()
                .map(|c| format!("  {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let user_story = if req.user_story.is_empty() {
            format!(
                "As a developer, I want {} so that it works correctly.",
                req.title.to_lowercase()
            )
        } else {
            req.user_story.clone()
        };
        let block = format!(
            r#"### REQ-{prefix}-{num:03}: {title}
**User Story**:
> {story}

**Acceptance Criteria**:
{criteria}

**Implementation Notes**: This requirement SHALL be implemented as 2–3 atomic tasks.

---
"#,
            prefix = module_prefix,
            num = number,
            title = req.title,
            story = user_story,
            criteria = criteria_lines,
        );
        reqs_block.push_str(&block);
    }

    format!("{}{}{}", before, reqs_block, after)
}

fn append_to_root_spec(name: &str, reqs: &ExtractedReqs) -> Result<()> {
    let spec_path = Path::new("specs").join("spec.md");
    let mut content = if spec_path.exists() {
        fs::read_to_string(&spec_path).context("Failed to read root spec.md")?
    } else {
        String::new()
    };

    let next_num = if content.is_empty() {
        1
    } else {
        find_last_req_number(&content) + 1
    };

    let description = if reqs.description.is_empty() {
        format!("add {} functionality to the project", name)
    } else {
        reqs.description.clone()
    };

    let root_entry = ROOT_SPEC_REQ
        .replace("{NUMBER}", &format!("{:03}", next_num))
        .replace("{MODULE_NAME}", name)
        .replace("{DESCRIPTION}", &description);

    content.push_str(&root_entry);
    fs::write(&spec_path, content.as_bytes()).context("Failed to write root spec.md")?;
    Ok(())
}

fn append_to_root_tasks(name: &str, reqs: &ExtractedReqs) -> Result<()> {
    let tasks_path = Path::new("specs").join("tasks.md");
    let mut content = if tasks_path.exists() {
        fs::read_to_string(&tasks_path).context("Failed to read root tasks.md")?
    } else {
        String::new()
    };

    let next_num = if content.is_empty() {
        1
    } else {
        find_last_task_number(&content) + 1
    };

    let req_num = if reqs.requirements.is_empty() {
        next_num
    } else {
        let last_req = &reqs.requirements[reqs.requirements.len() - 1];
        last_req
            .id
            .rsplit('-')
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(next_num)
    };

    let root_task = ROOT_TASK
        .replace("{NUMBER}", &format!("{:03}", next_num))
        .replace("{MODULE_NAME}", name)
        .replace("{REQ_NUM}", &format!("{:03}", req_num));

    content.push_str(&root_task);
    fs::write(&tasks_path, content.as_bytes()).context("Failed to write root tasks.md")?;
    Ok(())
}

fn find_last_req_number(content: &str) -> usize {
    let re = regex::Regex::new(r"REQ-(?:\w+-)?(\d+)").unwrap();
    re.captures_iter(content)
        .filter_map(|c| c.get(1)?.as_str().parse::<usize>().ok())
        .max()
        .unwrap_or(0)
}

fn find_last_task_number(content: &str) -> usize {
    let re = regex::Regex::new(r"\[T(\d+)\]").unwrap();
    re.captures_iter(content)
        .filter_map(|c| c.get(1)?.as_str().parse::<usize>().ok())
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::add::Requirement;

    #[test]
    fn test_render_simple_template() {
        let template = "# Constitution: [MODULE_NAME]";
        let result = render_simple_template(template, "auth");
        assert_eq!(result, "# Constitution: auth");
    }

    #[test]
    fn test_render_spec_template_no_reqs() {
        let template = "# Spec: [MODULE_NAME]\nOverview.";
        let reqs = ExtractedReqs {
            name: "auth".into(),
            description: String::new(),
            requirements: vec![],
        };
        let result = render_spec_template(template, &reqs);
        assert_eq!(result, "# Spec: auth\nOverview.");
    }

    #[test]
    fn test_render_spec_template_with_reqs() {
        let template = "# Spec: [MODULE_NAME]\n\n## Functional Requirements\n\n<!--REQ_START-->\n### REQ-[MODULE]-001: [Requirement Name]\n<!--REQ_END-->\n\n## Out of Scope\n";
        let reqs = ExtractedReqs {
            name: "auth".into(),
            description: String::new(),
            requirements: vec![Requirement {
                id: "REQ-001".into(),
                title: "Login".into(),
                user_story: "As a user, I want to log in.".into(),
                acceptance_criteria: vec![
                    "- WHEN credentials valid THEN the system SHALL authenticate".into(),
                ],
            }],
        };
        let result = render_spec_template(template, &reqs);
        assert!(result.contains("# Spec: auth"));
        assert!(result.contains("REQ-AUTH-001"));
        assert!(result.contains("Login"));
        assert!(result.contains("log in"));
    }

    #[test]
    fn test_find_last_req_number_empty() {
        assert_eq!(find_last_req_number(""), 0);
    }

    #[test]
    fn test_find_last_req_number_simple() {
        assert_eq!(find_last_req_number("### REQ-005: test"), 5);
    }

    #[test]
    fn test_find_last_task_number_empty() {
        assert_eq!(find_last_task_number(""), 0);
    }

    #[test]
    fn test_find_last_task_number() {
        assert_eq!(find_last_task_number("- [ ] [T042] test"), 42);
    }
}
