use crate::bail_app_err;
use anyhow::{Context, Result};
use is_terminal::IsTerminal;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::auto_fill::{
    detect_stack, fill_project_files, is_project_empty, scan_docs_for_context, DetectedStack,
    OptionalContext,
};
use super::templates::{InitLevel, ProjectType, Templates};

const PROJECT_TYPES: &[&str] = &["api", "cli", "web", "library", "other"];

pub fn run(level: InitLevel, project_type: ProjectType, _interactive: bool) -> Result<()> {
    let dec_path = Path::new(".dec");

    if dec_path.exists() {
        bail_app_err!(
            ".dec/ already exists. Remove it first if you want to reinitialize.\n\
             Hint: rm -rf .dec/",
            "Use --type cli, api, microservice, or other"
        );
    }

    let mut files = Templates::files_for_level(level);

    if level == InitLevel::Level2 || level == InitLevel::Level3 {
        let type_workflows = Templates::workflows_for_type(project_type);
        files.extend(type_workflows);

        if let Some((path, content)) = Templates::system_prompt_for_type(project_type) {
            files.push((path, content));
        }
    }

    for (path, _content) in &files {
        let full_path = Path::new(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }
    }

    for (path, content) in &files {
        if content.is_empty() {
            continue;
        }
        fs::write(path, *content).with_context(|| format!("Failed to write file {:?}", path))?;
    }

    if is_project_empty() {
        if std::io::stdout().is_terminal() {
            handle_empty_project()?;
        } else {
            println!("Empty project detected. Use interactive mode to fill files automatically.");
        }
    } else {
        println!("Detecting project languages...");
        let stack = detect_stack();
        let context = scan_docs_for_context();
        fill_project_files(&stack, &context, Path::new("."))?;
        println!("✓ Basic context written to project.toml");
        println!("→ Created .dec/prompts/tasks/auto-fill.md for AI-assisted setup");
        println!("→ On first session, the AI will complete project.toml and project.isa.md");
    }

    print_next_steps(level, project_type);

    Ok(())
}

fn handle_empty_project() -> Result<()> {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Detected: empty project (0 files)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Options:");
    println!("  1) Fill now (recommended) — I'll ask you a few questions");
    println!("  2) Manual setup — I'll edit files myself later");
    println!("  3) Cancel");
    println!();

    let choice = prompt_with_default("Choice", "1")?;

    match choice.trim() {
        "1" | "" => {
            let context = ask_user_questions()?;
            let stack = context_to_stack(&context);
            fill_project_files(&stack, &context, Path::new("."))?;
            println!("\n✓ Project files filled based on your answers.");
        }
        "2" => {
            println!("\nYou'll need to manually edit:");
            println!("  - .dec/config/project.toml");
            println!("  - .dec/isa/project.isa.md");
        }
        "3" => {
            println!("\nAborted. You can run 'dectl init' again later.");
            std::process::exit(0);
        }
        _ => {
            println!("Invalid choice. Please enter 1, 2, or 3.");
        }
    }

    Ok(())
}

fn ask_user_questions() -> Result<OptionalContext> {
    let mut context = OptionalContext::default();

    let default_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_default();

    context.name = Some(prompt_with_default("Project name", &default_name)?);

    println!("\nProject type:");
    for (i, t) in PROJECT_TYPES.iter().enumerate() {
        println!("  {}) {}", i + 1, t);
    }
    let type_choice = prompt_with_default("Type", "2")?;
    let type_idx: usize = type_choice
        .trim()
        .parse::<usize>()
        .unwrap_or(2)
        .saturating_sub(1);
    let _project_type = PROJECT_TYPES.get(type_idx).unwrap_or(&"other");

    context.description = Some(prompt_with_default(
        "Description",
        "A short description of what this project does",
    )?);

    context.vision = Some(prompt_with_default(
        "Vision",
        "What are we building? What's the main goal?",
    )?);

    println!("\nLanguages (comma separated, e.g., rust, typescript):");
    let _languages = prompt_with_default("Languages", "")?;

    println!("\n✓ Configuration saved.");

    Ok(context)
}

fn context_to_stack(context: &OptionalContext) -> DetectedStack {
    DetectedStack {
        name: context.name.clone(),
        project_type: "other".to_string(),
        ..Default::default()
    }
}

fn prompt_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

fn print_next_steps(level: InitLevel, project_type: ProjectType) {
    let level_name = match level {
        InitLevel::Level1 => "level 1 (minimal)",
        InitLevel::Level2 => "level 2 (standard)",
        InitLevel::Level3 => "level 3 (full)",
    };
    let type_info = match project_type {
        ProjectType::Other => String::new(),
        _ => format!(" [{}]", project_type.as_str()),
    };
    println!(
        "\nCreated .dec/ with {}{} ({} files)",
        level_name,
        type_info,
        Templates::files_for_level(level).len()
    );
    println!("\nNext steps:");
    println!("  1. Open the project with your AI tool (Claude Code, opencode, Gemini CLI, etc.)");
    println!("  2. The AI reads .dec/ automatically — no manual context loading needed");
    println!("  3. Inside the AI, run: dectl project info --json");
    println!("     → The AI sees your project stack, frameworks, and description instantly");
    println!("  4. AGENTS.md guides the AI on how to work with this project");
}
