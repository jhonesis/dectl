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
    pub fn agents_md() -> &'static str {
        Self::AGENTS_MD
    }

    pub fn project_toml_l1() -> &'static str {
        Self::PROJECT_TOML_L1
    }

    pub fn project_isa() -> &'static str {
        Self::PROJECT_ISA
    }

    pub fn level1() -> Vec<(&'static str, &'static str)> {
        vec![
            (".dec/.gitignore", Self::GITIGNORE_L1),
            (".dec/config/project.toml", Self::PROJECT_TOML_L1),
            (".dec/isa/project.isa.md", Self::PROJECT_ISA),
        ]
    }

    pub fn level2() -> Vec<(&'static str, &'static str)> {
        let mut files = Self::level1();
        files.extend([
            (".dec/decisions/.gitkeep", ""),
            (
                ".dec/workflows/implement_feature.yaml",
                Self::WORKFLOW_IMPLEMENT_FEATURE,
            ),
            (
                ".dec/workflows/design_architecture.yaml",
                Self::WORKFLOW_DESIGN_ARCHITECTURE,
            ),
            (".dec/prompts/system/base.md", Self::SYSTEM_BASE),
            (
                ".dec/prompts/system/integration.md",
                Self::SYSTEM_INTEGRATION,
            ),
            (".dec/state/progress.json", Self::PROGRESS_JSON),
            (".dec/state/last_session.md", Self::LAST_SESSION),
        ]);
        files
    }

    pub fn level3() -> Vec<(&'static str, &'static str)> {
        let mut files = Self::level2();
        files.extend([
            (".dec/isa/architecture.isa.md", Self::ARCHITECTURE_ISA),
            (
                ".dec/prompts/tasks/implement_feature.md",
                Self::TASK_IMPLEMENT_FEATURE,
            ),
            (".dec/prompts/tasks/write_tests.md", Self::TASK_WRITE_TESTS),
            (".dec/prompts/tasks/review_code.md", Self::TASK_REVIEW_CODE),
            (
                ".dec/prompts/tasks/document_module.md",
                Self::TASK_DOCUMENT_MODULE,
            ),
            (".dec/knowledge/glossary.md", Self::KNOWLEDGE_GLOSSARY),
            (".dec/knowledge/constraints.md", Self::KNOWLEDGE_CONSTRAINTS),
        ]);
        files
    }

    pub fn files_for_level(level: InitLevel) -> Vec<(&'static str, &'static str)> {
        let mut files = match level {
            InitLevel::Level1 => Self::level1(),
            InitLevel::Level2 => Self::level2(),
            InitLevel::Level3 => Self::level3(),
        };
        files.push(("AGENTS.md", Self::AGENTS_MD));
        files
    }

    const GITIGNORE_L1: &str = r#"# dectl — files that should not be versioned

# Personal local state (do not share with the team)
state/local_*.json

# Just in case — these files should NEVER be inside .dec/
*.env
*.key
*.pem
*.secret
secrets.*
.env.*
"#;

    const PROJECT_TOML_L1: &str = r#"# dectl project configuration
# This file defines the project for the model and tools.
# The model should read it at the start of every session.

[dec]
schema_version = "1.0"

[project]
name = "project-name"
# Project type: api | cli | microservice | monolith | library | other
type = "other"
description = "Brief project description in one sentence."

[stack]
# List of main project technologies
languages = []
frameworks = []
databases = []
tools = []

[conventions]
# Special conventions the model must follow for this project.
rules = []
"#;

    const PROJECT_ISA: &str = r#"# ISA: [Project Name]
> **For the model**: Read this document before making any important decisions.
> Update this file when the vision or scope change significantly.
> If anything here contradicts what you see in the code, ask the developer before assuming.

---

## Vision
<!-- One sentence: what the project is and for whom. -->


## Main Objective
<!-- What concrete problem it solves and how success is measured. -->


## Scope (what it DOES include)
<!-- Concrete list of what the project builds. -->
-
-

## Non-Goals (what it does NOT include)
<!-- Explicit list of what this project does NOT do. Prevents scope creep. -->
-
-

## Tech Stack
<!-- Main technologies. Details are in config/project.toml. -->


