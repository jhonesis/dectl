use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum InitLevel {
    Level1,
    Level2,
    Level3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum ProjectType {
    #[default]
    Other,
    Api,
    Cli,
    Microservice,
}

impl ProjectType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "api" => Some(Self::Api),
            "cli" => Some(Self::Cli),
            "microservice" => Some(Self::Microservice),
            "other" | "" => Some(Self::Other),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Other => "other",
            Self::Api => "api",
            Self::Cli => "cli",
            Self::Microservice => "microservice",
        }
    }
}

pub struct Templates;

impl Templates {
    pub fn project_toml_l1() -> &'static str {
        include_str!("txt/project.toml")
    }

    pub fn project_isa() -> &'static str {
        include_str!("txt/project.isa.md")
    }

    pub fn auto_fill_task() -> &'static str {
        include_str!("txt/auto_fill_task.md")
    }

    pub fn level1() -> Vec<(&'static str, &'static str)> {
        vec![
            (".dec/.gitignore", include_str!("txt/gitignore.dec")),
            (".dec/config/project.toml", include_str!("txt/project.toml")),
            (
                ".dec/isa/project.isa.md",
                include_str!("txt/project.isa.md"),
            ),
        ]
    }

    pub fn level2() -> Vec<(&'static str, &'static str)> {
        let mut files = Self::level1();
        files.extend([
            (".dec/decisions/.gitkeep", ""),
            (
                ".dec/workflows/implement_feature.yaml",
                include_str!("txt/workflow_implement_feature.yaml"),
            ),
            (
                ".dec/workflows/design_architecture.yaml",
                include_str!("txt/workflow_design_architecture.yaml"),
            ),
            (
                ".dec/workflows/execute_task.yaml",
                include_str!("txt/workflow_execute_task.yaml"),
            ),
            (
                ".dec/prompts/system/base.md",
                include_str!("txt/system_base.md"),
            ),
            (
                ".dec/prompts/system/integration.md",
                include_str!("txt/system_integration.md"),
            ),
            (
                ".dec/state/progress.json",
                include_str!("txt/progress.json"),
            ),
            (
                ".dec/state/last_session.md",
                include_str!("txt/last_session.md"),
            ),
            (".dec/sdd/SKILL.md", include_str!("txt/sdd_skill.md")),
            (
                ".dec/sdd/references/templates.md",
                include_str!("txt/sdd_templates.md"),
            ),
            (
                ".dec/sdd/references/examples.md",
                include_str!("txt/sdd_examples.md"),
            ),
        ]);
        files
    }

    pub fn level3() -> Vec<(&'static str, &'static str)> {
        let mut files = Self::level2();
        files.extend([
            (
                ".dec/isa/architecture.isa.md",
                include_str!("txt/architecture.isa.md"),
            ),
            (
                ".dec/prompts/tasks/implement_feature.md",
                include_str!("txt/task_implement_feature.md"),
            ),
            (
                ".dec/prompts/tasks/write_tests.md",
                include_str!("txt/task_write_tests.md"),
            ),
            (
                ".dec/prompts/tasks/review_code.md",
                include_str!("txt/task_review_code.md"),
            ),
            (
                ".dec/prompts/tasks/document_module.md",
                include_str!("txt/task_document_module.md"),
            ),
            (
                ".dec/knowledge/glossary.md",
                include_str!("txt/knowledge_glossary.md"),
            ),
            (
                ".dec/knowledge/constraints.md",
                include_str!("txt/knowledge_constraints.md"),
            ),
        ]);
        files
    }

    pub fn files_for_level(level: InitLevel) -> Vec<(&'static str, &'static str)> {
        let mut files = match level {
            InitLevel::Level1 => Self::level1(),
            InitLevel::Level2 => Self::level2(),
            InitLevel::Level3 => Self::level3(),
        };
        files.push(("AGENTS.md", include_str!("txt/agents.md")));
        files
    }

    pub fn workflows_for_type(project_type: ProjectType) -> Vec<(&'static str, &'static str)> {
        match project_type {
            ProjectType::Api => vec![
                (
                    ".dec/workflows/test_api.yaml",
                    include_str!("txt/workflow_test_api.yaml"),
                ),
                (
                    ".dec/workflows/document_endpoints.yaml",
                    include_str!("txt/workflow_document_endpoints.yaml"),
                ),
                (
                    ".dec/workflows/run_migrations.yaml",
                    include_str!("txt/workflow_run_migrations.yaml"),
                ),
            ],
            ProjectType::Cli => vec![
                (
                    ".dec/workflows/build_release.yaml",
                    include_str!("txt/workflow_build_release.yaml"),
                ),
                (
                    ".dec/workflows/document_args.yaml",
                    include_str!("txt/workflow_document_args.yaml"),
                ),
            ],
            ProjectType::Microservice => vec![
                (
                    ".dec/workflows/service_discovery.yaml",
                    include_str!("txt/workflow_service_discovery.yaml"),
                ),
                (
                    ".dec/workflows/dockerize.yaml",
                    include_str!("txt/workflow_dockerize.yaml"),
                ),
                (
                    ".dec/workflows/inter_service_comm.yaml",
                    include_str!("txt/workflow_inter_service_comm.yaml"),
                ),
            ],
            ProjectType::Other => vec![],
        }
    }

    pub fn system_prompt_for_type(
        project_type: ProjectType,
    ) -> Option<(&'static str, &'static str)> {
        match project_type {
            ProjectType::Api => Some((
                ".dec/prompts/system/api.md",
                include_str!("txt/system_prompt_api.md"),
            )),
            ProjectType::Cli => Some((
                ".dec/prompts/system/cli.md",
                include_str!("txt/system_prompt_cli.md"),
            )),
            ProjectType::Microservice => Some((
                ".dec/prompts/system/microservice.md",
                include_str!("txt/system_prompt_microservice.md"),
            )),
            ProjectType::Other => None,
        }
    }
}
