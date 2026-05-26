use crate::agent::schema::AgentSource;
use crate::core::output::{Output, OutputMode};
use anyhow::Result;
use serde::Serialize;

pub fn run(mode: OutputMode) -> Result<()> {
    let agents = crate::agent::loader::list_all_agents();

    if mode.is_json() {
        #[derive(Serialize)]
        struct AgentInfo {
            name: String,
            role: String,
            description: String,
            source: String,
        }
        #[derive(Serialize)]
        struct ListOutput {
            status: String,
            agents: Vec<AgentInfo>,
        }
        let info: Vec<AgentInfo> = agents
            .iter()
            .map(|(def, source)| AgentInfo {
                name: def.name.clone(),
                role: def.role.clone(),
                description: def.description.clone(),
                source: source_to_string(source),
            })
            .collect();
        Output::print(
            &ListOutput {
                status: "ok".to_string(),
                agents: info,
            },
            mode,
        );
    } else {
        println!("Available agents:");
        for (def, source) in &agents {
            println!();
            println!("  {}", def.name);
            println!("    Role:        {}", def.role);
            println!("    Description: {}", def.description);
            println!("    Source:      {}", source_to_string(source));
        }
    }

    Ok(())
}

fn source_to_string(source: &AgentSource) -> String {
    match source {
        AgentSource::Builtin => "builtin".to_string(),
        AgentSource::Custom(path) => format!("custom ({})", path.display()),
    }
}