## Known Constraints
<!-- Technical, time, or resource limitations the model must respect. -->


## Main Risks
<!-- The 2-3 most important risks. Brief. -->
1.
2.
"#;

    const WORKFLOW_IMPLEMENT_FEATURE: &str = r#"name: implement_feature
description: Implement a complete feature with tests and documentation
inputs:
  - name: feature_name
    description: Name of the feature (e.g. "user_authentication", "payment_processing")
    required: true
  - name: module
    description: Module or folder where it will be implemented
    required: true
  - name: include_tests
    description: Generate tests automatically (true/false)
    required: false
    default: "true"

steps:
  - type: prompt
    description: Load project context
    content: |
      Read .dec/isa/project.isa.md and .dec/config/project.toml.
      Read .dec/decisions/ to understand architectural constraints.
      Confirm you understand the project before continuing.

  - type: action
    description: Search for relevant decisions in memory
    cmd: ["dectl", "memory", "search", "{{feature_name}}"]

  - type: prompt
    description: Design the implementation
    content: |
      Design the implementation of "{{feature_name}}" in the "{{module}}" module.
      - Follow conventions in .dec/config/project.toml
      - Respect decisions in .dec/decisions/
      - Describe the files you will create/modify before doing so

  - type: prompt
    description: Implement
    content: |
      Implement "{{feature_name}}" in "{{module}}".
      Include tests: {{include_tests}}.
      After each file, confirm it compiles/passes lint.

  - type: action
    description: Record decisions made during implementation
    cmd: ["dectl", "memory", "add", "Implemented feature {{feature_name}} in {{module}}"]

  - type: write
    description: Update project state
    path: .dec/state/last_session.md
    content: |
      # Last Session
      **Date**: (fill in)
      **What was done**: Implemented feature {{feature_name}} in {{module}}
      **Pending**: (fill in)
      **Decisions made**: (fill in)
"#;

    const WORKFLOW_DESIGN_ARCHITECTURE: &str = r#"name: design_architecture
description: Guide the model to design or review the project architecture
inputs:
  - name: focus
    description: Specific aspect to design (e.g. "auth", "database", "api") or "general" for full architecture
    required: false
    default: "general"

steps:
  - type: prompt
    description: Load full context
    content: |
      Read .dec/isa/project.isa.md, .dec/config/project.toml and .dec/decisions/.
      Identify the main components and their responsibilities.
      Focus of this session: {{focus}}.

  - type: action
    description: Search for previous architecture decisions
    cmd: ["dectl", "memory", "search", "architecture"]

  - type: prompt
    description: Propose architecture
    content: |
      Propose the architecture for "{{focus}}".
      Include: modules, responsibilities, main flows and trade-offs.
      If there are previous decisions in decisions/ that apply, reference them.

  - type: write
    description: Document the architecture decision
    path: .dec/decisions/XXXX-architecture-{{focus}}.md
    content: |
      # [XXXX] Architecture: {{focus}}
      **Date**: (fill in)
      **Status**: active

      ## Context
      (fill in)

      ## Decision
      (fill in)

      ## Alternatives Considered
      (fill in)

      ## Justification
      (fill in)

      ## Consequences
      (fill in)

  - type: prompt
    description: Update architecture ISA
    content: |
      Update .dec/isa/architecture.isa.md with the designed architecture.
      Then run: dectl memory add "Architecture {{focus}} designed and documented"
"#;

    const SYSTEM_BASE: &str = r#"# System Prompt Base — [Project Name]
> **Instructions for the model**: This prompt defines your behavior for this project.
> The developer can update it at any time. Re-read if you receive contradictory instructions.

---

## Project Context
You are working on [project name]. Read .dec/isa/project.isa.md to understand what you are building.

## Expected Behavior

**Before acting**:
- Read the relevant context in .dec/ before making important decisions
- Consult .dec/decisions/ before proposing architectural changes
- If something is unclear, ask before assuming

**When writing code**:
- Follow conventions in .dec/config/project.toml
- Respect constraints in .dec/knowledge/constraints.md (if it exists)
- Use terms defined in .dec/knowledge/glossary.md (if it exists)

