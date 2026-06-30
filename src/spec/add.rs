use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::bail_app_err;
use crate::core::db::{get_db, Storage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Feature,
    Module,
}

pub struct SpecAddArgs {
    pub name: String,
    pub scope: Option<Scope>,
    pub from: Option<PathBuf>,
    pub json: bool,
    pub non_interactive: bool,
}

pub struct ExtractedReqs {
    pub name: String,
    pub description: String,
    pub requirements: Vec<Requirement>,
}

pub struct Requirement {
    pub id: String,
    pub title: String,
    pub user_story: String,
    pub acceptance_criteria: Vec<String>,
}

pub fn run(args: SpecAddArgs) -> Result<()> {
    let dec_path = Path::new(".dec");
    if !dec_path.exists() {
        bail_app_err!(
            ".dec/ not found",
            "Run `dectl project init` first to initialize a project"
        );
    }

    let specs_dir = Path::new("specs");
    if !specs_dir.exists() {
        bail_app_err!(
            "specs/ directory not found",
            "Run `dectl spec init` first to create the SDD structure"
        );
    }

    let scope = resolve_scope(&args)?;

    let mut reqs = if let Some(ref from_path) = args.from {
        let mut extracted =
            extract_reqs_from_file(from_path).map_err(|e| anyhow::anyhow!("{}", e))?;
        extracted.name = args.name.clone();
        extracted
    } else {
        ExtractedReqs {
            name: args.name.clone(),
            description: String::new(),
            requirements: vec![],
        }
    };

    crate::spec::interview::run(&args.name, &scope, &mut reqs, args.non_interactive)?;

    match scope {
        Scope::Feature => crate::spec::feature::run(&args.name, &reqs)?,
        Scope::Module => crate::spec::module::run(&args.name, &reqs)?,
    }

    let desc_preview = if reqs.description.is_empty() {
        args.name.clone()
    } else {
        format!("{}: {}", args.name, reqs.description)
    };
    let type_ = match scope {
        Scope::Feature => "decision",
        Scope::Module => "decision",
    };
    let tags = format!(
        "spec,{}",
        match scope {
            Scope::Feature => "feature",
            Scope::Module => "module",
        }
    );

    if let Ok(db) = get_db() {
        let now = chrono::Utc::now().to_rfc3339();
        let content = format!("Spec added: {} ({:?}) — {}", args.name, scope, desc_preview);
        let _ = db.execute(
            "INSERT INTO memories (content, tags, project, created_at, updated_at, type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![content, tags, "", now, now, type_],
        );
    }

    if !args.json {
        println!();
        println!("Next steps:");
        println!("  1. Review the generated files in specs/");
        println!(
            "  2. Implement tasks via: dectl workflow run execute_task --var task_id=<ID> --auto"
        );
        println!("  3. Update progress.json when done");
    }

    Ok(())
}

fn resolve_scope(args: &SpecAddArgs) -> Result<Scope> {
    if let Some(ref scope) = args.scope {
        return Ok(*scope);
    }

    if let Some(ref from_path) = args.from {
        if let Ok(reqs) = extract_reqs_from_file(from_path) {
            let detected = auto_detect_scope(&reqs);
            return Ok(detected);
        }
    }

    if args.non_interactive {
        return Ok(Scope::Feature);
    }

    let input = prompt_user_for_scope()?;
    Ok(input)
}

fn prompt_user_for_scope() -> Result<Scope> {
    use std::io::{self, Write};
    loop {
        print!("Scope: (f)eature or (m)odule? [f]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "" | "f" | "feature" => return Ok(Scope::Feature),
            "m" | "module" => return Ok(Scope::Module),
            _ => println!("Please enter 'f' for feature or 'm' for module."),
        }
    }
}

pub fn extract_reqs_from_file(path: &std::path::Path) -> Result<ExtractedReqs, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    parse_reqs_from_str(&content)
}

