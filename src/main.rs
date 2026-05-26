#![allow(dead_code)]

use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod agent;
mod core;
mod memory;
mod project;
mod protocol;
mod session;
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
    GenerateCompletions {
        shell: String,
    },
    Session {
        #[command(subcommand)]
        command: Option<SessionCommands>,
    },
    Agent {
        #[command(subcommand)]
        command: Option<AgentCommands>,
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

        #[arg(long, default_value = "other")]
        r#type: String,
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

        #[arg(long)]
        global: bool,
    },
    List {
        #[arg(long)]
        project: Option<String>,

        #[arg(long)]
        global: bool,

        #[arg(long, short = 'l')]
        limit: Option<usize>,
    },
    Search {
        query: String,

        #[arg(long)]
        project: Option<String>,

        #[arg(long)]
        global: bool,
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

#[derive(Subcommand)]
enum AgentCommands {
    List,
    Describe {
        r#type: String,
    },
    Run {
        r#type: String,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        var: Vec<String>,
        #[arg(long)]
        timeout: Option<u64>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        parallel: bool,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    End {
        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        skip_git: bool,
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
            Some(ProjectCommands::Init {
                standard,
                full,
                r#type,
            }) => {
                let level = if *full {
                    project::templates::InitLevel::Level3
                } else if *standard {
                    project::templates::InitLevel::Level2
                } else {
                    project::templates::InitLevel::Level1
                };

                let project_type = project::templates::ProjectType::from_str(r#type);
                if project_type.is_none() {
                    core::output::Output::print_error(
                        "Invalid project type. Use: api, cli, microservice, or other",
                        None,
                        mode,
                    );
                    std::process::exit(1);
                }

                if let Err(e) =
                    project::init::run(level, project_type.unwrap(), !cli.non_interactive)
                {
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
                if let Err(e) = project::context::run(*max_tokens, format.clone(), mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
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
                global,
            }) => {
                let resolved_project = resolve_project(project.as_deref(), *global);
                if let Err(e) =
                    memory::add::run(content.clone(), tags.clone(), resolved_project, mode)
                {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(MemoryCommands::List {
                project,
                global,
                limit,
            }) => {
                let resolved_project = resolve_project(project.as_deref(), *global);
                if let Err(e) = memory::list::run(resolved_project, *limit, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(MemoryCommands::Search {
                query,
                project,
                global,
            }) => {
                let resolved_project = resolve_project(project.as_deref(), *global);
                if let Err(e) = memory::search::run(query.clone(), resolved_project, mode) {
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
                if let Err(e) = memory::delete::run(*id, *hard, cli.non_interactive, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(MemoryCommands::Edit { id }) => {
                if let Err(e) = memory::edit::run(*id, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            None => {
                println!("dectl memory - Memory management");
            }
        },
        Some(Commands::Workflow { command }) => match command {
            Some(WorkflowCommands::List) => {
                if let Err(e) = workflow::list::run(mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(WorkflowCommands::Describe { name }) => {
                if let Err(e) = workflow::describe::run(name, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(WorkflowCommands::Run {
                name,
                var,
                dry_run,
                from_step,
            }) => {
                if let Err(e) = workflow::run::run(
                    name,
                    var.clone(),
                    *dry_run,
                    *from_step,
                    cli.non_interactive,
                    mode,
                ) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            None => {
                println!("dectl workflow - Workflow management");
            }
        },
        Some(Commands::Session { command }) => match command {
            Some(SessionCommands::End { dry_run, skip_git }) => {
                if let Err(e) = session::end::run(*dry_run, *skip_git, cli.non_interactive, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            None => {
                println!("dectl session - Session management");
            }
        },
        Some(Commands::Agent { command }) => match command {
            Some(AgentCommands::List) => {
                if let Err(e) = agent::list::run(mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(AgentCommands::Describe { r#type }) => {
                if let Err(e) = agent::describe::run(r#type, mode) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            Some(AgentCommands::Run {
                r#type,
                task,
                file,
                var,
                timeout,
                dry_run,
                parallel,
            }) => {
                if let Err(e) = agent::run::run(
                    r#type,
                    task.as_deref(),
                    file.as_deref(),
                    var,
                    *timeout,
                    *dry_run,
                    *parallel,
                    cli.non_interactive,
                    mode,
                ) {
                    core::output::Output::print_error(&e.to_string(), None, mode);
                    std::process::exit(1);
                }
            }
            None => {
                println!("dectl agent - Agent management");
            }
        },
        Some(Commands::ExecFromFile { path }) => {
            if let Err(e) = protocol::exec_from_file::run(path, mode) {
                core::output::Output::print_error(&e.to_string(), None, mode);
                std::process::exit(1);
            }
        }
        Some(Commands::GenerateCompletions { shell }) => {
            use clap_complete::Shell;
            let shell = match shell.to_lowercase().as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                "powershell" => Shell::PowerShell,
                _ => {
                    eprintln!("Supported shells: bash, zsh, fish, powershell");
                    std::process::exit(1);
                }
            };
            let mut cli = Cli::command();
            clap_complete::generate(shell, &mut cli, "dectl", &mut std::io::stdout());
        }
        None => {
            println!("dectl - Dev Environment Control");
            println!("Use --help for more information");
        }
    }
}

fn resolve_project(project_arg: Option<&str>, global: bool) -> Option<String> {
    if global {
        None
    } else if let Some(p) = project_arg {
        Some(p.to_string())
    } else {
        crate::core::config::ProjectConfig::current_project_name()
    }
}