**When completing a task**:
- Update .dec/state/progress.json if you completed a feature
- Update .dec/state/last_session.md with a summary of what was done
- Record important decisions with: dectl memory add "..."

## What you must NOT do
- Invent domain terms not in the glossary
- Propose changes that contradict decisions in .dec/decisions/
- Assume undocumented requirements — ask
"#;

    const SYSTEM_INTEGRATION: &str = r#"# Instrucciones de Sesión — [Project Name]
> **Para el modelo**: Lee y sigue estas instrucciones en cada sesión de trabajo.
> Actualiza este archivo si el equipo quiere cambiar el comportamiento del modelo.

---

## Al iniciar sesión

1. Lee `.dec/config/project.toml` y `.dec/isa/project.isa.md` para entender el proyecto
2. Lee `.dec/state/last_session.md` y retoma desde "Próximo paso recomendado"
3. Ejecuta `dectl project info --json` y escala al developer si hay warnings
4. Confirma en 2-3 líneas qué entendiste antes de preguntar qué hacer hoy

## Antes de actuar

1. Para cambios de arquitectura: lee `.dec/decisions/` primero
2. Para implementar una feature: busca su workflow en `.dec/workflows/`
3. Para términos de dominio: consulta `.dec/knowledge/glossary.md` si existe
4. Describe lo que vas a hacer antes de hacerlo — nunca actúes en silencio

## Agentes disponibles

Usa `dectl agent list` para ver todos los agentes (built-in + custom).

El proyecto incluye estos agentes built-in:
- **coder**: implementa código siguiendo las convenciones del stack
- **reviewer**: revisa código en busca de bugs y desviaciones
- **researcher**: busca contexto en memoria y decisiones previas
- **documenter**: genera o actualiza documentación técnica

Para invocar un agente:
```
dectl agent run <tipo> --task "<descripción de la tarea>"
dectl agent describe <tipo>     # ver definición completa
dectl agent run --parallel <t1>,<t2> --task "<desc>"  # ejecutar en paralelo
```

Usa agentes cuando la tarea sea autónoma y especializada. El modelo principal mantiene el contexto global mientras el agente ejecuta.

## Al completar una tarea

1. Si completaste o avanzaste una feature: actualiza `.dec/state/progress.json`
2. Para decisiones importantes: ejecuta `dectl memory add "[resumen de la decisión]"`
3. Para decisiones arquitectónicas: crea `.dec/decisions/XXXX-nombre.md`

## Al finalizar sesión

1. Ejecuta `dectl session end` para automatizar el cierre:
   - Genera `.dec/state/last_session.md` automáticamente
   - Sincroniza cambios git a `progress.json`
   - Captura decisiones y las guarda en memoria
   - Sincroniza cambios del stack con `project.toml`
   - Registra actividad de agentes
2. O manualmente:
   - Escribe `.dec/state/last_session.md` (qué se hizo, qué quedó pendiente, decisiones, próximo paso)
   - Ejecuta `dectl memory add "Sesión [fecha]: [resumen en una línea]"`
"#;

    const PROGRESS_JSON: &str = r#"{
  "_comment": "Project feature status. Update when completing tasks.",
  "schema_version": "1.0",
  "updated_at": "",
  "features": []
}
"#;

    const LAST_SESSION: &str = r#"# Last Session
> Update this file at the end of each work session.
> The model should read it at the start of a new session to resume context.

**Date**: (no sessions yet)
**What was done**: —
**What's pending**: —
**Decisions made**: —
**Recommended next step**: Complete project initialization in .dec/isa/project.isa.md
"#;

    const ARCHITECTURE_ISA: &str = r#"# ISA: Architecture — [Project Name]
> **For the model**: Read this document before proposing architectural changes.
> Also consult decisions/ to understand why the architecture is the way it is.

---

## Architecture Vision
<!-- 2-3 sentence description of the overall architecture. -->


## Main Modules / Components
<!-- List of components with a one-line responsibility each. -->
| Component | Responsibility |
|-----------|----------------|
|           |                |

## Main Flows
<!-- The 2-3 most important system flows, described briefly. -->

### Flow 1: [name]
<!-- Step-by-step flow description. -->

