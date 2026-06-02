# Project Specification — dectl (Dev Environment Control)
> *Technology-agnostic. Describes WHAT to build, not HOW.*
> *Version: 1.0 | Status: Updated | Last updated: 2026-06-02*

---

## Overview

dec is a developer life OS: a system that gives any AI coding environment (or a human in the terminal) persistent memory, executable workflows, and structured project context. It consists of three actors with distinct responsibilities that communicate through files and system commands — no proprietary API, no specific model required.

---

## Context & Motivation

AI coding tools are stateless by design. Every session starts from zero. Developers repeat context, re-explain architecture decisions, and manually track project state across tools and sessions. Existing solutions like PAI are powerful but locked to a single model provider and a single runtime environment.

dec solves this by separating concerns cleanly: the model thinks, the CLI executes, and `.dec/` defines the context that connects them. Any model that can read files and run terminal commands can use dec on day one.

---

## Users & Personas

- **Solo developer**: works across multiple projects, uses different AI tools depending on the task, wants memory and context without cloud lock-in.
- **AI coding environment**: Claude Code, Gemini CLI, Qwen CLI, or any tool that reads files and runs commands. Consumes `.dec/` context and invokes `dectl` commands to persist state and execute workflows.
- **Team contributor** *(Phase 3)*: shares `.dec/` structure with teammates through version control, with personal memory kept separate.

---

## Functional Requirements

### REQ-001: Project initialization
**User Story**:
> As a developer, I want to initialize dec in any project directory so that the project gets a structured context folder ready to use.

**Acceptance Criteria**:
- WHEN the developer runs the init command in a directory THEN dec SHALL create a `.dec/` folder with the base structure (config, isa, decisions, workflows, prompts, knowledge, state)
- WHEN the `.dec/` folder already exists THEN dec SHALL abort and inform the developer without overwriting anything
- WHEN init completes THEN dec SHALL display a summary of what was created and the next recommended step
- WHEN the project has existing code THEN dec SHALL auto-detect languages from config files and write them to `.dec/config/project.toml`, then create `.dec/prompts/tasks/auto-fill.md` so the AI completes the semantic context (frameworks, tools, description, vision) in the first session
- WHEN the project is empty THEN dec SHALL offer interactive prompts for project name, type, languages, description, and vision (if TTY is available)
- WHEN the developer specifies `--type api|cli|microservice|other` THEN dec SHALL create type-specific workflows and prompts
- WHEN init completes on a non-empty project THEN dec SHALL create `AGENTS.md` at the project root

---

### REQ-002: Project context reading
**User Story**:
> As an AI coding environment, I want to read the project's full context from `.dec/` so that I can work with accurate information about the project's architecture, decisions, and state without the developer re-explaining it.

**Acceptance Criteria**:
- WHEN an AI environment opens a project with `.dec/` THEN it SHALL be able to read ISA, decisions, prompts, and state as plain Markdown files without any tooling
- WHEN the developer runs the project info command THEN dec SHALL display a human-readable summary of the current project context
- WHEN `.dec/` is missing or malformed THEN dec SHALL report specifically which files are absent or invalid

---

### REQ-003: Persistent memory
**User Story**:
> As a developer, I want to store and retrieve notes, decisions, and learnings across sessions so that context is never lost between AI sessions or project switches.

**Acceptance Criteria**:
- WHEN the developer or model adds a memory entry THEN dec SHALL persist it with a timestamp and optional tags
- WHEN the developer searches memory THEN dec SHALL return relevant entries matching the query
- WHEN the developer lists memory THEN dec SHALL display entries in reverse chronological order with IDs
- WHEN the developer requests a specific memory by ID THEN dec SHALL display its full content
- WHEN memory is stored THEN it SHALL persist across terminal sessions and machine restarts

---

### REQ-004: Executable workflows
**User Story**:
> As a developer, I want to define multi-step workflows in YAML so that complex recurring tasks can be executed consistently by the CLI, the model, or both.

**Acceptance Criteria**:
- WHEN the developer runs a workflow THEN dec SHALL execute each step in order
- WHEN a step is of type `action` THEN dec SHALL execute it as a system command
- WHEN a step is of type `write` THEN dec SHALL write the specified content to the specified file
- WHEN a step is of type `prompt` THEN dec SHALL output the prompt text for the model to act on and pause
- WHEN a workflow contains `action` steps and has not been trusted before THEN dec SHALL ask for confirmation once and record the decision locally
- WHEN a workflow fails at a step THEN dec SHALL report which step failed, why, and the current state of execution

---

### REQ-005: Workflow and project inspection
**User Story**:
> As a developer or AI environment, I want to list and inspect available workflows and project files so that I can understand what automation is available and navigate the project structure.

**Acceptance Criteria**:
- WHEN the developer lists workflows THEN dec SHALL display all workflows defined in `.dec/workflows/` with name and description
- WHEN the developer describes a workflow THEN dec SHALL display its steps, types, and expected inputs
- WHEN the developer scans the project THEN dec SHALL display the file tree of the current project directory, excluding common noise directories

---

### REQ-006: Structured CLI output
**User Story**:
> As an AI coding environment, I want all CLI commands to support machine-readable output so that I can parse results reliably without screen-scraping.

