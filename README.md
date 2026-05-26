# dectl — Dev Environment Control

> A model-agnostic developer life OS that gives any AI coding environment persistent memory, executable workflows, and structured project context.

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
- Step types: `prompt`, `action`, `write`
- Trust system for security (approves action steps per-project)

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

### Specialized Agents
- 4 built-in agent roles: coder, reviewer, researcher, documenter
- Execute agents individually or in parallel
- Agents are prompt templates with reusable steps (prompt, action, write)
- All executions are logged to memory.db for auditing
- Custom agents via `.dec/agents/*.yaml`
- Configurable timeout per agent (default 5 min)
- Trust system for action steps (same as workflows)

### Automated Session Management
- `dectl session end` — single command to close a session
- Auto-generates session summary from git log and previous session
- Syncs git changes to progress.json (auto-marks features as done)
- Captures uncaptured decisions and saves to memory
- Reports agent activity during the session (Paso 5)
- Each step is independent; failures don't stop other steps
- `--dry-run` to preview, `--skip-git` to bypass git sync

## Installation

### From Source

```bash
git clone https://github.com/jhonesis/dectl.git
cd dectl
cargo build --release
sudo install target/release/dectl /usr/local/bin/
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

### 5. Use Agents

```bash
# List available agents
dectl agent list

# Describe an agent (see its role, steps, inputs)
dectl agent describe coder

# Run an agent for a specific task
dectl agent run coder --task "Add input validation to the API"

# Run multiple agents in parallel
dectl agent run --parallel reviewer,documenter --task "src/auth/"

# Preview without side effects
dectl agent run coder --task "test" --dry-run
```

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
| `dectl agent run <type> --task <desc>` | Execute an agent for a task |
| `dectl agent run --parallel t1,t2 --task <desc>` | Run multiple agents in parallel |
| `[--file <path>] [--var k=v] [--timeout <secs>] [--dry-run]` | Optional flags for agent run |

### Workflows

| Command | Description |
|---------|-------------|
| `dectl workflow list` | List all workflows |
| `dectl workflow describe <name>` | Show workflow details |
| `dectl workflow run <name> [--var k=v] [--dry-run] [--from-step N]` | Execute workflow |

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

## dectl Agents vs IDE AI Agents

Tools like **Claude Code agents** and **opencode agents** are extensions of their conversational model — the AI uses them internally to delegate subtasks, read files, or run commands. They are tightly coupled to a specific provider and IDE.

**dectl agents** solve a different problem:

| Aspect | IDE Agents (Claude Code, opencode) | dectl Agents |
|--------|-----------------------------------|--------------|
| **What they are** | Tool-calling functions the model uses internally | Prompt templates + step recipes in YAML |
| **Model dependency** | Tied to a specific AI provider | Model-agnostic — any model or human can follow them |
| **Portability** | Stay inside the IDE/chat | Portable across any environment |
| **Audit trail** | Usually none | Every execution logged to SQLite with timestamp |
| **Customization** | Requires provider-specific config | Just drop a `.yaml` file in `.dec/agents/` |
| **Composability** | Single-model internal calls | Can be chained in workflows (`type: agent`) |
| **Persistence** | Session-only | Full memory integration via `dectl memory` |

**They complement each other.** An IDE agent can invoke `dectl agent run coder --task "..."` as part of its workflow, benefiting from standardized prompts and auditable execution. dectl is the "what to do" (the recipe); IDE agents are the "who does it" (the executor with IDE access).

## Why dectl?

| Feature | Claude Code | Gemini CLI | Ollama | dectl |
|---------|-------------|------------|--------|-------|
| Persistent memory | ❌ | ❌ | ❌ | ✅ |
| Local storage | ⚠️ | ❌ | ❌ | ✅ |
| Model-agnostic | ❌ | ❌ | ❌ | ✅ |
| Workflows | ⚠️ | ❌ | ❌ | ✅ |
| Specialized agents | ⚠️ | ❌ | ❌ | ✅ |
| No telemetry | ⚠️ | ❌ | ❌ | ✅ |
| Project templates | ❌ | ❌ | ❌ | ✅ |
| Auto-fill on init | ❌ | ❌ | ❌ | ✅ |
| Session management | ❌ | ❌ | ❌ | ✅ |

dectl complements any AI coding tool by providing the memory they lack.

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
languages = ["Rust", "TypeScript"]

[settings]
auto_init = true
```

## Security

- Action steps in workflows require explicit trust
- Trust is granted per-project via `~/.dectl/trust.toml`
- First run with action steps prompts for confirmation
- Use `--non-interactive` to skip prompts in CI/CD

## Requirements

- Rust 1.70+
- Linux/macOS (Windows support via WSL)

## Development

```bash
cd dectl
cargo test        # Run all tests (83 passing)
cargo fmt         # Format code
cargo clippy      # Lint check
cargo build --release  # Build binary (~4.5MB)
```

## Contributing

See `specs/` for architecture and implementation details.

## License

MIT