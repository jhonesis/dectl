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

### 🧠 Persistent Memory
- Add important context: decisions, architecture notes, team conventions
- Search and retrieve across all stored memories
- Tag-based organization
- Soft-delete with `--hard` for permanent removal

### ⚡ Executable Workflows
- Define reusable workflows in YAML
- Variable interpolation: `{{variable}}`
- Step types: `prompt`, `action`, `write`
- Trust system for security (approves action steps per-project)

### 📋 Project Context
- Structured `.dec/` directory with schemas for:
  - Project identity (`project.isa.md`)
  - Configuration (`config/project.toml`)
  - State tracking (`state/progress.json`, `state/last_session.md`)
  - Decisions (`decisions/YYYY-*.md`)
  - Prompts (`prompts/system/*.md`)
  - Workflows (`.dec/workflows/*.yaml`)

### 🔄 Model Agnostic
- Works with **any** AI coding environment
- No API dependencies
- No telemetry
- All data stored locally

## Installation

### From Source

```bash
git clone https://github.com/yourusername/dectl.git
cd dectl
cargo build --release
sudo install target/release/dectl /usr/local/bin/
```

### From Release

Download the binary for your platform from the releases page and add it to your PATH.

## Quick Start

### 1. Initialize a Project

```bash
cd your-project
dectl project init --standard
```

This creates a `.dec/` directory with:
- `config/project.toml` — project metadata
- `isa/project.isa.md` — identity and vision
- `prompts/system/integration.md` — session instructions for the AI
- `decisions/` — for architectural decisions
- `workflows/` — for reusable workflows

### 2. Add Context to Memory

```bash
# Direct input
dectl memory add "We're using SQLite for local storage because it simplifies deployment"

# From file
cat architecture.md | dectl memory add --tags architecture,database

# With project filter
dectl memory add "API uses REST, not GraphQL" --project myapp
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

## Commands

### Project Management

```bash
dectl project init                    # Level 1: minimal structure
dectl project init --standard         # Level 2: + decisions, workflows, prompts
dectl project init --full             # Level 3: + architecture, knowledge
dectl project info                    # Show project summary + schema warnings
dectl project scan [--depth N]        # File tree (respects .gitignore)
dectl project context [--max-tokens N] [--format text|json]
                                       # Compact context for stateless environments
```

### Memory

```bash
dectl memory add "<content>" [--tags t1,t2] [--project <name>]
dectl memory list [--project <name>] [--limit N]
dectl memory search "<query>" [--project <name>]
dectl memory show <id>
dectl memory delete <id> [--hard]     # Soft-delete; --hard for permanent
dectl memory edit <id>                # Opens in $EDITOR
```

### Workflows

```bash
dectl workflow list                    # List all workflows
dectl workflow describe <name>       # Show workflow details
dectl workflow run <name> [--var k=v] [--dry-run] [--from-step N]
```

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
1. AI loads: .dec/config/project.toml + .dec/isa/project.isa.md
2. AI reads: .dec/state/last_session.md → resumes from "Next step"
3. AI runs: dectl project info --json → checks for warnings
4. AI reads: .dec/prompts/system/integration.md (if exists)
5. AI confirms: "I understand X, will work on Y"
```

### Data Storage

| Data | Location |
|------|-----------|
| Global config | `~/.dectl/config.toml` |
| Memory (SQLite) | `~/.dectl/memory.db` |
| Trust registry | `~/.dectl/trust.toml` |
| Project context | `.dec/` (in project root) |

## Why dectl?

| Feature | Claude Code | Gemini CLI | Ollama | dectl |
|---------|-------------|------------|--------|-------|
| Persistent memory | ❌ | ❌ | ❌ | ✅ |
| Local storage | ⚠️ | ❌ | ❌ | ✅ |
| Model-agnostic | ❌ | ❌ | ❌ | ✅ |
| Workflows | ⚠️ | ❌ | ❌ | ✅ |
| No telemetry | ⚠️ | ❌ | ❌ | ✅ |

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
type = "application"
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
- Linux/macOS/Windows (SQLite bundled)

## Contributing

See `specs/` for architecture and implementation details.

## License

MIT