## Key Design Decisions
<!-- Decisions that most impact the architecture. Reference decisions/ for details. -->
- [Decision]: [one-line consequence]

## Diagram (optional)
<!-- ASCII diagram or textual description of the architecture. -->

## What must NOT change without reviewing decisions/
<!-- Parts of the architecture with strong constraints. -->
-
"#;

    const TASK_IMPLEMENT_FEATURE: &str = r#"# Prompt: Implement Feature

## Context
You are implementing a new feature. Read `.dec/isa/project.isa.md` and `.dec/config/project.toml` first.

## Your task
1. Read `.dec/decisions/` to understand architectural constraints
2. Briefly design the implementation before writing code
3. Implement the feature following project conventions
4. If `include_tests` is true, generate tests for the new functionality
5. Confirm the code compiles and passes lint

## Constraints
- Follow conventions in `config/project.toml` → `[conventions]`
- Do not modify files outside the assigned module without approval
- Consult `.dec/decisions/` before making architectural decisions

## When done
- Run `dectl memory add` with a summary of what you did
- Update `.dec/state/progress.json` if the feature is complete
"#;

    const TASK_WRITE_TESTS: &str = r#"# Prompt: Write Tests

## Context
You need to write tests for an existing functionality. Read the module first.

## Your task
1. Identify which functionalities need tests
2. Write tests covering normal cases and edge cases
3. Follow the project's testing framework (see `config/project.toml`)
4. Run the tests to confirm they pass

## Constraints
- Tests must be independent and executable in any order
- Do not hardcode paths — use environment variables or configuration
- Minimum coverage: happy path + main error cases

## When done
- Run all module tests to confirm nothing is broken
- Record with `dectl memory add` which tests you added
"#;

    const TASK_REVIEW_CODE: &str = r#"# Prompt: Code Review

## Context
You need to review code changes or proposed code. Focus on quality, not style.

## Your task
1. Read the proposed code or recent changes
2. Identify:
   - Potential bugs or unhandled edge cases
   - Security issues
   - Violations of project conventions
   - Opportunities for improvement
3. Provide constructive feedback with specific examples

## What to look for
- Logical errors or null handling
- Obvious performance issues
- Violations of architecture decisions/
- Missing tests for critical code

## When done
- Record with `dectl memory add` a summary of the review
- If there are critical issues, propose specific solutions
"#;

    const TASK_DOCUMENT_MODULE: &str = r#"# Prompt: Document Module

## Context
You need to document an existing module of the project.

## Your task
1. Read the complete module code
2. Identify:
   - The module's main responsibility
   - Public functions/methods and their contracts
   - Dependencies and side effects
