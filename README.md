# dectl — Dev Environment Control

> A model-agnostic developer life OS that gives any AI coding environment persistent memory, executable workflows, and structured project context.

> **The anchor moment**: Open a legacy project you haven't touched in months,
> run `dectl project init --standard`, open your AI, and the model already
> understands the project's architecture. No explanations. No setup.

```bash
# 1. Install dectl (macOS, Linux, WSL)
curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/scripts/install.sh | bash

# 2. Enter any project
cd your-legacy-project

# 3. Create automatic context
dectl project init --standard
# → Detects stack, analyzes code, generates .dec/

# 4. Open your AI and ask about the project
# "How is this project structured?"
# → The model already has all the context. It responds without you explaining anything.
```

<div style="text-align:center">
  <img src="output.gif" alt="dectl in action" width="700">
</div>

*No external APIs. No telemetry. No manual configuration.*

## The Problem

Every AI coding assistant (Claude Code, Gemini CLI, Qwen CLI, Ollama, or a human in a terminal) starts from scratch in each session. They forget:
- Your project architecture decisions
- Why you chose a particular approach
- The context of previous conversations
- your team's conventions and standards

**dectl** solves this by creating a structured `.dec/` directory that persists project knowledge between sessions—without requiring any specific AI model or provider.

## Features

### Persistent Memory
- Add important context: decisions, architecture notes, team conventions
- Search and retrieve across all stored memories
- Tag-based organization
- Per-project memory with auto-detection
- Soft-delete with `--hard` for permanent removal

### Executable Workflows
- Define reusable workflows in YAML
- Variable interpolation: `{{variable}}`
- Step types: `prompt`, `action`, `write`, `agent`
- `run_always: true` — steps that execute even if previous steps fail
- `--auto` flag to skip trust confirmation prompts
- Trust system for security (approves action steps per-project)
- `--from-step N` to resume workflows after fixing failures

### Project Context with Auto-fill
- Automatic stack detection (Rust, Node.js, Go, Python, Java, etc.)
- Auto-fill project configuration from existing project files
- Structured `.dec/` directory with schemas for:
  - Project identity (`project.isa.md`)
  - Configuration (`config/project.toml`)
  - State tracking (`state/progress.json`, `state/last_session.md`)
  - Decisions (`decisions/YYYY-*.md`)
  - Prompts (`prompts/system/*.md`)
  - Workflows (`.dec/workflows/*.yaml`)

### Project Type Templates
Pre-configured workflows and prompts based on project type:
- **API**: REST, GraphQL, gRPC workflows
- **CLI**: Argument parsing, help generation workflows
- **Microservice**: Service orchestration, deployment workflows

### Model Agnostic
- Works with **any** AI coding environment
- No API dependencies
- No telemetry
- All data stored locally
- AGENTS.md auto-generated for AI configuration

### Specialized Agents (Pipeline)
- 4 built-in agent roles: **researcher → coder → reviewer → documenter**
- Agents are **executable pipelines** — they run real commands (`action`), write files (`write`), and generate prompts (`prompt`)
- All agent artifacts are saved to `.dec/agent-output/{{task_id}}-*` for cross-session persistence
- Custom agents via `.dec/agents/*.yaml` (override builtins)
- Configurable timeout per agent (default 5 min)
- Trust system for action steps (same as workflows)
- `dectl agent trust <type> [--project]` — trust an agent without running it (for CI/headless)

### Executable Task Pipeline (Workflow)

The `execute_task` workflow (auto-created with `dectl project init --standard`) orchestrates the full SDD cycle in one command:

```bash
dectl workflow run execute_task --var task_id=T001 --var description="Add user auth"
```

**Pipeline steps:**

| Step | Type | Description |
|------|------|-------------|
| 1 | `agent: researcher` | Scans project, searches memory, reads decisions, saves context to `.dec/agent-output/` |
| 2 | `agent: coder` | Loads research context, checks git log, prepares implementation brief |
| 3 | `prompt` | **Pause** — the AI model implements the changes using its editing tools, then resumes with `--from-step 4` |
| 4 | `agent: reviewer` | Builds project, runs tests + linter, shows diff, generates review report |
| 5 | `agent: documenter` | Records task completion + decisions in memory, writes summary, lists artifacts (`run_always: true`) |

**Why this saves tokens and keeps the model focused:**

