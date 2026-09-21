use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod agent;
mod core;
mod doctor;
mod memory;
mod migrate;
mod project;
mod protocol;
mod rules;
mod session;
mod spec;
mod workflow;

#[derive(Parser)]
#[command(name = "dectl")]
#[command(version = "1.1.0 (schema 1.0)")]
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
    Spec {
        #[command(subcommand)]
        command: Option<SpecCommands>,
    },
    Rules {
        #[command(subcommand)]
        command: Option<RulesCommands>,
    },
    Doctor {
        #[arg(long)]
        fix: bool,
    },
    Migrate {
        #[arg(long)]
        dry_run: bool,
    },
    Hello,
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
    Watch {
        #[arg(long, default_value_t = 5)]
        interval: u64,
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

        #[arg(long, short = 'T', default_value = "note", value_parser = ["note", "decision", "context", "research", "incident", "code-snippet"])]
        r#type: String,
    },
    List {
        #[arg(long)]
        project: Option<String>,

        #[arg(long)]
        global: bool,

        #[arg(long, short = 'l')]
        limit: Option<usize>,

        #[arg(long)]
        include_deleted: bool,
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
    Restore {
        id: i64,
    },
    Export {
        path: PathBuf,

        #[arg(long, default_value = "json")]
        format: String,
    },
    Import {
        path: PathBuf,
    },
    Query {
        query: String,

        #[arg(long)]
        project: Option<String>,

        #[arg(long)]
        global: bool,

        #[arg(long, short = 'l')]
        limit: Option<usize>,
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

        #[arg(long)]
        auto: bool,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    List,
    Describe {
        r#type: String,
    },
    Trust {
        r#type: String,
        #[arg(long, default_value = ".")]
        project: String,
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
        #[arg(long)]
        auto: bool,
    },
}

#[derive(Subcommand)]
enum SpecCommands {
    Init {
        /// Path to a .md file to use as input for creating specs
        #[arg(long)]
        from: Option<PathBuf>,
    },
    Add {
        /// Name of the feature or module (e.g., "biometric-auth", "auth")
        name: String,

        /// Scope: "feature" (adds to root specs) or "module" (creates subdirectory)
        #[arg(long, value_parser = ["feature", "module"])]
        scope: Option<String>,

        /// Path to a .md file with requirements (parsed automatically)
        #[arg(long)]
        from: Option<PathBuf>,

        /// Skip the trust prompt and run non-interactively
        #[arg(long)]
        auto: bool,
    },
}

#[derive(Subcommand)]
enum RulesCommands {
    /// Browse the embedded catalog, optionally filtered
    List {
        /// Filter by section name (e.g. "Security")
        #[arg(long)]
        category: Option<String>,

        /// Filter by severity
        #[arg(long, value_parser = ["must", "should", "avoid"])]
        severity: Option<String>,
    },
    /// Full-text search over the embedded catalog
    Search {
        /// Case-insensitive text matched against id, section, condition, action and tags
        query: String,
    },
    /// Recompute and print the active ruleset (embedded catalog + profile + [rules])
    Resolve,
    /// Stage-aware context slice with a token budget (default 2000)
    Context {
        /// Consumer stage: spec (metadata only), plan, task or review
        #[arg(long, default_value = "task")]
        stage: String,

        /// Files in scope; repeatable, accepts spaces, commas and newlines
        #[arg(long)]
        files: Vec<String>,

        /// Token budget (words x 1.3 estimator); truncated with an omitted footer
        #[arg(long, default_value_t = crate::rules::context::DEFAULT_BUDGET)]
        max_tokens: usize,

        /// Task description used to rank rules by relevance
        #[arg(long)]
        task: Option<String>,
    },
    Profile {
        #[command(subcommand)]
        command: Option<RulesProfileCommands>,
    },
    /// Regenerate every on-disk YAML from the embedded catalog
    Resync,
}

#[derive(Subcommand)]
enum RulesProfileCommands {
    /// Re-run the questionnaire, update profile.toml and regenerate everything
    Update,
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .init();

    let cli = Cli::parse();
    let mode = core::output::OutputMode::from_json_flag(cli.json);

    if let Err(e) = core::bootstrap::initialize(mode) {
        core::output::Output::print_error(&e.to_string(), None, mode);
        std::process::exit(2);
    }

    match &cli.command {
        Some(Commands::Migrate { dry_run }) => {
            if let Err(e) = migrate::run(*dry_run, mode) {
                core::error::exit_for_error(e, mode);
            }
        }
        Some(Commands::Version) => {
            let version = format!(
                "dectl v{} (schema {})",
                env!("CARGO_PKG_VERSION"),
                crate::migrate::engine::SCHEMA_VERSION
            );
            core::output::Output::print_success(&version, mode);
        }
        Some(Commands::Hello) => {
            core::output::Output::print_success("hola mundo", mode);
        }
        Some(Commands::Doctor { fix }) => {
            if let Err(e) = doctor::run(*fix, mode) {
                core::error::exit_for_error(e, mode);
            }
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
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(ProjectCommands::Info) => {
                if let Err(e) = project::info::run(mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(ProjectCommands::Scan { depth }) => {
                if let Err(e) = project::scan::run(*depth, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(ProjectCommands::Context { max_tokens, format }) => {
                if let Err(e) = project::context::run(*max_tokens, format.clone(), mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(ProjectCommands::Watch { interval }) => {
                if let Err(e) = project::watch::run(*interval, mode) {
                    core::error::exit_for_error(e, mode);
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
                r#type,
            }) => {
                let resolved_project = resolve_project(project.as_deref(), *global);
                if let Err(e) = memory::add::run(
                    content.clone(),
                    tags.clone(),
                    resolved_project,
                    r#type.clone(),
                    mode,
                ) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::List {
                project,
                global,
                limit,
                include_deleted,
            }) => {
                let resolved_project = resolve_project(project.as_deref(), *global);
                if let Err(e) = memory::list::run(resolved_project, *limit, *include_deleted, mode)
                {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Search {
                query,
                project,
                global,
            }) => {
                let resolved_project = resolve_project(project.as_deref(), *global);
                if let Err(e) = memory::search::run(query.clone(), resolved_project, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Show { id }) => {
                if let Err(e) = memory::show::run(*id, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Delete { id, hard }) => {
                if let Err(e) = memory::delete::run(*id, *hard, cli.non_interactive, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Edit { id }) => {
                if let Err(e) = memory::edit::run(*id, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Restore { id }) => {
                if let Err(e) = memory::restore::run(*id, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Export { path, format }) => {
                if let Err(e) = memory::export::run(path.clone(), format.clone(), mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Import { path }) => {
                if let Err(e) = memory::import::run(path.clone(), mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(MemoryCommands::Query {
                query,
                project,
                global,
                limit,
            }) => {
                let resolved_project = resolve_project(project.as_deref(), *global);
                if let Err(e) = memory::query::run(query.clone(), resolved_project, *limit, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            None => {
                println!("dectl memory - Memory management");
            }
        },
        Some(Commands::Workflow { command }) => match command {
            Some(WorkflowCommands::List) => {
                if let Err(e) = workflow::list::run(mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(WorkflowCommands::Describe { name }) => {
                if let Err(e) = workflow::describe::run(name, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(WorkflowCommands::Run {
                name,
                var,
                dry_run,
                from_step,
                auto,
            }) => {
                if let Err(e) = workflow::run::run(
                    name,
                    var.clone(),
                    *dry_run,
                    *from_step,
                    *auto,
                    cli.non_interactive,
                    mode,
                ) {
                    core::error::exit_for_error(e, mode);
                }
            }
            None => {
                println!("dectl workflow - Workflow management");
            }
        },
        Some(Commands::Session { command }) => match command {
            Some(SessionCommands::End { dry_run, skip_git }) => {
                if let Err(e) = session::end::run(*dry_run, *skip_git, cli.non_interactive, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            None => {
                println!("dectl session - Session management");
            }
        },
        Some(Commands::Agent { command }) => match command {
            Some(AgentCommands::List) => {
                if let Err(e) = agent::list::run(mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(AgentCommands::Describe { r#type }) => {
                if let Err(e) = agent::describe::run(r#type, mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(AgentCommands::Trust { r#type, project }) => {
                if let Err(e) = agent::trust::run(r#type, project, mode) {
                    core::error::exit_for_error(e, mode);
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
                auto,
            }) => {
                if let Err(e) = agent::run::run(
                    r#type,
                    task.as_deref(),
                    file.as_deref(),
                    var,
                    *timeout,
                    *dry_run,
                    *parallel,
                    *auto,
                    cli.non_interactive,
                    mode,
                ) {
                    core::error::exit_for_error(e, mode);
                }
            }
            None => {
                println!("dectl agent - Agent management");
            }
        },
        Some(Commands::Spec { command }) => match command {
            Some(SpecCommands::Init { from }) => {
                if let Err(e) = spec::init::run(cli.json, cli.non_interactive, from.as_deref()) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(SpecCommands::Add {
                ref name,
                ref scope,
                ref from,
                auto,
            }) => {
                let args = spec::add::SpecAddArgs {
                    name: name.clone(),
                    scope: scope.clone().map(|s| match s.as_str() {
                        "module" => spec::add::Scope::Module,
                        _ => spec::add::Scope::Feature,
                    }),
                    from: from.clone(),
                    json: cli.json,
                    non_interactive: cli.non_interactive,
                    auto: *auto,
                };
                if let Err(e) = spec::add::run(args) {
                    core::error::exit_for_error(e, mode);
                }
            }
            None => {
                core::output::Output::print_success("dectl spec - Spec-Driven Development", mode);
            }
        },
        Some(Commands::Rules { command }) => match command {
            Some(RulesCommands::List { category, severity }) => {
                if let Err(e) = rules::cli::run_list(category.clone(), severity.clone(), mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(RulesCommands::Search { query }) => {
                if let Err(e) = rules::cli::run_search(query.clone(), mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(RulesCommands::Resolve) => {
                if let Err(e) = rules::cli::run_resolve(mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(RulesCommands::Context {
                stage,
                files,
                max_tokens,
                task,
            }) => {
                if let Err(e) = rules::cli::run_context(
                    stage.clone(),
                    files.clone(),
                    *max_tokens,
                    task.clone(),
                    mode,
                ) {
                    core::error::exit_for_error(e, mode);
                }
            }
            Some(RulesCommands::Profile { command }) => match command {
                Some(RulesProfileCommands::Update) => {
                    if let Err(e) = rules::cli::run_profile_update(cli.non_interactive, mode) {
                        core::error::exit_for_error(e, mode);
                    }
                }
                None => {
                    core::output::Output::print_success(
                        "dectl rules profile - use `dectl rules profile update`",
                        mode,
                    );
                }
            },
            Some(RulesCommands::Resync) => {
                if let Err(e) = rules::cli::run_resync(mode) {
                    core::error::exit_for_error(e, mode);
                }
            }
            None => {
                core::output::Output::print_success("dectl rules - Development rules", mode);
            }
        },
        Some(Commands::ExecFromFile { path }) => {
            if let Err(e) = protocol::exec_from_file::run(path, mode) {
                core::error::exit_for_error(e, mode);
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