3. Write clear documentation:
   - README.md in the module folder or docs/
   - Doc comments (/// in Rust, docstrings in Python)
   - Usage examples where helpful

## Constraints
- Documentation must be useful for someone who didn't write the code
- Don't document the what (the code already says it), document the why and how
- Keep documentation close to the code (comments, docstrings)

## When done
- Record with `dectl memory add` what you documented
"#;

    const KNOWLEDGE_GLOSSARY: &str = r#"# Project Glossary
> **For the model**: Define domain terms specific to this project here.
> Use these terms consistently. If you need to add one, consult the developer first.

---

## Terms

### [term]
Brief definition. One sentence.

### [term]
Brief definition. One sentence.

---

## Acronyms

| Acronym | Meaning |
|---------|---------|
|         |         |
"#;

    const KNOWLEDGE_CONSTRAINTS: &str = r#"# Project Constraints
> **For the model**: These constraints must be respected at all times.
> If a constraint changes, update this file and notify the team.

---

## Technical Constraints

### [title]
Description of the constraint and why it exists.
- Limit: [example]
- Impact: [what limitations it imposes on the code]

---

## Business Constraints

### [title]
Description of the business constraint.
- Limit: [example]
- Impact: [what limitations it imposes]

---

## Mandatory Conventions

- [ ] Rule 1
- [ ] Rule 2
- [ ] Rule 3
"#;

    const AGENTS_MD: &str = r#"# AGENTS.md — [PROJECT NAME]

> This project uses **dectl** (Dev Environment Control) with a structured `.dec/`
> directory that persists context, decisions, memory and workflows between sessions.
> Read this file and the `.dec/` directory completely before responding to any task.

---

## Session Cycle — Run at the start of every session

1. Read `.dec/config/project.toml` → project name, type, stack, conventions
2. Read `.dec/isa/project.isa.md` → vision, objectives and scope
3. If `.dec/isa/architecture.isa.md` exists → read before any architectural decision
4. Read `.dec/state/last_session.md` → resume from where you left off
5. If `.dec/decisions/` has files → read them before proposing structural changes
6. If `.dec/prompts/system/base.md` exists → read it for behavioral guidelines
7. Run `dectl project info --json` → verify schema compliance and project metadata

Do not skip these steps even for simple requests. Context is always required before acting.

---

## dectl Commands Reference

### Memory
```bash
dectl memory add "<text>" [--tags t1,t2]     # save a decision, note or fact
dectl memory list [--limit <n>]              # list all memories
dectl memory search "<query>"                 # search by keyword
dectl memory show <id>                        # show a specific entry
dectl memory delete <id> [--hard]             # soft-delete (or --hard for permanent)
dectl memory edit <id>                        # open entry in $EDITOR
```

### Project
```bash
dectl project init [--standard|--full]        # initialize .dec/ structure
dectl project info [--json]                   # show project metadata + warnings
dectl project scan [--depth <n>]              # file tree (gitignore-aware)
dectl project context [--max-tokens <n>]      # compact summary for stateless environments
```

### Workflows
```bash
dectl workflow list                           # list available workflows
dectl workflow describe <name>                # show workflow schema
dectl workflow run <name> [--var k=v] [--dry-run] [--from-step N]
```

### Protocol
```bash
dectl exec-from-file <path>                   # execute commands from a file
```

---

## When to Use dectl

| Situation | Command |
|-----------|---------|
| Architectural decision made | `dectl memory add "Decision: ..."` |
| Library or technology chosen | `dectl memory add "Stack: ..."` |
| Formal decision to record | create `.dec/decisions/XXXX-title.md` |
| Significant feature completed | `dectl memory add "Feature X done: ..."` |
| Run a structured process | `dectl workflow run <name>` |
| Need a compact project summary | `dectl project context` |

---

## Behavior Rules

- Read `.dec/` before acting, not after.
- Consult `.dec/decisions/` before proposing architecture changes.
- Follow `.dec/workflows/` as a thinking guide for complex tasks.
- After completing a significant task, update `.dec/state/progress.json`.
- At the end of every session, update `.dec/state/last_session.md`.

---

## Project Structure

```
.dec/
├── config/
│   └── project.toml          ← name, type, stack, conventions
├── isa/
│   ├── project.isa.md        ← vision, objectives, scope, non-goals
│   └── architecture.isa.md  ← modules, flows, trade-offs (if exists)
├── decisions/
│   └── *.md                  ← ADR-style decision records
├── workflows/
│   └── *.yaml                ← executable step-by-step processes
├── prompts/
│   ├── system/
│   │   └── base.md           ← behavioral guidelines (if exists)
│   └── tasks/
│       └── *.md              ← task-specific prompts (level 3)
├── knowledge/
│   ├── glossary.md           ← domain terms (if exists)
│   └── constraints.md        ← project constraints (if exists)
└── state/
    ├── progress.json         ← feature status tracking
    └── last_session.md      ← session continuity log
```

---

## If the Project Is New (First Session)

If `project.toml` has placeholder values (e.g. "project-name") or `project.isa.md`
has placeholder content (e.g. "[Project Name]"):

1. **Read `.dec/prompts/tasks/auto-fill.md`** for detailed fill instructions.
2. **Auto-detect the stack**: Read the project's source code, config files,
   dependencies, and imports to determine the full tech stack. Be thorough:
   languages, frameworks, databases, tools, testing frameworks, CI/CD.
3. **Analyze the project**: Read README.md, docs/, specs/, and any existing
   documentation to extract project name, description, vision, and objectives.
4. **Fill `.dec/config/project.toml`**: Update `[project].description`,
   `[stack].frameworks`, `[stack].databases`, `[stack].tools`. Never remove
   existing values — only add what is missing.
5. **Fill `.dec/isa/project.isa.md`**: Complete Vision, Main Objective, Scope,
   Non-Goals, Tech Stack, Known Constraints, and Main Risks.
6. **Log it**: Run `dectl memory add "Project initialized: [name] — [one-liner]"`.
7. **Update progress**: Set `updated_at` in `.dec/state/progress.json`.

Do NOT ask the user for what you can determine by reading the project code.
Only ask if something is genuinely ambiguous or requires human judgment.
"#;

    const AUTO_FILL_TASK: &str = r#"# Task: Auto-Fill Project Context

## Why this file exists
This project was initialized with `dectl project init`. The `.dec/` structure
is ready but some files contain placeholder values that need to be filled based
on the actual project code and documentation.

## Your task
Read this entire file. Then fill the missing context by analyzing the project:

### Step 1 — Detect the stack
Read the project source code, config files, and dependencies:
- Languages: detect from source files and config (Cargo.toml, package.json, go.mod, etc.)
- Frameworks: detect from imports, dependencies, and code patterns
- Databases: detect from ORM imports, migration files, connection strings
- Tools: detect from config files (Docker, CI/CD, linters, formatters, etc.)

### Step 2 — Analyze project context
Read documentation files for project intent:
- README.md → project name, description, what it does
- docs/ → architecture, design decisions, vision
- specs/ → requirements, acceptance criteria
- Other .md files → additional context

### Step 3 — Fill .dec/config/project.toml
Set:
- [project].description → one-sentence summary
- [stack].frameworks → detected from code (not just config files)
- [stack].databases → detected from imports/config
- [stack].tools → detected from config files

Do NOT modify [project].name, [project].type, [stack].languages,
[dec], or [conventions] — those are already set.

### Step 4 — Fill .dec/isa/project.isa.md
Complete:
- Vision → one sentence: what the project is and for whom
- Main Objective → what problem it solves and success metrics
- Scope → concrete list of what the project builds
- Non-Goals → what it explicitly does NOT do
- Tech Stack → main technologies (summary from project.toml)
- Known Constraints → technical, time, or resource limitations
- Main Risks → the 2-3 most important risks

### Step 5 — Log the initialization
```bash
dectl memory add "Project initialized: [name] — [one-line description]"
```

## What NOT to do
- Do NOT remove existing data from project.toml or project.isa.md
- Do NOT guess frameworks if unsure — leave as empty array or ask
- Do NOT modify files outside of .dec/ unless explicitly requested
- Do NOT run session end — the project is not ready for that yet

## Verification
After filling, run `dectl project info --json` to verify the setup is valid.
"#;

    pub fn auto_fill_task() -> &'static str {
        Self::AUTO_FILL_TASK
    }

    pub fn workflows_for_type(project_type: ProjectType) -> Vec<(&'static str, &'static str)> {
        match project_type {
            ProjectType::Api => vec![
                (".dec/workflows/test_api.yaml", Self::WORKFLOW_TEST_API),
                (
                    ".dec/workflows/document_endpoints.yaml",
                    Self::WORKFLOW_DOCUMENT_ENDPOINTS,
                ),
                (
                    ".dec/workflows/run_migrations.yaml",
                    Self::WORKFLOW_RUN_MIGRATIONS,
                ),
            ],
            ProjectType::Cli => vec![
                (
                    ".dec/workflows/build_release.yaml",
                    Self::WORKFLOW_BUILD_RELEASE,
                ),
                (
                    ".dec/workflows/document_args.yaml",
                    Self::WORKFLOW_DOCUMENT_ARGS,
                ),
            ],
            ProjectType::Microservice => vec![
                (
                    ".dec/workflows/service_discovery.yaml",
                    Self::WORKFLOW_SERVICE_DISCOVERY,
                ),
                (".dec/workflows/dockerize.yaml", Self::WORKFLOW_DOCKERIZE),
                (
                    ".dec/workflows/inter_service_comm.yaml",
                    Self::WORKFLOW_INTER_SERVICE_COMM,
                ),
            ],
            ProjectType::Other => vec![],
        }
    }

    pub fn system_prompt_for_type(
        project_type: ProjectType,
    ) -> Option<(&'static str, &'static str)> {
        match project_type {
            ProjectType::Api => Some((".dec/prompts/system/api.md", Self::SYSTEM_PROMPT_API)),
            ProjectType::Cli => Some((".dec/prompts/system/cli.md", Self::SYSTEM_PROMPT_CLI)),
            ProjectType::Microservice => Some((
                ".dec/prompts/system/microservice.md",
                Self::SYSTEM_PROMPT_MICROSERVICE,
            )),
            ProjectType::Other => None,
        }
    }

    const WORKFLOW_TEST_API: &str = r#"name: test_api
