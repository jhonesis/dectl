use std::fs;
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

    if scope == Scope::Module {
        let module_dir = specs_dir.join(&args.name);
        if module_dir.exists() {
            bail_app_err!(
                format!("Module directory already exists: specs/{}/", args.name),
                "Choose a different name or remove the existing directory"
            );
        }
    }

    match scope {
        Scope::Feature => add_feature(&args.name, args.from.as_deref(), &args)?,
        Scope::Module => dispatch_to_spec_writer(&args.name, args.from.as_deref(), &args)?,
    }

    let desc_preview = if let Some(ref from_path) = args.from {
        format!("{}: from {}", args.name, from_path.display())
    } else {
        args.name.clone()
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
            rusqlite::params![content, tags, "", now, now, "decision"],
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

fn add_feature(name: &str, from_path: Option<&Path>, args: &SpecAddArgs) -> Result<()> {
    let from_content = if let Some(path) = from_path {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;
        if content.trim().is_empty() {
            bail_app_err!(
                format!("File {} is empty", path.display()),
                "Provide a file with content"
            );
        }
        Some(content)
    } else {
        None
    };

    let specs_dir = Path::new("specs");

    // Append requirement to root spec.md
    let spec_path = specs_dir.join("spec.md");
    let mut spec_content = if spec_path.exists() {
        fs::read_to_string(&spec_path).unwrap_or_default()
    } else {
        String::from("# Spec\n\n## Functional Requirements\n\n")
    };

    let last_req = find_last_req_number(&spec_content);
    let next_req = last_req + 1;
    let req_id = format!("REQ-{:03}", next_req);

    if let Some(ref content) = from_content {
        spec_content.push_str(&format!(
            "### {}: {}\n**User Story**:\n> Based on source file\n\n**Acceptance Criteria**:\n- See source file requirements\n\n---\n\n",
            req_id, name
        ));
        // Append source content as reference
        spec_content.push_str(&format!("<!-- SOURCE FILE CONTENT:\n{}\n-->\n\n", content));
    } else {
        spec_content.push_str(&format!(
            "### {}: {}\n**User Story**:\n> As a user, I want {} functionality\n\n**Acceptance Criteria**:\n- WHEN user requests {} THEN system SHALL provide it\n\n---\n\n",
            req_id, name, name, name
        ));
    }

    fs::write(&spec_path, spec_content.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write specs/spec.md: {}", e))?;

    // Append task to root tasks.md
    let tasks_path = specs_dir.join("tasks.md");
    let mut tasks_content = if tasks_path.exists() {
        fs::read_to_string(&tasks_path).unwrap_or_default()
    } else {
        String::from("# Tasks\n\n## Phase 1: Implementation\n\n")
    };

    let last_task = find_last_task_number(&tasks_content);
    let next_task = last_task + 1;
    let task_id = format!("T{:03}", next_task);
    let next_task_id = format!("T{:03}", next_task + 1);

    tasks_content.push_str(&format!(
        "- [{}] [Feature] Implement {} — M ({})\n  **Build**: `cargo build`\n  **Verify**: `cargo test`\n  **Gate**: must pass before [{}]\n\n",
        task_id, name, req_id, next_task_id
    ));

    fs::write(&tasks_path, tasks_content.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write specs/tasks.md: {}", e))?;

    if !args.json {
        println!(
            "Added feature '{}' to specs/spec.md ({}) and specs/tasks.md ({})",
            name, req_id, task_id
        );
    }

    Ok(())
}

fn dispatch_to_spec_writer(name: &str, from_path: Option<&Path>, args: &SpecAddArgs) -> Result<()> {
    let from_content = if let Some(path) = from_path {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;

        if content.trim().is_empty() {
            bail_app_err!(
                format!("File {} is empty", path.display()),
                "Provide a file with content"
            );
        }
        Some(content)
    } else {
        None
    };

    let agent_def = crate::agent::loader::load_agent("spec_writer")
        .ok_or_else(|| anyhow::anyhow!("Agent 'spec_writer' not found"))?
        .0;

    let mut vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    vars.insert("scope".to_string(), "module".to_string());
    vars.insert("name".to_string(), name.to_string());
    vars.insert(
        "task_id".to_string(),
        format!("spec-{}", name.replace(' ', "-").to_lowercase()),
    );

    if let Some(ref content) = from_content {
        vars.insert("from_content".to_string(), content.clone());
    }

    let task_desc = format!("Write specs for module: {}", name);

    if !args.json {
        println!("Dispatching to spec_writer agent...");
        println!("   Scope: module");
        if let Some(path) = from_path {
            println!("   Source: {}", path.display());
        }
        println!();
    }

    let result = crate::agent::runner::run_agent(
        &agent_def,
        &task_desc,
        &vars,
        None,
        false,
        Some(300),
        args.non_interactive,
        &crate::core::output::OutputMode::Human,
        false,
        false,
    )?;

    match &result.status {
        crate::agent::schema::AgentRunStatus::Ok => {
            if !args.json {
                println!();
                println!("Specs written successfully for '{}'", name);
                println!("Next steps:");
                println!("  1. Review the generated files in specs/{}/", name);
                println!(
                    "  2. Implement tasks via: dectl workflow run execute_task --var task_id=<ID> --auto"
                );
                println!("  3. Update progress.json when done");
            }
            Ok(())
        }
        crate::agent::schema::AgentRunStatus::Error { message } => {
            bail_app_err!(
                format!("spec_writer agent failed: {}", message),
                "Check the agent output above for details"
            );
        }
        crate::agent::schema::AgentRunStatus::Timeout => {
            bail_app_err!(
                "spec_writer agent timed out",
                "Try increasing timeout or simplifying the task"
            );
        }
    }
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

fn resolve_scope(args: &SpecAddArgs) -> Result<Scope> {
    if let Some(ref scope) = args.scope {
        return Ok(*scope);
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

#[cfg(test)]
mod tests {}
