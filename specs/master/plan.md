# Technical Implementation Plan — dectl
> *Technology-specific. Describes HOW to build what spec.md defines.*
> *Version: 1.0 | Status: Updated | Last updated: 2026-06-02*

---

## References

- Implements: `specs/master/spec.md`
- Constitution: `specs/master/constitution.md`
- Research: `specs/master/research.md`

---

## Tech Stack

| Layer | Technology | Version | Justification |
|-------|-----------|---------|---------------|
| CLI framework | `clap` (derive API) | 4.x | Best `--help` output, shell completions, industry standard |
| Memory backend | `rusqlite` (bundled) | 0.31+ | Static SQLite, no runtime deps, transactions |
| Workflow format | YAML via `serde_yaml` | 0.9+ | Serde integration, model-writable format |
| Config format | TOML via `toml` | 0.8+ | Rust-native, human-readable |
| Project scan | `ignore` crate | 0.4+ | Correct gitignore parsing (ripgrep-grade) |
| JSON output | `serde_json` | 1.x | `--json` flag on all commands (REQ-006) |
| Error handling | `anyhow` | 1.x | Mandated by constitution |
| Timestamps | `chrono` | 0.4+ | Memory entry timestamps |
| Language | Rust | stable | Single binary, performance, portability |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                  AI Coding Environment               │
│         (Claude Code, Gemini CLI, Qwen CLI…)        │
│                                                     │
│  reads .dec/ directly   │   executes dectl commands   │
└────────────┬────────────┴───────────┬───────────────┘
             │                        │
             ▼                        ▼
┌────────────────────┐    ┌──────────────────────────┐
│     .dec/          │    │      dectl (CLI binary)     │
│                    │    │                           │
│  config/           │◄───│  core      (parser, log)  │
│  isa/              │    │  project   (scan, files)  │
│  decisions/        │    │  memory    (SQLite)       │
│  workflows/        │    │  workflow  (YAML runner)  │
│  prompts/          │    │  session   (end, sync)    │
│  knowledge/        │    │  agent     (roles, run)   │
│  state/            │    │  spec      (sdd init)     │
│                    │    │  protocol  (exec-from)    │
│                    │    │                           │
│                    │    └──────────────┬────────────┘
└────────────────────┘                   │
                                         ▼
                              ┌──────────────────────┐
                              │   ~/.dectl/            │
                              │   config.toml        │
                              │   memory.db          │
                              │   trust.toml         │
                              └──────────────────────┘
```

**Three-actor contract:**
- `.dec/` — read by the model directly as Markdown/YAML; written by the CLI
- `dectl` binary — executes actions, manages memory, runs workflows; never contains a model
- Model layer — reads `.dec/`, invokes `dectl` commands; fully interchangeable

---

## Project Structure (Rust)

```
dectl/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── CLAUDE.md                    ← agent context file
├── specs/
│   └── master/                  ← this SDD suite
├── src/
│   ├── main.rs                  ← entry point, command registration
│   ├── core/
│   │   ├── mod.rs
│   │   ├── config.rs            ← global + project config loading
│   │   ├── error.rs             ← error types (removed in Phase 10, uses anyhow)
│   │   └── output.rs            ← text/json output formatting
│   ├── project/
│   │   ├── mod.rs
│   │   ├── init.rs              ← dectl project init (with auto-fill)
│   │   ├── info.rs              ← dectl project info
│   │   ├── scan.rs              ← dectl project scan (uses ignore crate)
│   │   ├── context.rs           ← dectl project context
│   │   ├── templates.rs         ← .dec/ folder templates
│   │   └── auto_fill.rs         ← stack detection + auto-fill
│   ├── memory/
│   │   ├── mod.rs
│   │   ├── db.rs                ← SQLite connection + migrations
│   │   ├── add.rs               ← dectl memory add
│   │   ├── list.rs              ← dectl memory list
│   │   ├── search.rs            ← dectl memory search
│   │   ├── show.rs              ← dectl memory show <id>
│   │   ├── delete.rs            ← dectl memory delete (soft/hard)
│   │   └── edit.rs              ← dectl memory edit ($EDITOR)
│   ├── workflow/
│   │   ├── mod.rs
│   │   ├── schema.rs            ← Workflow, Step, StepType structs
│   │   ├── loader.rs            ← YAML parsing + validation
│   │   ├── runner.rs            ← step execution logic
│   │   ├── trust.rs             ← trust registry (trust.toml)
│   │   ├── list.rs              ← dectl workflow list
│   │   ├── run.rs               ← dectl workflow run <name>
│   │   └── describe.rs          ← dectl workflow describe <name>
│   ├── session/
│   │   ├── mod.rs
│   │   ├── types.rs             ← SessionSummary, GitChanges, CapturedDecision, SessionEndResult
│   │   ├── session_summary.rs   ← generate + write last_session.md
│   │   ├── git_sync.rs          ← detect git changes + sync progress.json
│   │   ├── decision_capture.rs  ← regex-based decision extraction
│   │   ├── config_sync.rs       ← detect stack changes, merge into project.toml
│   │   └── end.rs               ← orchestrator for all 5 steps
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── schema.rs            ← AgentDef, AgentSource, AgentResult
│   │   ├── loader.rs            ← load built-in + custom agents
│   │   ├── runner.rs            ← execute agent steps with timeout
│   │   ├── parallel.rs          ← run multiple agents in threads
│   │   ├── log.rs               ← agent_log table migration + INSERT
│   │   ├── list.rs              ← dectl agent list
│   │   ├── describe.rs          ← dectl agent describe
│   │   ├── run.rs               ← dectl agent run (single + --parallel)
│   │   └── builtins/            ← 4 built-in templates (YAML)
│   ├── spec/
│   │   ├── mod.rs
│   │   ├── init.rs              ← dectl spec init orchestrator
│   │   ├── templates.rs         ← embedded SKILL.md + references/
│   │   └── bridge.rs            ← update project.toml + isa.md
│   └── protocol/
│       ├── mod.rs
│       └── exec.rs              ← dectl exec-from-file <path>
└── tests/
    ├── project_tests.rs
    ├── memory_tests.rs
    ├── workflow_tests.rs
    ├── session_tests.rs
    ├── spec_init.rs             ← spec init integration tests (6)
    └── e2e_anchor.rs            ← anchor moment end-to-end test
