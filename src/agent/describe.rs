use crate::agent::schema::AgentSource;
use crate::core::output::{Output, OutputMode};
use anyhow::Result;
use serde::Serialize;

pub fn run(agent_type: &str, mode: OutputMode) -> Result<()> {
    let agent = crate::agent::loader::load_agent(agent_type);

    let (def, source) = match agent {
        Some(a) => a,
        None => {
            let msg = format!(
                "Agent '{}' not found. Run 'dectl agent list' to see available agents.",
                agent_type
            );
            Output::print_error(&msg, None, mode);
            std::process::exit(1);
        }
    };

    if mode.is_json() {
        #[derive(Serialize)]
        struct InputInfo {
            name: String,
            description: String,
            required: bool,
            default: Option<String>,
        }
        #[derive(Serialize)]
        struct StepInfo {
            step_type: String,
            description: String,
            content: Option<String>,
            cmd: Option<Vec<String>>,
        }
        #[derive(Serialize)]
        struct AgentInfo {
            name: String,
            role: String,
            description: String,
            source: String,
            inputs: Vec<InputInfo>,
            steps: Vec<StepInfo>,
        }
        #[derive(Serialize)]
        struct OutputData {
            status: String,
            agent: AgentInfo,
        }

        let source_str = match &source {
            AgentSource::Builtin => "builtin".to_string(),
            AgentSource::Custom(p) => format!("custom ({})", p.display()),
        };

        let inputs: Vec<InputInfo> = def
            .inputs
            .iter()
            .map(|i| InputInfo {
                name: i.name.clone(),
                description: i.description.clone(),
                required: i.is_required,
                default: i.default.clone(),
            })
            .collect();

        let steps: Vec<StepInfo> = def
            .steps
            .iter()
            .map(|s| StepInfo {
                step_type: s.step_type.to_string(),
                description: s.description.clone(),
                content: s.content.clone(),
                cmd: s.cmd.clone(),
            })
            .collect();

        Output::print(
            &OutputData {
                status: "ok".to_string(),
                agent: AgentInfo {
                    name: def.name.clone(),
                    role: def.role.clone(),
                    description: def.description.clone(),
                    source: source_str,
                    inputs,
                    steps,
                },
            },
            mode,
        );
    } else {
        let source_str = match &source {
            AgentSource::Builtin => "builtin".to_string(),
            AgentSource::Custom(p) => format!("custom ({})", p.display()),
        };

        println!("Agent: {}", def.name);
        println!("  Role:        {}", def.role);
        println!("  Description: {}", def.description);
        println!("  Source:      {}", source_str);

        if !def.inputs.is_empty() {
            println!("\n  Inputs:");
            for input in &def.inputs {
                let req = if input.is_required {
                    "required"
                } else {
                    "optional"
                };
                let default = input
                    .default
                    .as_ref()
                    .map(|d| format!(" (default: {})", d))
                    .unwrap_or_default();
                println!(
                    "    - {} ({}){} - {}",
                    input.name, req, default, input.description
                );
            }
        }

        println!("\n  Steps:");
        for (i, step) in def.steps.iter().enumerate() {
            let step_type = step.step_type.to_string();
            println!("    {}. [{}] {}", i + 1, step_type, step.description);
            if let Some(content) = &step.content {
                let truncated: String = content
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(80)
                    .collect();
                println!("       Content: {}...", truncated);
            }
            if let Some(cmd) = &step.cmd {
                println!("       Cmd: {}", cmd.join(" "));
            }
        }
    }

    Ok(())
}