The pipeline delegates all mechanical work to dectl (scans, builds, git operations, memory writes) so the AI model only spends tokens on what requires intelligence — understanding context and writing code. Without it, the model would waste tokens on:
- Scanning file trees and searching memory (done by researcher)
- Remembering past decisions (read from files instead of prompt history)
- Compiling and verifying mentally (done by reviewer via real commands)
- Formatting and saving documentation (done by documenter)

Each agent ends with a **next-step hint** guiding the model to the next command:
```
→ Next in pipeline: dectl agent run coder --task T001 --var task_id=T001
→ After implementing, run: dectl agent run reviewer --task T001 --var task_id=T001
→ Task cycle complete. Verify: dectl memory search T001 --json
```

Use `--auto` to skip trust confirmation prompts (once agents are trusted):
```bash
dectl workflow run execute_task --var task_id=T001 --var description="test" --auto
```

### Automated Session Management
- `dectl session end` — single command to close a session
- Auto-generates session summary from git log and previous session
- Syncs git changes to progress.json (auto-marks features as done)
- Captures uncaptured decisions and saves to memory
- Reports agent activity during the session (Paso 5)
- Each step is independent; failures don't stop other steps
- `--dry-run` to preview, `--skip-git` to bypass git sync

### Spec-Driven Development
- `dectl spec init` — ensure `.dec/sdd/` exists with SDD methodology
- Auto-creates SKILL.md (atomic tasks, Build+Verify+Gate) + references/
- Updates `.dec/config/project.toml` and `.dec/isa/project.isa.md`
- Built into `project init --standard` (no extra command needed)
- Signals the AI model to interview you and generate `specs/` documents

## Installation

### Quick Install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/scripts/install.sh | bash
```

To pin a specific version or custom path:

```bash
curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/scripts/install.sh | bash -s -- --version v1.0.0 --to ~/.local/bin
```

### From Source

```bash
git clone https://github.com/jhonesis/dectl.git
cd dectl
cargo build --release
cp target/release/dectl "$(which dectl | xargs dirname)"/
```


## Quick Start

### 1. Initialize a Project

```bash
cd your-project
dectl project init --standard
# or with type-specific templates
dectl project init --standard --type api
```

This creates a `.dec/` directory with:
- `config/project.toml` — project metadata
- `isa/project.isa.md` — identity and vision
- `prompts/system/integration.md` — session instructions for the AI
- `decisions/` — for architectural decisions
- `workflows/` — for reusable workflows
- `AGENTS.md` — AI configuration file

### 2. Add Context to Memory

```bash
# Direct input
dectl memory add "We're using SQLite for local storage because it simplifies deployment"

# From file
cat architecture.md | dectl memory add --tags architecture,database

# With project filter
dectl memory add "API uses REST, not GraphQL" --project myapp

# View all memories (across projects)
dectl memory list --global
```

### 3. Use in AI Sessions

At the start of each session, the AI can load project context:

```bash
dectl project context --max-tokens 4000
```

This returns a compact summary including:
- Project identity and vision
- Last session state
- Recent decisions
- Progress tracking

### 4. Define Workflows

Create `.dec/workflows/test.yaml`:

```yaml
name: Run Tests
description: Run test suite with coverage
inputs:
  - name: coverage
    required: false
    default: "false"
steps:
  - type: prompt
    content: "Running tests..."
  - type: action
    cmd: "cargo test --{{coverage}}"
  - type: write
    path: ".dec/state/last_test.md"
    content: "Tests completed at {{timestamp}}"
```

Run it:

```bash
dectl workflow run test --var coverage=--cov
```

### 5. Use the Task Pipeline (Recommended)

The fastest way to execute a full SDD task cycle is with the `execute_task` workflow
(auto-created with `dectl project init --standard`):

```bash
# One command: research → code context → [you implement] → review → document → close
dectl workflow run execute_task --var task_id=T011 --var description="Implement Telegram notifications"

# After implementing the code at the prompt step, resume with:
dectl workflow run execute_task --var task_id=T011 --var description="Implement Telegram notifications" --from-step 4