pub fn parse_reqs_from_str(content: &str) -> Result<ExtractedReqs, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut name = String::new();
    let mut description = String::new();
    let mut requirements = Vec::new();
    let mut in_desc = false;
    let mut desc_lines = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        // Title: first # or ## heading
        if name.is_empty() && (line.starts_with("# ") || line.starts_with("## ")) {
            name = line.trim_start_matches('#').trim().to_string();
            in_desc = true;
            i += 1;
            continue;
        }

        // Collect description (paragraph between title and first REQ)
        if in_desc {
            if line.starts_with("### REQ-") {
                in_desc = false;
                description = desc_lines.join(" ").trim().to_string();
                continue;
            }
            if !line.trim().is_empty() {
                desc_lines.push(line.trim());
            }
            i += 1;
            continue;
        }

        // REQ section
        if let Some(req) = parse_req_block(&lines, i) {
            requirements.push(req.0);
            i = req.1;
            continue;
        }

        i += 1;
    }

    if in_desc && !desc_lines.is_empty() {
        description = desc_lines.join(" ").trim().to_string();
    }

    Ok(ExtractedReqs {
        name,
        description,
        requirements,
    })
}

fn parse_req_block(lines: &[&str], start: usize) -> Option<(Requirement, usize)> {
    let header = lines[start];
    let re = regex::Regex::new(r"^###\s+(REQ-[\w-]+):\s*(.*)").ok()?;
    let caps = re.captures(header)?;
    let id = caps.get(1)?.as_str().to_string();
    let mut title = caps.get(2)?.as_str().trim().to_string();

    // If no title on the same line, look at next line
    if title.is_empty() {
        if let Some(next) = lines.get(start + 1) {
            if !next.trim().is_empty() && !next.starts_with("**") {
                title = next.trim().to_string();
            }
        }
    }

    let mut user_story = String::new();
    let mut acceptance_criteria = Vec::new();
    let mut in_story = false;
    let mut in_ac = false;
    let mut i = start + 1;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("### ") {
            break; // next REQ section
        }

        if line.contains("**User Story**") || line.contains("*User Story*") {
            in_story = true;
            in_ac = false;
            i += 1;
            continue;
        }

        if line.contains("**Acceptance Criteria**") || line.contains("*Acceptance Criteria*") {
            in_story = false;
            in_ac = true;
            i += 1;
            continue;
        }

        if line.contains("**Implementation Notes**") || line.contains("*Implementation Notes*") {
            in_story = false;
            in_ac = false;
            i += 1;
            continue;
        }

        if in_story {
            // Capture As a... user story (often prefixed with >)
            let story_line = line.trim_start_matches('>').trim();
            if !story_line.is_empty() {
                if user_story.is_empty() {
                    user_story = story_line.to_string();
                } else {
                    user_story.push(' ');
                    user_story.push_str(story_line);
                }
            }
        }

        if in_ac && line.starts_with("- WHEN") {
            acceptance_criteria.push(line.to_string());
        }

        i += 1;
    }

    Some((
        Requirement {
            id,
            title,
            user_story,
            acceptance_criteria,
        },
        i,
    ))
}

