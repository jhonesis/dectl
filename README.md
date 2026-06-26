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

### Persistent Memory with Structured Types
- Add important context with **typed entries**: `--type decision`, `--type research`, `--type context`, `--type incident`
- **FTS5-powered search** with ranking — the most relevant results appear first
- **Query language** (`dectl memory query`) — filter by type, tags, project, date with boolean operators (`AND`/`OR`/`NOT`)
- **Agent auto-link** — agent results are automatically saved to memory (researcher → `type=research`, documenter → `type=note`)
- **Decision capture** — `dectl session end` auto-captures decisions as `type=decision`
- **Restore** — `dectl memory restore <id>`: undo soft-delete, recovered like a trash bin
- **Export/Import** — `dectl memory export <path> [--format json|jsonl]` and `dectl memory import <path>` with automatic dedup
- **Interactive selection** — `dectl memory show` and `dectl memory delete` open `fzf` (or numbered fallback) when run without an ID
- Tag-based organization
- Per-project memory with auto-detection
- Soft-delete with `--hard` for permanent removal

### Project Health & Diagnostics
- `dectl doctor` — 6 automated health checks: database integrity, schema version, `.dec/` structure, trust file, config files, and git availability
- `--fix` flag attempts to repair detected issues automatically (creates missing directories, runs migrations)
- `--json` output for CI/CD integration
- Catch problems early: misconfigured projects, missing directories, schema drift

### Executable Workflows
- Define reusable workflows in YAML
- **Handlebars template engine** — full `{{variable}}` interpolation plus `{{#if}}` conditionals, `{{#each}}` loops, and helpers
- **Conditional step execution** — `skip_if: "{{expression}}"` skips steps when the expression evaluates to `true`, `1`, or `yes`
- **Configurable timeout** — `timeout_secs: N` per step prevents hung commands from blocking the workflow
- **Captured stderr** — failed steps display their stderr output in both human and JSON output, so you know exactly why a command failed
- `run_always: true` — steps that execute even if previous steps fail (e.g. documenter after failed build)
- `--from-step N` to resume workflows after fixing failures
- `--auto` flag to skip trust confirmation prompts
- Step types: `prompt`, `action`, `write`, `agent`
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

### Specialized Agents (Pipeline)
- 4 built-in agent roles: **researcher → coder → reviewer → documenter**
- Agents are **executable pipelines** — they run real commands (`action`), write files (`write`), and generate prompts (`prompt`)
- All agent artifacts are saved to `.dec/agent-output/{{task_id}}-*` for cross-session persistence
- The **reviewer** agent includes spec compliance verification: looks up `task_id` in `specs/*/tasks.md`, extracts the associated `REQ-XXX` identifier, reads acceptance criteria from `specs/*/spec.md`, and includes them in the review prompt so the AI model can verify the implementation matches the spec
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
| 4 | `agent: reviewer` | Looks up task in SDD specs (task_id → REQ-XXX → acceptance criteria), builds project, runs tests + linter, shows diff, verifies spec compliance, generates review report |
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

### Memory Query Language
- `dectl memory query "<expression>"` — field-based query syntax without SQL
- **Fields**: `type:`, `tags:`, `project:`, `created:` — filter by any memory attribute
- **Comparators**: `=` (default), `>`, `<`, `>=`, `<=`, `!=` — use with `created:` for date ranges
- **Boolean operators**: `AND`, `OR`, `NOT` — combine multiple conditions
- **Sorting & limit**: `ORDER BY created DESC`, `LIMIT N`
- **Examples**:
  ```
  type:decision AND tags:architecture
  type:research ORDER BY created DESC LIMIT 5
  created:>2026-06-01 AND NOT type:note
  ```
- **Zero SQL injection risk** — all values are parameterized
- **Why it matters**: Instead of scrolling through `dectl memory list`, ask precisely for what you need. The AI model can query its own memory with surgical precision, saving tokens and time.

### Automated Session Management
- `dectl session end` — single command to close a session
- Auto-generates session summary from git log and previous session
- Syncs git changes to progress.json (auto-marks features as done)
- Captures uncaptured decisions and saves to memory
- Reports agent activity during the session (Paso 5)
- Each step is independent; failures don't stop other steps
- `--dry-run` to preview, `--skip-git` to bypass git sync