# Skip trust prompts (when already trusted):
dectl workflow run execute_task --var task_id=T011 --var description="test" --auto
```

The documenter step always runs (`run_always: true`), so even if the build fails,
the attempted task is recorded in memory for traceability.

### 6. Or Run Individual Agents

```bash
dectl agent list                                        # List available agents
dectl agent describe coder                              # Describe an agent
dectl agent run coder --task "Add input validation"     # Run a single agent
dectl agent run --parallel reviewer,documenter --task "src/auth/"  # Parallel
dectl agent run coder --task "test" --dry-run           # Preview only
```

**Required project config** for the reviewer agent — add this to `.dec/config/project.toml`:
```toml
[build]
command = "cargo build"
```
The reviewer runs this command to validate your code compiles before reporting.

Agent artifacts are written to `.dec/agent-output/{{task_id}}-*` and persist between sessions.

## Commands

### Project Management

| Command | Description |
|---------|-------------|
| `dectl project init` | Initialize with minimal structure |
| `dectl project init --standard` | Level 2: + decisions, workflows, prompts |
| `dectl project init --full` | Level 3: + architecture, knowledge |
| `dectl project init --type api\|cli\|microservice` | With type-specific templates |
| `dectl project info` | Show project summary + schema warnings |
| `dectl project scan [--depth N]` | File tree (respects .gitignore) |
| `dectl project context [--max-tokens N] [--format text\|json]` | Compact context for stateless environments |

### Memory

| Command | Description |
|---------|-------------|
| `dectl memory add "<content>" [--tags t1,t2] [--project <name>]` | Add a memory |
| `dectl memory list [--project <name>] [--limit N] [--global]` | List memories |
| `dectl memory search "<query>" [--project <name>]` | Search memories |
| `dectl memory show <id>` | Show memory details |
| `dectl memory delete <id>` | Soft-delete (can be recovered) |
| `dectl memory delete <id> --hard` | Permanent deletion |
| `dectl memory edit <id>` | Edit in $EDITOR |

### Agents

| Command | Description |
|---------|-------------|
| `dectl agent list` | List available agents (built-in + custom) |
| `dectl agent describe <type>` | Show agent definition (role, steps, inputs) |
| `dectl agent trust <type> [--project <path>]` | Trust an agent for a project without running it |
| `dectl agent run researcher --task <desc> --var task_id=<id>` | Start full research→review→document pipeline |
| `dectl agent run <type> --task <desc>` | Execute a single agent for a task |
| `dectl agent run --parallel t1,t2 --task <desc>` | Run multiple agents in parallel |
| `[--file <path>] [--var k=v] [--timeout <secs>] [--dry-run]` | Optional flags for agent run |

Pipeline: `researcher` saves context to `.dec/agent-output/`, chains to `coder` (implement), then `reviewer` (compile via `[build] command`), then `documenter` (persist progress).

### Workflows

| Command | Description |
|---------|-------------|
| `dectl workflow list` | List all workflows |
| `dectl workflow describe <name>` | Show workflow details |
| `dectl workflow run <name> [--var k=v] [--dry-run] [--from-step N] [--auto]` | Execute workflow (`--auto` skips trust prompts) |
| Steps with `run_always: true` execute even if previous steps fail (e.g. documenter after failed build) |

### Session

| Command | Description |
|---------|-------------|
| `dectl session end` | End session: update last_session.md, sync git, capture decisions, report agents |
| `dectl session end --dry-run` | Preview changes without writing |
| `dectl session end --skip-git` | Skip git synchronization |

When executed, `dectl session end` performs five actions:
1. Updates `.dec/state/last_session.md` with a structured session summary
2. Syncs git changes to `.dec/state/progress.json` (marks features as done)
3. Captures uncaptured decisions and saves them to memory
4. Detects stack changes and syncs `project.toml`
5. Reports agent activity during the session from `agent_log`

### Spec-Driven Development

| Command | Description |
|---------|-------------|
| `dectl spec init` | Ensure `.dec/sdd/` exists with SKILL.md + references/, update bridge |
| `dectl spec init --json` | JSON output with envelope |

### Shell Completions

```bash
# Bash
dectl generate-completions bash > ~/.bash_completion
# Zsh
dectl generate-completions zsh > ~/.zsh/completions/_dectl
# Fish
dectl generate-completions fish > ~/.config/fish/completions/dectl.fish
```

### Protocol

```bash
# Execute commands from file (for automation)
dectl exec-from-file <path>
```

## Global Flags

All commands support:
- `--json` — JSON output with envelope `{status: "ok"|"error", ...}`
- `--non-interactive` — Abort instead of prompting
- `--help` — Command help
- `--version` — Version info

## How It Works

### The Three Actors

1. **`.dec/`** — Project context (Markdown + YAML + TOML + JSON)
2. **`dectl` binary** — Executor (Rust, SQLite, no runtime deps)
3. **Model/Environment** — Thinks, generates code, invokes `dectl`

They communicate through files and shell commands—no proprietary API.

### Session Flow

```
START
  1. Read .dec/config/project.toml + .dec/isa/project.isa.md
  2. Read .dec/state/last_session.md → resume from "Next step"
  3. Run: dectl project info --json → check warnings
  4. Read .dec/prompts/system/integration.md (if exists)
  5. Confirm understanding to developer

