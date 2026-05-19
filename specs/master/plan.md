# Technical Implementation Plan — dectl
> *Technology-specific. Describes HOW to build what spec.md defines.*
> *Version: 1.0 | Status: Draft | Last updated: 2026-05-13*

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
│  prompts/          │    │  protocol  (exec-from)    │
│  knowledge/        │    │                           │
│  state/            │    └──────────────┬────────────┘
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
│   │   ├── error.rs             ← error types
│   │   └── output.rs            ← text/json output formatting
│   ├── project/
│   │   ├── mod.rs
│   │   ├── init.rs              ← dectl project init
│   │   ├── info.rs              ← dectl project info
│   │   ├── scan.rs              ← dectl project scan (uses ignore crate)
│   │   └── templates.rs         ← .dec/ folder templates
│   ├── memory/
│   │   ├── mod.rs
│   │   ├── db.rs                ← SQLite connection + migrations
│   │   ├── add.rs               ← dectl memory add
│   │   ├── list.rs              ← dectl memory list
│   │   ├── search.rs            ← dectl memory search
│   │   └── show.rs              ← dectl memory show <id>
│   ├── workflow/
│   │   ├── mod.rs
│   │   ├── schema.rs            ← Workflow, Step, StepType structs
│   │   ├── loader.rs            ← YAML parsing + validation
│   │   ├── runner.rs            ← step execution logic
│   │   ├── trust.rs             ← trust registry (trust.toml)
│   │   ├── list.rs              ← dectl workflow list
│   │   ├── run.rs               ← dectl workflow run <name>
│   │   └── describe.rs          ← dectl workflow describe <name>
│   └── protocol/
│       ├── mod.rs
│       └── exec.rs              ← dectl exec-from-file <path>
└── tests/
    ├── project_tests.rs
    ├── memory_tests.rs
    └── workflow_tests.rs
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
  → core/output: print summary of created files + next steps
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
│   │   └── base.md          ← base system prompt for the model
│   └── tasks/               ← task-specific prompts
├── knowledge/
│   ├── glossary.md
│   └── constraints.md
└── state/
    └── progress.json        ← feature status tracking
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

### Phase 2 — Workflows + Agents (target: full automation loop)
**Goal**: Developers and models can define and execute multi-step workflows. Trust system prevents accidental execution.

**Deliverables**:
- `dectl workflow list`
- `dectl workflow run <name>` with trust confirmation
- `dectl workflow describe <name>`
- `dectl exec-from-file <path>` — protocol module
- Semantic memory search (embeddings via local model, optional)
- `dectl agent run <type> --task "..."` — agent module (basic)

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
