#![allow(dead_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod core;
mod memory;
mod project;
mod protocol;
mod workflow;

#[derive(Parser)]
#[command(name = "dectl")]
#[command(version = "0.1.0")]
#[command(about = "Dev Environment Control", long_about = None)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[arg(long, global = true)]
    non_interactive: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Project {
        #[command(subcommand)]
        command: Option<ProjectCommands>,
    },
    Memory {
        #[command(subcommand)]
        command: Option<MemoryCommands>,
    },
    Workflow {
        #[command(subcommand)]
        command: Option<WorkflowCommands>,
    },
    ExecFromFile {
        path: PathBuf,
    },
    Version,
}

#[derive(Subcommand)]
enum ProjectCommands {
    Init {
        #[arg(long)]
        standard: bool,

        #[arg(long)]
        full: bool,
    },
    Info,
    Scan {
        #[arg(long)]
        depth: Option<usize>,
    },
    Context {
        #[arg(long)]
        max_tokens: Option<usize>,

        #[arg(long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    Add {
        content: Option<String>,

        #[arg(long, short = 't')]
        tags: Option<String>,

        #[arg(long)]
        project: Option<String>,
    },
    List {
        #[arg(long)]
        project: Option<String>,

        #[arg(long, short = 'l')]
        limit: Option<usize>,
    },
    Search {
        query: String,

        #[arg(long)]
        project: Option<String>,
    },
    Show {
        id: i64,
    },
    Delete {
        id: i64,

        #[arg(long)]
        hard: bool,
    },
    Edit {
        id: i64,
    },
}

#[derive(Subcommand)]
enum WorkflowCommands {
    List,
    Describe {
        name: String,
    },
    Run {
        name: String,

        #[arg(long)]
        var: Vec<String>,

        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        from_step: Option<usize>,
    },
}

fn main() {
    let cli = Cli::parse();
    let mode = core::output::OutputMode::from_json_flag(cli.json);

    match &cli.command {
        Some(Commands::Version) => {
            core::output::Output::print_success("dectl v0.1.0", mode);
        }
        Some(Commands::Project { command }) => match command {
            Some(ProjectCommands::Init { standard, full }) => {
                let level = if *full {
                    project::templates::InitLevel::Level3
                } else if *standard {
                    project::templates::InitLevel::Level2
                } else {
                    project::templates::InitLevel::Level1
                };

                if let Err(e) = project::init::run(level, !cli.non_interactive) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(ProjectCommands::Info) => {
                if let Err(e) = project::info::run(mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(ProjectCommands::Scan { depth }) => {
                if let Err(e) = project::scan::run(*depth, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(ProjectCommands::Context { max_tokens, format }) => {
                #[derive(serde::Serialize)]
                struct ContextResult {
                    max_tokens: Option<usize>,
                    format: String,
                }
                core::output::Output::print(
                    &ContextResult {
                        max_tokens: *max_tokens,
                        format: format.clone(),
                    },
                    mode,
                );
            }
            None => {
                core::output::Output::print_success("dectl project - Project management", mode);
            }
        },
        Some(Commands::Memory { command }) => match command {
            Some(MemoryCommands::Add {
                content,
                tags,
                project,
            }) => {
                if let Err(e) =
                    memory::add::run(content.clone(), tags.clone(), project.clone(), mode)
                {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(MemoryCommands::List { project, limit }) => {
                if let Err(e) = memory::list::run(project.clone(), *limit, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(MemoryCommands::Search { query, project }) => {
                if let Err(e) = memory::search::run(query.clone(), project.clone(), mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(MemoryCommands::Show { id }) => {
                if let Err(e) = memory::show::run(*id, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(MemoryCommands::Delete { id, hard }) => {
                println!("memory delete: id={}, hard={}", id, hard);
            }
            Some(MemoryCommands::Edit { id }) => {
                println!("memory edit: id={}", id);
            }
            None => {
                println!("dectl memory - Memory management");
            }
        },
        Some(Commands::Workflow { command }) => match command {
            Some(WorkflowCommands::List) => {
                println!("workflow list");
            }
            Some(WorkflowCommands::Describe { name }) => {
                println!("workflow describe: {}", name);
            }
            Some(WorkflowCommands::Run {
                name,
                var,
                dry_run,
                from_step,
            }) => {
                println!(
                    "workflow run: name={}, var={:?}, dry_run={}, from_step={:?}",
                    name, var, dry_run, from_step
                );
            }
            None => {
                println!("dectl workflow - Workflow management");
            }
        },
        Some(Commands::ExecFromFile { path }) => {
            if let Err(e) = protocol::exec_from_file::run(path, mode) {
                core::output::Output::print_error(&e.to_string(), None, mode);
                std::process::exit(1);
            }
        }
        None => {
            println!("dectl - Dev Environment Control");
            println!("Use --help for more information");
        }
    }
}
