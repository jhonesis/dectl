# Project Constitution — dectl
> *Governing principles for dectl (Dev Environment Control). This document is the supreme authority — all other documents must comply with it.*

---

## 1. Project Identity

- **Name**: dectl (Dev Environment Control)
- **Purpose**: A model-agnostic developer life OS that provides persistent memory, executable workflows, and structured project context — operable by any AI coding environment or human via terminal.
- **Owners**: Open source community

---

## 2. Core Principles

1. **Model-agnostic by design** — dectl must work with any AI environment (Claude Code, Gemini CLI, Qwen CLI, Phi, Ollama, or a human) without modification. No component may assume a specific model or provider.
2. **Local-first** — all data lives on the developer's machine. No cloud dependency, no telemetry, no external API required to function.
3. **Separation of concerns** — three actors, three responsibilities: the model thinks, the CLI executes, `.dec/` defines context. These roles must never bleed into each other.
4. **Executable over textual** — workflows are not documentation; they are instructions that produce real actions. Prefer executable steps over prose descriptions.
5. **Simplicity over completeness** — a command that does one thing well beats a command that does ten things poorly. Optimize for models with 7B–14B parameters.
6. **Contracts over assumptions** — every interface between components must be explicit and documented. The model reads `.dec/`; it does not guess at it.
7. **Composable and extensible** — any module can be used independently. Projects can adopt `.dec/` without the CLI, or use the CLI without all `.dec/` features.

---

## 3. Technology Constraints

### Mandatory Stack
- **CLI language**: Rust (performance, portability, single binary distribution)
- **Memory backend**: SQLite via `rusqlite`
- **Workflow format**: YAML (human-readable, model-writable)
- **Config format**: TOML (`~/.dectl/config.toml`, `project.toml`)
- **Context format**: Markdown (`.dec/**/*.md`) — readable by any model, any tool, `cat`

### Forbidden Technologies
- No cloud storage or sync for core features (optional integrations may exist)
- No model-specific APIs in core modules (no Anthropic SDK, no OpenAI SDK in CLI)
- No GUI frameworks in the CLI binary
- No runtime dependencies for the binary (must ship as a single static binary)

### Required Integrations (optional modules, not core)
- Git (read-only: branch, diff, log)
- Docker (optional agent workflows)
- Local LLM servers (Ollama, llama.cpp) — via external commands only, not embedded

---

## 4. Coding Standards

- **Style guide**: `rustfmt` default formatting, enforced via CI
- **Naming conventions**: `snake_case` for Rust, `kebab-case` for CLI commands (`dectl memory add`, `dectl workflow run`)
- **Folder structure**: one module = one subfolder under `src/` with `mod.rs` entry point
- **Comment policy**: public functions must have doc comments; internal logic commented only when non-obvious
- **Error handling**: use `anyhow` for application errors; never panic in production paths
- **CLI output**: structured output always includes a machine-readable option (`--json` flag on all commands)

---

## 5. Testing Strategy

- **Unit tests**: required for all business logic (memory operations, workflow parsing, config loading)
- **Integration tests**: required for all CLI commands (test actual binary invocation)
- **E2E tests**: required for full workflow execution (Phase 2+)
- **Coverage target**: 70% minimum for Phase 1 MVP; 80% for Phase 2+
- **Model interaction tests**: use fixture-based approach (no live model required)

---

## 6. Security Non-Negotiables

- No secrets stored in `.dec/` files (use env vars or system keychain)
- `.dec/` must be `.gitignore`-able by default (template provided)
- Memory database (`~/.dectl/memory.db`) is local-only, never synced automatically
- No arbitrary code execution from workflow YAML without user confirmation on first run
- All file writes from the CLI must be within the project directory or `~/.dectl/`

---

## 7. Definition of Done

A task is complete when:
- [ ] Code implements the spec requirement and matches the plan
- [ ] Unit tests pass
- [ ] CLI command works end-to-end (manual or integration test)
- [ ] `--help` output is accurate and complete for any new command
- [ ] No new `clippy` warnings
- [ ] Spec/plan updated if implementation deviated from original design
- [ ] `CLAUDE.md` (or equivalent agent context file) updated if new commands are added