```

---

## Data Flow

### Memory add
```
developer/model → dectl memory add "text" [--tags t1,t2]
  → core: parse args, load config
  → memory/db: open ~/.dectl/memory.db, run migration if needed
  → memory/add: INSERT INTO memories (content, tags, created_at)
  → core/output: print confirmation (id, preview) or --json
```

### Workflow run
```
developer/model → dectl workflow run <name>
  → workflow/loader: read .dec/workflows/<name>.yaml, parse into Workflow struct
  → workflow/trust: check trust.toml for this workflow
    → if not trusted + has action steps: prompt user once → write to trust.toml
  → workflow/runner: iterate steps in order
    → prompt step: print text, pause for model/user to act
    → action step: std::process::Command (or sh -c if shell:true), capture output
    → write step: write content to target path
  → on failure: report step index, command, stderr → exit non-zero
```

### Project init
```
developer → dectl project init
  → project/init: check .dec/ does not exist
  → project/templates: create folder structure + base files from templates
  → project/auto_fill: detect stack, scan docs, fill files (if non-empty project)
  → core/output: print summary of created files + next steps
```

### Session end
```
developer/model → dectl session end [--dry-run] [--skip-git]
  → session/end: orchestrate 5 independent steps
    Step 1: session/session_summary
      → read .dec/state/last_session.md (previous context)
      → git log --oneline -20 (recent commits)
      → git diff --name-only (modified files)
      → generate SessionSummary struct
      → write .dec/state/last_session.md (or preview if --dry-run)
    Step 2: session/git_sync (skip if --skip-git or no git repo)
      → git diff --name-status (modified files with status)
      → git log --oneline -20 (recent commits)
      → read .dec/state/progress.json
      → mark features as "done" if related files modified
      → detect new features from commit messages (feat:, feature:, add:)
      → write updated progress.json
    Step 3: session/decision_capture
      → read .dec/state/last_session.md + git log
      → regex patterns for decision language (ES/EN)
      → compare with existing memories (avoid duplicates)
      → INSERT new decisions into ~/.dectl/memory.db
    Step 4: session/config_sync
      → detect_stack() re-run on filesystem
      → compare with stack in .dec/config/project.toml
      → merge new items (languages, frameworks, tools) without removing
      → check project.isa.md coherence → generate warnings
    Step 5: session/agent_sync
      → query agent_log table since last session end
      → count agent sessions, steps executed
      → report to developer
  → session/end: collect results, print summary per step
  → core/output: print result (human or --json)
```

---

## .dec/ Folder Structure (created by `dectl project init`)

```
.dec/
├── config/
│   └── project.toml         ← project name, type, stack, conventions
├── isa/
│   └── project.isa.md       ← vision, objectives, scope, risks
├── decisions/               ← ADR files (0001-*.md, 0002-*.md…)
├── workflows/               ← YAML workflow definitions
├── prompts/
│   ├── system/
│   │   ├── base.md          ← base system prompt for the model
│   │   └── integration.md   ← session instructions for AI
│   └── tasks/               ← task-specific prompts
├── knowledge/
│   ├── glossary.md
│   └── constraints.md
└── state/
    ├── progress.json        ← feature status tracking
    └── last_session.md      ← session summary (auto-generated)