### Session Hooks
- **Configurable post-session automation** via `[session]` section in `.dec/config/project.toml`:
  ```toml
  [session]
  hooks = ["post_session_cleanup"]
  ```
- Hooks are **workflow names** from `.dec/workflows/` that execute after all 5 standard session end steps
- **Fault-tolerant**: if a hook fails, other hooks and main steps continue unaffected
- Built-in example: `post_session_cleanup` — cleans up temporary agent artifacts after session end
- Extend `dectl session end` without modifying code — just add a workflow and reference it

### Spec-Driven Development (SDD)
- `dectl spec init` — ensure `.dec/sdd/` exists with SDD methodology (built into `project init --standard`)
- Auto-creates 3 embedded templates:
  - **SKILL.md** — SDD workflow: 8 interview questions, clarification phase, adversarial agent pattern (Coordinator/Implementer/Verifier), model tiering, WHAT vs HOW enforcement, memory integration
  - **templates.md** — 9 document templates per project type (spec, architecture, tasks, API, data, auth, deployment, testing, monitoring), edge case catalog, purity boundaries, drift detection
  - **examples.md** — 5 reference implementations (CLI logsnap, API SnippetVault, brownfield LegacyPay, EDA EventStream, Next.js 14 HabitStack)
- Updates `.dec/config/project.toml` and `.dec/isa/project.isa.md`
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
# Direct input (defaults to type=note)
dectl memory add "We're using SQLite for local storage because it simplifies deployment"

# From file with type and tags
cat architecture.md | dectl memory add --type context --tags architecture,database

# Mark an architectural decision
dectl memory add "API uses REST, not GraphQL" --type decision --project myapp

# View all memories with their type
dectl memory list --global

# Search with FTS5 (ranking by relevance)
dectl memory search "SQLite storage"

# Query with field filters and boolean operators
dectl memory query "type:decision AND tags:architecture"
dectl memory query "type:research ORDER BY created DESC LIMIT 5"
dectl memory query "created:>2026-01-01 AND NOT type:note"
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

**Configure build/test/lint** for the reviewer agent in `.dec/config/project.toml`:
```toml
[build]
command = "cargo build"

[test]
command = "cargo test"

[lint]
command = "cargo clippy"
```
The reviewer auto-detects commands for common stacks (Cargo.toml, package.json, etc.) when not explicitly configured, and skips any missing step gracefully.

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
| `dectl project scan [--depth N]` | File tree with progress spinner (respects .gitignore) |
| `dectl project context [--max-tokens N] [--format text\|json\|compact]` | Compact context for stateless environments (with progress spinner) |
| `dectl project watch [--interval N] [--json]` | Watch mode — polls and diffs file tree every N seconds |
| `dectl doctor [--fix] [--json]` | Health check: 6 diagnostics with optional auto-repair |

### Memory

| Command | Description |
|---------|-------------|
| `dectl memory add "<content>" [--tags t1,t2] [--project <name>] [--type <type>]` | Add a memory (types: note, decision, context, research, incident, code-snippet) |
| `dectl memory list [--project <name>] [--limit N] [--global] [--include-deleted]` | List memories (use `--include-deleted` to see soft-deleted entries) |
| `dectl memory search "<query>" [--project <name>]` | Search memories (FTS5 with ranking) |
| `dectl memory query "<query>" [--project <name>] [--limit N]` | **Filter memories** with field query language (type:, tags:, project:, created: + AND/OR/NOT + ORDER BY + LIMIT) |
| `dectl memory show [<id>]` | Show memory details; without ID opens `fzf` interactive selector (or numbered fallback) |
| `dectl memory delete [<id>]` | Soft-delete; without ID opens `fzf` interactive selector (or numbered fallback) |
| `dectl memory delete <id> --hard` | Permanent deletion |
| `dectl memory restore <id>` | Restore a soft-deleted memory entry |
| `dectl memory export <path> [--format json\|jsonl]` | Export all memories to JSON or JSONL file |
| `dectl memory import <path>` | Import memories from JSON or JSONL (with dedup) |
| `dectl memory edit <id>` | Edit in `$EDITOR` (falls back to vi → nano → vim) |

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