description: Test a REST API endpoint with proper validation
inputs:
  - name: method
    description: HTTP method (GET, POST, PUT, DELETE)
    required: true
  - name: endpoint
    description: API endpoint path
    required: true
  - name: body
    description: Request body (JSON string)
    required: false
    default: "{}"
  - name: expected_status
    description: Expected HTTP status code
    required: false
    default: "200"

steps:
  - type: prompt
    description: Load API context
    content: |
      Read .dec/config/project.toml to understand the API structure.
      Check if there's OpenAPI/Swagger documentation.

  - type: action
    description: Execute API request
    cmd: ["curl", "-X", "{{method}}", "{{endpoint}}", "-H", "Content-Type: application/json"{{#if body}}, "-d", "{{body}}"{{/if}}]

  - type: prompt
    description: Validate response
    content: |
      Validate the response matches expected status {{expected_status}}.
      Check for proper error handling in the response.

  - type: action
    description: Log test result
    cmd: ["dectl", "memory", "add", "API test: {{method}} {{endpoint}} - status {{expected_status}}"]
"#;

    const WORKFLOW_DOCUMENT_ENDPOINTS: &str = r#"name: document_endpoints
description: Generate API documentation from code or schema
inputs:
  - name: format
    description: Output format (openapi, swagger, markdown)
    required: false
    default: "markdown"

steps:
  - type: prompt
    description: Analyze API structure
    content: |
      Scan the codebase for route handlers, controllers, or route definitions.
      Identify: endpoints, methods, request/response schemas, auth requirements.

  - type: prompt
    description: Generate documentation
    content: |
      Generate {{format}} documentation based on the analysis.
      Include: endpoint, method, path params, query params, request body, response codes.

  - type: write
    description: Save API docs
    path: .dec/knowledge/api-docs.md
    content: |
      # API Documentation
      (generated content)
"#;

    const WORKFLOW_RUN_MIGRATIONS: &str = r#"name: run_migrations
description: Run database migrations safely
inputs:
  - name: direction
    description: up or down
    required: false
    default: "up"
  - name: name
    description: Migration name (for down migrations)
    required: false
    default: ""

steps:
  - type: prompt
    description: Check migration status
    content: |
      Check current migration state in the database.
      Identify pending migrations.

  - type: action
    description: Run migrations
    cmd: ["dectl", "exec-from-file", ".dec/workflows/run_migrations.yaml"]

  - type: action
    description: Verify migration
    cmd: ["dectl", "memory", "add", "Database migration: {{direction}} - {{name}}"]
"#;

    const WORKFLOW_BUILD_RELEASE: &str = r#"name: build_release
description: Build and release a CLI tool
inputs:
  - name: version
    description: Semantic version (e.g., v1.0.0)
    required: true
  - name: target
    description: Target platform (linux, macos, windows)
    required: false
    default: "all"

steps:
  - type: prompt
    description: Load build configuration
    content: |
      Read .dec/config/project.toml for build targets and tools.
      Check existing release artifacts.

  - type: action
    description: Run build
    cmd: ["cargo", "build", "--release"]

  - type: action
    description: Create release
    cmd: ["dectl", "memory", "add", "Release {{version}} built for {{target}}"]

  - type: prompt
    description: Update changelog
    content: |
      Generate release notes based on recent commits.
      Update CHANGELOG.md if it exists.
"#;

    const WORKFLOW_DOCUMENT_ARGS: &str = r#"name: document_args
description: Generate CLI argument documentation
inputs:
  - name: command
    description: Command to document (or "all")
    required: false
    default: "all"

steps:
  - type: action
    description: Get command help
    cmd: ["{{binary_name}}", "--help"]

  - type: prompt
    description: Generate documentation
    content: |
      Parse the help output and generate markdown documentation.
      Include: description, positional args, options, examples.

  - type: write
    description: Save CLI docs
    path: .dec/knowledge/cli-commands.md
    content: |
      # CLI Commands
      (generated content)
"#;

    const WORKFLOW_SERVICE_DISCOVERY: &str = r#"name: service_discovery
description: Discover and register microservices
inputs:
  - name: service_name
    description: Name of the service to register
    required: true
  - name: port
    description: Port the service listens on
    required: true

steps:
  - type: prompt
    description: Analyze service topology
    content: |
      Map out the microservices architecture.
      Identify: service dependencies, communication patterns, data flow.

  - type: action
    description: Register service
    cmd: ["dectl", "memory", "add", "Service {{service_name}} registered on port {{port}}"]

  - type: write
    description: Document service registry
    path: .dec/knowledge/service-registry.md
    content: |
      # Service Registry
      (generated content)
"#;

    const WORKFLOW_DOCKERIZE: &str = r#"name: dockerize
description: Containerize a microservice
inputs:
  - name: service_name
    description: Name of the service
    required: true

steps:
  - type: prompt
    description: Analyze service requirements
    content: |
      Identify runtime dependencies, ports, environment variables.
      Determine optimal base image.

  - type: prompt
    description: Generate Dockerfile
    content: |
      Create a Dockerfile with multi-stage build if needed.
      Include: build stage, runtime stage, health checks.

  - type: write
    description: Save Docker config
    path: .dec/docker/{{service_name}}/Dockerfile
    content: |
      # Generated Dockerfile
"#;

    const WORKFLOW_INTER_SERVICE_COMM: &str = r#"name: inter_service_comm
description: Document inter-service communication patterns
inputs:
  - name: from_service
    description: Source service
    required: true
  - name: to_service
    description: Target service
    required: true

steps:
  - type: prompt
    description: Analyze communication
    content: |
      Identify: protocol (HTTP, gRPC, message queue), data format, auth mechanism.
      Document: endpoints, contracts, error handling.

  - type: write
    description: Save communication docs
    path: .dec/knowledge/service-communication.md
    content: |
      # Inter-Service Communication
      (generated content)
"#;

    const SYSTEM_PROMPT_API: &str = r#"# System Prompt — API Project
> **Instructions for the model**: You're working on an API server project.

---

## API-Specific Guidelines

**Endpoint Design**:
- RESTful conventions: resource-oriented URLs, proper HTTP methods
- Consistent error response format
- Proper status codes (200, 201, 400, 401, 404, 500)

**Security**:
- Validate all inputs
- Authenticate and authorize appropriately
- Never log sensitive data

**Documentation**:
- Document endpoints in .dec/knowledge/api-docs.md
- Keep OpenAPI spec updated if applicable
"#;

    const SYSTEM_PROMPT_CLI: &str = r#"# System Prompt — CLI Project
> **Instructions for the model**: You're working on a command-line tool project.

---

## CLI-Specific Guidelines

**User Experience**:
- Clear, helpful error messages
- Consistent output format
- Progress indicators for long operations

**Arguments**:
- Follow POSIX conventions for flags
- Provide --help for every command
- Use sensible defaults

**Documentation**:
- Document commands in .dec/knowledge/cli-commands.md
- Include usage examples
"#;

    const SYSTEM_PROMPT_MICROSERVICE: &str = r#"# System Prompt — Microservice Project
> **Instructions for the model**: You're working on a microservices architecture.

---

## Microservice-Specific Guidelines

**Service Design**:
- Single responsibility per service
- Independent deployability
- Stateless where possible

**Communication**:
- Document inter-service contracts
- Handle failures gracefully (circuit breakers, retries)
- Use appropriate protocols (HTTP, gRPC, messages)

**Infrastructure**:
- Configuration via environment variables
- Health checks for all services
- Logging aggregation

**Documentation**:
- Keep .dec/knowledge/service-registry.md updated
- Document Docker setup in .dec/docker/
"#;
}
