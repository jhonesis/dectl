# Project Specification — dectl (Dev Environment Control)
> *Technology-agnostic. Describes WHAT to build, not HOW.*
> *Version: 1.0 | Status: Draft | Last updated: 2026-05-13*

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
- Agent coordination (Phase 2)

---

## Open Questions

- [x] Should `dectl project scan` respect `.gitignore` rules, or use its own ignore list? → **Respects `.gitignore`**
- [x] Should memory entries support markdown formatting, or be plain text only? → **Markdown supported**
