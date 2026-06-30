use std::io::{self, Write};

use anyhow::Result;

use crate::spec::add::{ExtractedReqs, Requirement, Scope};

pub fn run(
    name: &str,
    scope: &Scope,
    extracted: &mut ExtractedReqs,
    non_interactive: bool,
) -> Result<()> {
    fill_description(name, extracted, non_interactive)?;
    fill_requirements(extracted, non_interactive)?;
    fill_scope(name, scope, extracted, non_interactive)?;
    Ok(())
}

fn fill_description(
    name: &str,
    extracted: &mut ExtractedReqs,
    non_interactive: bool,
) -> Result<()> {
    if !extracted.description.is_empty() {
        return Ok(());
    }
    if non_interactive {
        extracted.description = format!("{} feature/module", name);
        return Ok(());
    }
    let desc = prompt(&format!("Describe {} in 1-2 lines:", name))?;
    extracted.description = desc;
    Ok(())
}

fn fill_requirements(extracted: &mut ExtractedReqs, non_interactive: bool) -> Result<()> {
    if !extracted.requirements.is_empty() {
        return Ok(());
    }
    if non_interactive {
        return Ok(());
    }
    println!("No requirements extracted. Describe the functionalities (one per line):");
    println!("  Format: - As a <user>, I want <goal>");
    println!("  (empty line to finish)");
    loop {
        let line = prompt(">")?;
        if line.trim().is_empty() {
            break;
        }
        let cleaned = line.trim_start_matches("- ").trim();
        let req = parse_interactive_line(cleaned, extracted.requirements.len() + 1);
        extracted.requirements.push(req);
    }
    Ok(())
}

fn fill_scope(
    _name: &str,
    scope: &Scope,
    _extracted: &mut ExtractedReqs,
    non_interactive: bool,
) -> Result<()> {
    if let Scope::Feature = scope {
        return Ok(());
    }
    if non_interactive {
        return Ok(());
    }
    // scope was auto-detected as Module or explicit, no prompt needed
    Ok(())
}

fn parse_interactive_line(line: &str, index: usize) -> Requirement {
    let id = format!("REQ-{:03}", index);
    let title: String;
    let user_story: String;

    if let Some(story_start) = line.find("As a ") {
        let prefix = line[..story_start].trim();
        if prefix.is_empty() {
            title = line.to_string();
            user_story = line.to_string();
        } else {
            title = prefix.to_string();
            user_story = line[story_start..].to_string();
        }
    } else {
        title = line.to_string();
        user_story = String::new();
    }

    let title = if title.is_empty() {
        format!("Requirement {}", index)
    } else {
        title
    };

    Requirement {
        id,
        title,
        user_story,
        acceptance_criteria: vec![],
    }
}

fn prompt(text: &str) -> Result<String> {
    print!("{} ", text);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interactive_line_with_story_at_start() {
        let req = parse_interactive_line("As a user, I want to log in", 1);
        assert_eq!(req.id, "REQ-001");
        assert_eq!(req.title, "As a user, I want to log in");
        assert_eq!(req.user_story, "As a user, I want to log in");
    }

    #[test]
    fn test_parse_interactive_line_with_prefix() {
        let req = parse_interactive_line("Login: As a user, I want to log in", 2);
        assert_eq!(req.id, "REQ-002");
        assert_eq!(req.title, "Login:");
        assert_eq!(req.user_story, "As a user, I want to log in");
    }

    #[test]
    fn test_parse_interactive_line_empty() {
        let req = parse_interactive_line("something", 3);
        assert_eq!(req.id, "REQ-003");
        assert_eq!(req.title, "something");
        assert!(req.user_story.is_empty());
    }

    #[test]
    fn test_fill_description_skips_if_present() {
        let mut extracted = ExtractedReqs {
            name: "test".into(),
            description: "already set".into(),
            requirements: vec![],
        };
        fill_description("test", &mut extracted, false).unwrap();
        assert_eq!(extracted.description, "already set");
    }

    #[test]
    fn test_fill_requirements_skips_if_present() {
        let mut extracted = ExtractedReqs {
            name: "test".into(),
            description: String::new(),
            requirements: vec![Requirement {
                id: "REQ-001".into(),
                title: "existing".into(),
                user_story: String::new(),
                acceptance_criteria: vec![],
            }],
        };
        fill_requirements(&mut extracted, false).unwrap();
        assert_eq!(extracted.requirements.len(), 1);
    }

    #[test]
    fn test_non_interactive_defaults() {
        let mut extracted = ExtractedReqs {
            name: "auth".into(),
            description: String::new(),
            requirements: vec![],
        };
        fill_description("auth", &mut extracted, true).unwrap();
        assert_eq!(extracted.description, "auth feature/module");
        fill_requirements(&mut extracted, true).unwrap();
        assert!(extracted.requirements.is_empty());
    }
}