pub fn auto_detect_scope(reqs: &ExtractedReqs) -> Scope {
    let req_count = reqs.requirements.len();
    let has_module_keyword = reqs.description.to_lowercase().contains("module")
        || reqs.name.to_lowercase().contains("module");
    if req_count >= 5 || (req_count >= 3 && has_module_keyword) {
        Scope::Module
    } else {
        Scope::Feature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extracts_single_req() {
        let md = "# Biometric Auth
Adds fingerprint login.

### REQ-AUTH-001: Biometric Login
**User Story**:
> As a user, I want to log in with my fingerprint so that I don't need to type my password.

**Acceptance Criteria**:
- WHEN user has fingerprint enabled THEN the login screen SHALL show biometric option
- WHEN fingerprint matches THEN the system SHALL authenticate the user";
        let result = parse_reqs_from_str(md).unwrap();
        assert_eq!(result.name, "Biometric Auth");
        assert!(result.description.contains("fingerprint"));
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].id, "REQ-AUTH-001");
        assert_eq!(result.requirements[0].title, "Biometric Login");
        assert!(result.requirements[0].user_story.contains("fingerprint"));
        assert_eq!(result.requirements[0].acceptance_criteria.len(), 2);
    }

    #[test]
    fn test_extracts_multiple_reqs() {
        let md = "# Auth Module
Authentication system.

### REQ-001: User Login
**User Story**:
> As a user, I want to log in.

### REQ-002: User Logout
**User Story**:
> As a user, I want to log out.

### REQ-003: Password Reset
**User Story**:
> As a user, I want to reset my password.

### REQ-004: OAuth Login
**User Story**:
> As a user, I want to log in with Google.

### REQ-005: 2FA
**User Story**:
> As a user, I want two-factor authentication.";
        let result = parse_reqs_from_str(md).unwrap();
        assert_eq!(result.requirements.len(), 5);
        assert_eq!(result.requirements[0].id, "REQ-001");
        assert_eq!(result.requirements[4].id, "REQ-005");
    }

    #[test]
    fn test_extracts_no_reqs() {
        let md = "# Simple Note\nJust a plain document with no requirements.";
        let result = parse_reqs_from_str(md).unwrap();
        assert_eq!(result.requirements.len(), 0);
        assert_eq!(result.name, "Simple Note");
    }

    #[test]
    fn test_extracts_user_story() {
        let md = "### REQ-001: Feature
**User Story**:
> As a developer, I want tests so that I can verify correctness.";
        let result = parse_reqs_from_str(md).unwrap();
        assert_eq!(
            result.requirements[0].user_story,
            "As a developer, I want tests so that I can verify correctness."
        );
    }

    #[test]
    fn test_extracts_acceptance_criteria() {
        let md = "### REQ-001: Feature
**Acceptance Criteria**:
- WHEN condition THEN the system SHALL do X
- WHEN other THEN the system SHALL do Y";
        let result = parse_reqs_from_str(md).unwrap();
        assert_eq!(result.requirements[0].acceptance_criteria.len(), 2);
        assert!(result.requirements[0].acceptance_criteria[0].contains("WHEN condition"));
    }

    #[test]
    fn test_empty_file() {
        let result = parse_reqs_from_str("").unwrap();
        assert_eq!(result.name, "");
        assert!(result.description.is_empty());
        assert_eq!(result.requirements.len(), 0);
    }

    #[test]
    fn test_extract_reqs_from_file_nonexistent() {
        let result = extract_reqs_from_file(std::path::Path::new("/nonexistent/path.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_detect_feature() {
        let reqs = ExtractedReqs {
            name: "feat".into(),
            description: "simple".into(),
            requirements: vec![Requirement {
                id: "REQ-001".into(),
                title: "test".into(),
                user_story: String::new(),
                acceptance_criteria: vec![],
            }],
        };
        assert_eq!(auto_detect_scope(&reqs), Scope::Feature);
    }

    #[test]
    fn test_auto_detect_module() {
        let reqs = ExtractedReqs {
            name: "auth".into(),
            description: "auth module for the system".into(),
            requirements: vec![
                Requirement {
                    id: "REQ-001".into(),
                    title: "a".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
                Requirement {
                    id: "REQ-002".into(),
                    title: "b".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
                Requirement {
                    id: "REQ-003".into(),
                    title: "c".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
                Requirement {
                    id: "REQ-004".into(),
                    title: "d".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
                Requirement {
                    id: "REQ-005".into(),
                    title: "e".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
            ],
        };
        assert_eq!(auto_detect_scope(&reqs), Scope::Module);
    }

    #[test]
    fn test_auto_detect_module_by_keyword() {
        let reqs = ExtractedReqs {
            name: "auth".into(),
            description: "the auth module handles login".into(),
            requirements: vec![
                Requirement {
                    id: "REQ-001".into(),
                    title: "a".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
                Requirement {
                    id: "REQ-002".into(),
                    title: "b".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
                Requirement {
                    id: "REQ-003".into(),
                    title: "c".into(),
                    user_story: String::new(),
                    acceptance_criteria: vec![],
                },
            ],
        };
        assert_eq!(auto_detect_scope(&reqs), Scope::Module);
    }

    #[test]
    fn test_parse_req_title_on_next_line() {
        let md = "### REQ-001:
Title on Next Line
**User Story**:
> As a user, I want X.";
        let result = parse_reqs_from_str(md).unwrap();
        assert_eq!(result.requirements[0].title, "Title on Next Line");
    }
}