Pipeline: `researcher` saves context to `.dec/agent-output/`, chains to `coder` (implement), then `reviewer` (verify spec compliance + compile via `[build] command` + test + lint), then `documenter` (persist progress).

### Workflows

| Command | Description |
|---------|-------------|
| `dectl workflow list` | List all workflows |
| `dectl workflow describe <name>` | Show workflow details |
| `dectl workflow run <name> [--var k=v] [--dry-run] [--from-step N] [--auto]` | Execute workflow (`--auto` skips trust prompts) |
| Steps with `run_always: true` execute even if previous steps fail (e.g. documenter after failed build) |
| Workflow steps support Handlebars interpolation, `skip_if` conditionals, `timeout_secs`, and capture stderr on failure |

### Session

| Command | Description |
|---------|-------------|
| `dectl session end` | End session: update last_session.md, sync git, capture decisions, report agents |
| `dectl session end --dry-run` | Preview changes without writing |
| `dectl session end --skip-git` | Skip git synchronization |

When executed, `dectl session end` performs five actions (plus optional hooks):
1. Updates `.dec/state/last_session.md` with a structured session summary
2. Syncs git changes to `.dec/state/progress.json` (marks features as done)
3. Captures uncaptured decisions and saves them to memory
4. Detects stack changes and syncs `project.toml`
5. Reports agent activity during the session from `agent_log`
6. **[Optional hooks]** Executes configured workflows from `[session] hooks` in `project.toml` (e.g. `post_session_cleanup`)

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
- `--json` — JSON output with envelope `{status: "ok"|"error", "hint": "..."}` — every error includes a `hint` field suggesting the corrective action
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

### Architecture v2 — Foundation (Phase 1) ✅

The first architectural improvement phase introduces four infrastructure upgrades that improve performance, debuggability, and extensibility:

| Change | What it does | Benefit |
|--------|-------------|---------|
| **Singleton DB connection** | Single `OnceLock<Mutex<Connection>>` shared by all commands | Eliminates 15 independent SQLite connections; reduces latency and file contention under parallel agent execution |
| **Structured logging** | `RUST_LOG=debug` support via `log` + `env_logger` | Four log levels (error/warn/info/debug); control verbosity without code changes — `RUST_LOG=debug dectl workflow run ...` |
| **Output trait pattern** | `OutputFormat` trait with `HumanFormat`/`JsonFormat` strategies | New output formats require zero changes to command code; single envelope format across all commands |
| **Thread pool with timeout** | Bounded pool (size = `available_parallelism`) with `with_timeout()` | Prevents unlimited thread spawning; deterministic timeout kills hung agents and logs the failure |

### Architecture v2 — Code Quality (Phase 2) ✅

Phase 2 eliminates inconsistency layers that accumulated across 14 modules and makes every error actionable:

| Change | What it does | Benefit |
|--------|-------------|---------|
| **Unified TOML merge** | Single `merge_array` + `ensure_section` replacing 3 ad-hoc implementations | Session end and spec init never produce duplicate entries in `project.toml`, regardless of how many times they run |
| **Git abstraction layer** | `is_git_repo`, `recent_commits`, `diff_since` typed API replacing raw `Command::new("git")` across 3 session modules | Session end detects changes consistently — if git is missing it fails early with a clear message instead of silently producing empty diffs |
| **Storage trait (DI)** | `Storage` trait with `RealStorage` (SQLite) and `InMemoryStorage` (HashMap) implementations | Tests run 3x faster for memory logic; production behavior is identical |
| **Actionable error hints** | Every user-facing error includes a next-step hint | In human mode: `Error: Workflow 'foo' not found (Run \`dectl workflow list\` to see available workflows)`. In `--json`: the `hint` field appears in the error envelope — the AI model receives the corrective action directly |

### Architecture v2 — Agent & Workflow (Phase 3) ✅

Phase 3 upgrades workflow orchestration with a proper template engine, step-level diagnostics, conditional execution, and configurable timeouts:

