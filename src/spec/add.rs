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
    pub auto: bool,
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
        Scope::Feature | Scope::Module => {
            dispatch_to_spec_writer(&args.name, args.from.as_deref(), &args, scope)?
        }
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
        if scope == Scope::Module {
            println!("  1. Agent: create the spec files in specs/{}/", args.name);
        } else {
            println!("  1. Agent: update specs/spec.md and specs/tasks.md");
        }
        println!(
            "  2. Implement tasks via: dectl workflow run execute_task --var task_id=<ID> --auto"
        );
        println!("  3. Update progress.json when done");
    }

    Ok(())
}

fn dispatch_to_spec_writer(
    name: &str,
    from_path: Option<&Path>,
    args: &SpecAddArgs,
    scope: Scope,
) -> Result<()> {
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

    let scope_str = match scope {
        Scope::Feature => "feature",
        Scope::Module => "module",
    };

    let mut vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    vars.insert("scope".to_string(), scope_str.to_string());
    vars.insert("name".to_string(), name.to_string());
    vars.insert(
        "task_id".to_string(),
        format!("spec-{}", name.replace(' ', "-").to_lowercase()),
    );

    if let Some(ref content) = from_content {
        vars.insert("from_content".to_string(), content.clone());
    }

    let task_desc = format!("Write specs for {}: {}", scope_str, name);

    if !args.json {
        println!("Dispatching to spec_writer agent...");
        println!("   Scope: {}", scope_str);
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
        args.auto,
        false,
    )?;

    match &result.status {
        crate::agent::schema::AgentRunStatus::Ok => {
            if !args.json {
                println!();
                println!("Spec guidance emitted for '{}' ({})", name, scope_str);
                println!(
                    "The AI agent must now create/update the spec files following the SDD skill."
                );
                println!("Next steps:");
                if scope == Scope::Module {
                    println!("  1. Agent: create specs/{}/ with the SDD documents", name);
                } else {
                    println!("  1. Agent: update specs/spec.md and specs/tasks.md");
                }
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