**Acceptance Criteria**:
- WHEN any dec command is run with the `--json` flag THEN dec SHALL output the result as valid JSON
- WHEN a command succeeds THEN the JSON output SHALL include a `status: "ok"` field and the relevant data
- WHEN a command fails THEN the JSON output SHALL include a `status: "error"` field and a human-readable `message`

---

### REQ-007: Global configuration
**User Story**:
> As a developer, I want to configure dec's global behavior once so that preferences apply across all projects without repeating setup.

**Acceptance Criteria**:
- WHEN dec is first run THEN it SHALL create a default config file at `~/.dectl/config.toml` if one does not exist
- WHEN the developer modifies the config file THEN dec SHALL reflect those changes on next invocation without restart
- WHEN a project-level config exists in `.dec/config/project.toml` THEN it SHALL override global config for that project

---

### REQ-008: Session end automation
**User Story**:
> As a developer or AI model, I want to run a single command at the end of a session so that all session context is automatically captured and persisted for the next session.

**Acceptance Criteria**:
- WHEN the developer runs `dectl session end` THEN dec SHALL perform five actions in sequence:
  1. Update `.dec/state/last_session.md` with a structured session summary (date, actions performed, pending items, decisions taken, next recommended step)
  2. Sync git changes to `.dec/state/progress.json` (mark features as done based on modified files, detect new features from commits)
  3. Capture uncaptured decisions and save them to memory (using pattern matching on commit messages and session files)
- WHEN a step fails THEN the remaining steps SHALL continue independently (one failure does not stop others)
- WHEN `--dry-run` is specified THEN dec SHALL preview all changes without writing any files
- WHEN `--skip-git` is specified THEN dec SHALL skip the git synchronization step without error
- WHEN no git repository exists THEN dec SHALL skip the git step gracefully (not an error)
- WHEN `--json` is specified THEN dec SHALL output a structured result with per-step status and decision count
- WHEN all steps fail THEN dec SHALL exit with a non-zero exit code
- WHEN at least one step succeeds THEN dec SHALL exit with code 0

---

### REQ-009: SDD Spec Generator (spec init)
**User Story**:
> As a developer or AI model, I want to generate the SDD methodology inside .dec/ so that the AI agent can follow a structured process (Build+Verify+Gate) to create specs/ documents.

**Acceptance Criteria**:
- WHEN the developer runs `dectl spec init` THEN dec SHALL ensure `.dec/sdd/` exists with SKILL.md, references/templates.md, and references/examples.md
- WHEN `.dec/sdd/` already exists THEN dec SHALL be idempotent and do nothing
- WHEN `spec init` runs THEN it SHALL update `.dec/config/project.toml` with `[specs] dir = "specs"`
- WHEN `spec init` runs THEN it SHALL update `.dec/isa/project.isa.md` with a link to "See specs/ for SDD artifacts"
- WHEN `spec init` runs with `--json` THEN dec SHALL output `{status: "ok", data: {message: ".dec/sdd/ ready", bridge: {project_toml: true, project_isa: true}, next: "Interview the user and create specs/"}}`
- WHEN `.dec/` does not exist THEN dec SHALL error with ".dec/ not found. Run `dectl project init` first."
- WHEN `.dec/config/project.toml` does not exist THEN dec SHALL error with clear message

---

## Non-Functional Requirements

- **Performance**: all CLI commands SHALL complete in under 500ms on a standard developer machine (excluding external command execution in workflows)
- **Portability**: the CLI binary SHALL run on Linux and macOS without runtime dependencies; Windows support is a stretch goal
- **Size**: the CLI binary SHALL not exceed 20MB
- **Reliability**: a failure in one workflow step SHALL never corrupt the memory database or project files
- **Discoverability**: every command and subcommand SHALL have a `--help` output with usage examples

---

## Out of Scope (v1)

- GUI or web dashboard (planned for later phase)
- Cloud sync or remote memory access
- Built-in model execution (dec does not run a model; it works alongside one)
- Multi-user collaboration features
- Windows native support (may work but not guaranteed)
- Semantic search / embeddings (Phase 2)
- Agent coordination (Phase 6) — see `specs/agents/spec.md`

## Implemented (post-v1)

- **Session Management** (Phase 4): `dectl session end` — automated session closure with 5 steps (summary, git sync, decisions, config sync, agent sync)
- **Auto-fill + Interactive Init** (Phase 3b): automatic stack detection, interactive prompts for empty projects, type-specific templates
- **Project Context** (Phase 3b): `dectl project context` — compact project summary for stateless AI environments
- **Config Sync** (Phase 5): `dectl session end` Paso 4 — automatic stack detection and project.toml merge
- **SDD Spec Generator** (Phase 8): `dectl spec init` — SDD methodology in .dec/sdd/, bridge updates, agent interview flow
- **Auto-Trust Interactive** (Phase 4): `dectl agent trust <type>` — trust an agent without running it, path canonicalization, improved `--non-interactive` error messages
- **Install Script**: `scripts/install.sh` — curl-based installation for Linux, macOS, and WSL
- **CI/CD Pipeline**: GitHub Actions workflow with fmt, clippy, test, and build steps

---

## Open Questions

- [x] Should `dectl project scan` respect `.gitignore` rules, or use its own ignore list? → **Respects `.gitignore`**
- [x] Should memory entries support markdown formatting, or be plain text only? → **Markdown supported**