```

---

## Workflow YAML Schema

```yaml
name: design_architecture
description: Guide the model through designing the system architecture
steps:
  - type: prompt
    content: |
      Read .dec/isa/project.isa.md and .dec/config/project.toml.
      Identify the main components and their responsibilities.

  - type: action
    cmd: ["git", "log", "--oneline", "-10"]
    description: Show recent commits for context

  - type: action
    cmd: ["dectl", "memory", "search", "architecture"]
    description: Retrieve relevant past decisions

  - type: write
    path: .dec/decisions/0001-architecture.md
    content: |
      # Decision: Initial Architecture
      <!-- model fills this in -->

  - type: prompt
    content: |
      Fill in .dec/decisions/0001-architecture.md with the architecture decision.
      Then run: dectl memory add "Architecture decision recorded: [summary]"
```

---

## Global Config Schema (`~/.dectl/config.toml`)

```toml
[core]
default_editor = "vim"      # used when dectl opens files
color = true                # colored output

[memory]
db_path = "~/.dectl/memory.db"
max_results = 20            # default for dectl memory list

[workflow]
trust_path = "~/.dectl/trust.toml"
```

---

## Implementation Phases

### Phase 1 — MVP (target: working CLI with core value)
**Goal**: A developer can init a project, store memory, and inspect workflows. An AI environment can read `.dec/` and invoke basic commands.

**Deliverables**:
- `dectl project init` — creates `.dec/` structure
- `dectl project info` — displays project context summary
- `dectl project scan` — file tree respecting `.gitignore`
- `dectl memory add` — stores entry in SQLite
- `dectl memory list` — lists entries reverse chronological
- `dectl memory search` — keyword search
- `dectl memory show <id>` — full entry display
- `--json` flag on all Phase 1 commands
- Global config loading (`~/.dectl/config.toml`)
- Project config loading (`.dec/config/project.toml`)

**Requirements covered**: REQ-001, REQ-002, REQ-003, REQ-006, REQ-007

---

### Phase 2 — Workflows (target: full automation loop)
**Goal**: Developers and models can define and execute multi-step workflows. Trust system prevents accidental execution.

**Deliverables**:
- `dectl workflow list`
- `dectl workflow run <name>` with trust confirmation
- `dectl workflow describe <name>`
- `dectl exec-from-file <path>` — protocol module
- Semantic memory search (embeddings via local model, optional)

**Requirements covered**: REQ-004, REQ-005

---

### Phase 3 — Polish + Ecosystem (target: community-ready)
**Goal**: dectl is installable, documented, and extensible by the community.

**Deliverables**:
- Shell completions (bash, zsh, fish) via `clap`
- `dectl update` — self-update mechanism
- Project templates by type (API, CLI, microservice)
- Team-friendly `.dec/` — personal memory kept in `~/.dectl/`, shared context in `.dec/`
- Plugin system for custom step types in workflows
- Windows support investigation

---

### Phase 3b — Auto-fill + Project Context (target: smarter init)
**Goal**: `dectl project init` auto-detects project stack and fills context automatically.

**Deliverables**:
- `is_project_empty()` — detect if project has existing code
- `detect_stack()` — detect language from config files (package.json, Cargo.toml, go.mod, etc.)
- `scan_docs_for_context()` — extract project name from README title (basic fallback)
- `fill_project_files()` — update `.dec/` with detected context
- Interactive prompts for empty projects (name, type, languages, description, vision)
- `dectl project context [--max-tokens N] [--format text|json]` — compact summary for stateless AI environments
- `AGENTS.md` auto-generated at project root

**Requirements covered**: REQ-001 (extended), REQ-002 (extended)

---

### Phase 4 — Session Management (target: automated session closure)
**Goal**: A single command captures all session context so the next session starts with full knowledge.

**Deliverables**:
- `dectl session end` — orchestrates five independent steps:
  1. Generate and write `.dec/state/last_session.md` from git log and previous session
  2. Sync git changes to `.dec/state/progress.json` (mark features done, detect new ones)
  3. Capture uncaptured decisions via regex patterns and save to memory
  4. Config sync: detect stack changes, merge into project.toml, check isa coherence
  5. Agent sync: query agent_log since last session, report activity
- `--dry-run` flag to preview changes without writing
- `--skip-git` flag to bypass git synchronization
- `--json` flag for machine-readable output
- Each step is independent; failures don't stop other steps
- Session module structure: `src/session/` with `types.rs`, `session_summary.rs`, `git_sync.rs`, `decision_capture.rs`, `config_sync.rs`, `end.rs`

**Requirements covered**: REQ-008

### Phase 5 — Config Sync (target: automatic stack coherence)
**Goal**: `dectl session end` detects stack changes during development and keeps `.dec/config/project.toml` in sync.

**Deliverables**:
- `dectl session end` Paso 4: config_sync
- `detect_stack()` re-run compares filesystem with registered stack
- Merge new languages/frameworks/tools into `project.toml` (never overwrite)
- Check `project.isa.md` coherence and generate warnings
- `--dry-run` support for config_sync

**Requirements covered**: REQ-C-017

### Phase 7 — Polish & Adopción (target: anchor moment)
**Goal**: `dectl project context` delivers maximum relevant information within token limits, the anchor moment flow is tested end-to-end, and onboarding takes <60 seconds.

**Deliverables**:
- **Proportional token budget**: `dectl project context` assigns budgets per section with weights (last_session 25%, isa 20%, config 15%, progress 10%, integration 5%, decisions 25%); files that don't exist are skipped and their weight redistributed
- `truncate_to_budget()` — character-based truncation per file (≈4.5 chars/token) with footer
- `calculate_budgets()` — iterative surplus redistribution (up to 5 iterations, 1% threshold)
- `--format compact` — 6-line summary (project, stack, last_session, progress, decisions, memory_hits) via `CompactOutput` struct; JSON envelope supported
- **Recent changes prioritization** — `parse_session_date()` reads `**Fecha**:` from `last_session.md`; compares file mtimes vs session date; changed files get ×2 weight, unchanged ×0.5
- **E2E anchor moment test** (`tests/e2e_anchor.rs`): legacy Rust/Axum project → `init --standard` → verify `.dec/` structure + stack detection + context text/compact output
- **Onboarding**: README anchor moment demo in <30s, quickstart.md with 3 scenarios (legacy, new, team)

**Tests added**: 9 (3 P001, 2 P002, 2 P003, 2 P004)

**Requirements covered**: REQ-C-013 (updated)

### Phase 8 — SDD Spec Generator (target: structured spec creation)
**Goal**: A single command ensures SDD methodology exists in .dec/ and signals the AI agent to interview the user and create specs/ documents with real content.

**Deliverables**:
- `dectl spec init` — creates `.dec/sdd/` with SKILL.md + references/ (idempotent)
- Bridge updates: `.dec/config/project.toml` with `[specs] dir = "specs"`, `.dec/isa/project.isa.md` with SDD link
- Templates embedded as const &str (SKILL.md, references/templates.md, references/examples.md)
- Full integration into `dectl project init --standard` (sdd/ files auto-created)
- 6 integration tests (creates dir, bridge updates, error case, JSON output, idempotent, standard init includes sdd/)
- Agent interview flow: model reads SKILL.md + templates.md → interviews user → creates specs/

**Requirements covered**: REQ-009

**Tests added**: 6 (in tests/spec_init.rs)

### Phase 6 — Agent System (target: specialized roles)
**Goal**: Developers and models can invoke specialized agents for coding, reviewing, researching, and documenting.

**Deliverables**:
- `dectl agent list` — shows built-in and custom agents
- `dectl agent describe <type>` — shows full agent definition
- `dectl agent run <type> --task "..."` — executes agent with context
- `dectl agent run --parallel <type1>,<type2>` — parallel execution
- 4 built-in agents: coder, reviewer, researcher, documenter
- Custom agents in `.dec/agents/*.yaml`
- `agent_log` table in memory.db for audit trail
- `agent_sync` as Paso 5 in `dectl session end`
- New StepType `agent` in workflows

**Requirements covered**: REQ-A-001 to REQ-A-009

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| SQLite corruption on interrupted write | Low | High | Always use transactions; WAL mode enabled |
| Workflow YAML schema breaks between versions | Medium | Medium | Version field in YAML; migration tool in Phase 3 |
| `.gitignore` edge cases causing wrong scan output | Low | Low | Delegate entirely to `ignore` crate; don't reimplement |
| Binary size exceeds 20MB with all deps | Low | Medium | Monitor with `cargo bloat`; strip debug symbols in release |
| AI model generates malformed workflow YAML | Medium | Low | Validate schema on load; report specific parse errors |

---

## Dependencies & Prerequisites

- [ ] Rust stable toolchain installed (`rustup`)
- [ ] `cargo` available in PATH
- [ ] Target platforms decided for CI: Linux x86_64, macOS arm64, macOS x86_64

## Testing Approach

- **Unit tests**: in-module `#[cfg(test)]` blocks for memory operations, config parsing, workflow schema validation
- **Integration tests**: `tests/` directory invokes the compiled binary via `std::process::Command`, checks stdout/stderr/exit codes
- **Fixtures**: sample `.dec/` folders and workflow YAML files in `tests/fixtures/`
- **No live model required**: all tests use static inputs and expected outputs