DURING SESSION
  • Use dectl memory add/search for context
  • Use dectl workflow run for repeatable tasks
  • Use dectl agent run for specialized roles (coder, reviewer, etc.)

CLOSE
  • Run: dectl session end (auto-summarizes, syncs git, captures decisions,
    syncs config, reports agent activity)
```

### Data Storage

| Data | Location |
|------|-----------|
| Global config | `~/.dectl/config.toml` |
| Memory (SQLite) | `~/.dectl/memory.db` |
| Trust registry | `~/.dectl/trust.toml` |
| Project context | `.dec/` (in project root) |

## Configuration

### Global Config (`~/.dectl/config.toml`)

```toml
[core]
default_editor = "vim"

[memory]
max_results = 50

[workflows]
default_timeout = 300
```

### Project Config (`.dec/config/project.toml`)

```toml
schema_version = "1.0"

[project]
name = "my-project"
type = "api|cli|microservice|other"
description = "A Rust CLI tool"

[stack]
languages = ["Rust"]
frameworks = []
databases = []
tools = []

[conventions]
rules = []

[build]
# Command to build the project. Used by reviewer agent for verification.
command = "cargo build --release"

[test]
# Command to run tests. Optional. Used by reviewer agent to validate changes.
command = "cargo test"

[lint]
# Command to run linter. Optional. Used by reviewer agent for code quality checks.
command = "cargo clippy"
```

The `[build]`, `[test]`, and `[lint]` sections are used by the reviewer agent to compile, test, and validate your project. Fill in the commands according to your project's stack:

#### Examples by Stack

**Rust (Cargo)**
```toml
[build]
command = "cargo build --release"

[test]
command = "cargo test"

[lint]
command = "cargo clippy"
```

**Node.js (npm)**
```toml
[build]
command = "npm run build"

[test]
command = "npm test"

[lint]
command = "npx eslint ."
```

**Node.js (pnpm)**
```toml
[build]
command = "pnpm build"

[test]
command = "pnpm test"

[lint]
command = "pnpm lint"
```

**Python (Poetry)**
```toml
[build]
command = "poetry build"

[test]
command = "poetry run pytest"

[lint]
command = "poetry run flake8"
```

**Python (pip)**
```toml
[build]
command = "python -m pip install -e ."

[test]
command = "pytest"

[lint]
command = "flake8"
```

**Go**
```toml
[build]
command = "go build -o bin/app ./cmd/main.go"

[test]
command = "go test ./..."

[lint]
command = "golangci-lint run"
```

**Java (Maven)**
```toml
[build]
command = "mvn clean package -DskipTests"

[test]
command = "mvn test"

[lint]
command = "mvn checkstyle:check"
```

**Java (Gradle)**
```toml
[build]
command = "gradle build -x test"

[test]
command = "gradle test"

[lint]
command = "gradle checkstyleMain"
```

If a command is not applicable to your project, leave it empty or omit the section entirely. The reviewer agent will skip it gracefully.

## Security

- Action steps in workflows and agents require explicit trust
- Trust is granted per-project via `~/.dectl/trust.toml`
- Three ways to trust:
  - **Interactive**: Run the agent — it prompts "Do you trust this agent? (y/N)"
  - **Explicit**: `dectl agent trust <type> --project .` — trust without executing
  - **Automatic**: `--auto` flag skips trust checks (for trusted pipelines)
- Paths are canonicalized (`canonicalize()`) to prevent duplicate entries from relative vs absolute paths
- Use `--non-interactive` to fail with a helpful suggestion instead of prompting in CI/CD

## Requirements

- Rust 1.70+
- Linux/macOS (Windows support via WSL)

## Development

```bash
cd dectl
cargo test        # Run all tests (115 passing)
cargo fmt         # Format code
cargo clippy      # Lint check
cargo build --release  # Build binary (~4.9MB)
```

## Contributing

See `specs/` for architecture and implementation details.

## License

MIT