| Change | What it does | Benefit |
|--------|-------------|---------|
| **Handlebars template engine** | Full `{{#if}}`, `{{#each}}`, and helpers replacing simple regex interpolation | Workflows can adapt their behavior based on variables — conditional prompts, dynamic file lists, nested template blocks |
| **Step error with stderr** | Every step captures stdout + stderr separately; failed steps aggregate into `ExecutionResult.logs` | Debug failures in seconds — the actual command error is right there in the output, not buried in terminal scrollback |
| **Conditional steps** | `skip_if: "{{expression}}"` skips execution when truthy (`true`/`1`/`yes`) | One workflow handles multiple scenarios: `skip_if: "{{#if no_code}}true{{/if}}"` skips the coder step when no code changes are needed |
| **Per-step timeout** | `timeout_secs: N` per step via bounded thread pool with `with_timeout()` | Hung commands (infinite loops, network waits) are killed automatically — no more blocked workflows |

### Architecture v2 — New Features (Phase 4) ✅

Phase 4 adds five missing features that fill critical gaps in the developer workflow — diagnostics, automation, data safety, portability, and live monitoring:

| Change | What it does | Benefit |
|--------|-------------|---------|
| **Doctor command** | `dectl doctor` runs 6 health checks (DB, schema, `.dec/`, trust, config, git) with `--fix` to auto-repair | One command to diagnose and repair project health — no more wondering if your setup is broken |
| **Session hooks** | Configurable post-session workflows via `[session] hooks` in `project.toml` | Extend `dectl session end` without modifying code — add cleanup, validation, or notifications |
| **Memory restore** | `dectl memory restore <id>` undoes soft-delete; `--include-deleted` lists recoverable entries | Zero fear when deleting — memories go to a recoverable trash bin, not to /dev/null |
| **Memory export/import** | `dectl memory export <path> [--format json|jsonl]` and `dectl memory import <path>` with automatic dedup (by content + timestamp) | Backup your knowledge base, migrate between projects, share context with teammates |
| **Project watch** | `dectl project watch [--interval N]` polls and diffs the file tree every N seconds | See file changes in real time without re-running `scan` — ideal for long sessions |

### Architecture v2 — UX/DX Polish (Phase 5) ✅

Phase 5 transforms dectl from purely functional to delightful — visual feedback, interactive selection, editor resilience, consistent styling, and immediate input validation:

| Change | What it does | Benefit |
|--------|-------------|---------|
| **Progress bars** | `indicatif` spinners in `project scan` and `project context` (Human mode only; suppressed in `--json`) | You know the command is running — no second-guessing or premature Ctrl+C on large projects |
| **Fuzzy finder (fzf)** | `dectl memory show` / `dectl memory delete` without an ID opens `fzf` for interactive selection; falls back to numbered list if `fzf` is not installed | Eliminates the "copy-paste-ID" friction — select from a fuzzy-filtered list in one step |
| **Editor fallback chain** | `memory edit` tries `$EDITOR` → `$VISUAL` → `vi` → `nano` → `vim`; errors with a clear hint if none exist | Works out of the box on any fresh system, Docker container, or CI runner |
| **Consistent color palette** | Centralized `core/output.rs::palette` with `SUCCESS` (green), `ERROR` (red), `WARNING` (yellow), `DIM` (bright black) across all commands | Your eyes learn the color grammar — red means error, green means done, no matter which command you run |
| **Shell completion for --type** | `value_parser` with the 6 valid types (`note`, `decision`, `context`, `research`, `incident`, `code-snippet`) — autocompletes in bash/zsh/fish | Invalid types are rejected by clap before touching the database; tab-completion discoverable at the shell |

### Data Storage

| Data | Location |
|------|-----------|
| Global config | `~/.dectl/config.toml` |
| Memory (SQLite) | `~/.dectl/memory.db` (single shared connection) |
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
cargo test        # Run all tests (148 passing)
cargo fmt         # Format code
cargo clippy      # Lint clean (0 warnings)
cargo build --release  # Build binary (~5.7MB)
```

## Contributing

See `specs/` for architecture and implementation details.

## License

MIT