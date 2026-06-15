# Implementation Tasks — dectl
> *Atomic, ordered, trackable tasks derived from plan.md.*
> *Each task = independently implementable + testable + reviewable as single PR.*
> *Last updated: 2026-06-12*

---

## Legend

- `[Txxx]` = Task ID
- `[P]` = Can run in parallel with other `[P]` tasks in the same phase
- `S / M / L` = Estimated complexity (Small / Medium / Large)
- `(REQ-xxx)` = Traceability to spec requirement

---

## Phase 1 — MVP ✅ COMPLETE

### Setup

- [x] [T001] Initialize Rust project with `cargo new dec --bin` and configure `Cargo.toml` with all Phase 1 dependencies (`clap`, `rusqlite`, `serde`, `serde_yaml`, `serde_json`, `anyhow`, `toml`, `chrono`, `ignore`) — S
- [x] [T002] Create full `src/` module structure: `core/`, `project/`, `memory/`, `protocol/` with empty `mod.rs` files and register all modules in `main.rs` — S
- [x] [T003] Configure `rustfmt.toml` and `clippy` settings per constitution standards; add `Makefile` or `justfile` with `fmt`, `lint`, `test`, `build-release` targets — S

---

### Core Module

- [x] [T004] Implement `core/config.rs`: load `~/.dectl/config.toml` on startup, create default file if absent, expose typed `GlobalConfig` struct — M (REQ-007)
- [x] [T005] Implement `core/config.rs` project config: load `.dec/config/project.toml` if present in current directory, merge with global config (project overrides global) — M (REQ-007)
- [x] [T006] Implement `core/output.rs`: two output modes — human-readable (colored, formatted) and `--json` (valid JSON with `status: "ok"` or `status: "error"` envelope); expose `Output` helper used by all commands — M (REQ-006)
- [x] [T007] Wire `--json` as a global flag in `main.rs` via `clap` so any subcommand can inherit it without redeclaring — S (REQ-006)

---

### Project Module

- [x] [T008] Implement `project/templates.rs`: define the full `.dec/` folder structure as embedded string templates (config, isa, decisions, workflows, prompts, knowledge, state) — M (REQ-001)
- [x] [T009] Implement `dectl project init`: check `.dec/` does not exist → create folder tree from templates → print summary of created files and next recommended step; abort with clear message if `.dec/` already exists — M (REQ-001)
- [x] [T010][P] Implement `dectl project info`: read `.dec/config/project.toml` and `.dec/isa/project.isa.md`, display structured summary (project name, type, stack, ISA excerpt); handle missing files gracefully with per-file warnings — M (REQ-002)
- [x] [T011][P] Implement `dectl project scan`: walk current directory using `ignore` crate (respects `.gitignore`, `.git/info/exclude`), print file tree excluding common noise; support `--depth <n>` flag to limit traversal depth — M (REQ-005)
- [x] [T012] Write integration tests for `project` commands: `init` creates expected structure, `init` aborts on existing `.dec/`, `info` handles missing files, `scan` excludes gitignored paths — M

---

### Memory Module

- [x] [T013] Implement `memory/db.rs`: open (or create) `~/.dectl/memory.db`, enable WAL mode, run pending migrations on every open; expose `DbConn` wrapper used by all memory commands — M (REQ-003)
- [x] [T014] Implement migration `0001_initial`: create `memories` table and `migrations` table with schema from data-model.md; create indexes `idx_memories_project` and `idx_memories_created_at` — S (REQ-003)
- [x] [T015][P] Implement `dectl memory add "<content>"`: accept `--tags t1,t2` and `--project <name>` flags (auto-detect project from `.dec/config/project.toml` if flag absent); INSERT into `memories`; print confirmation with assigned ID — M (REQ-003)
- [x] [T016][P] Implement `dectl memory list`: SELECT from `memories` ORDER BY `created_at DESC` with default limit from config (`max_results`); support `--project <name>` filter and `--limit <n>` override; display ID, date, tag, content preview — M (REQ-003)
- [x] [T017][P] Implement `dectl memory search "<query>"`: keyword search via SQLite `LIKE` on `content` and `tags`; support `--project <name>` filter; display results with match context — M (REQ-003)
- [x] [T018][P] Implement `dectl memory show <id>`: SELECT by ID, render full Markdown content; return clear error if ID not found — S (REQ-003)
- [x] [T019] Write unit tests for all memory operations: add/list/search/show against a temporary in-memory SQLite database; test tag filtering, project scoping, missing ID error — M

---

### Protocol Module

- [x] [T020] Implement `dectl exec-from-file <path>`: read a plain text file where each non-empty line is a `dectl` subcommand; execute each line in order via `std::process::Command`; stop on first failure and report which line failed — M

---

### Phase 1 Validation

- [x] [T021] Write end-to-end integration test: init project → add 3 memories → list → search → show, all via binary invocation; verify `--json` output is valid parseable JSON on every command — L
- [x] [T022] Verify binary size on release build does not exceed 20MB (`cargo build --release` + `strip`); document result in `specs/master/research.md` — S

**Result**: 3.6MB → ~4.5MB (well under 20MB limit)
- [x] [T023] Write `--help` text review: run every Phase 1 command with `--help`, verify output is accurate, includes usage example, and lists all flags — S

**All help text verified**: main, project, memory, workflow, init, add, search, run
- [x] [T024] Write `CLAUDE.md` agent context file at project root pointing to all SDD artifacts and listing all Phase 1 commands with one-line descriptions — S

**Phase 1 COMPLETE** ✅

---

## Phase 2 — Workflows + Agents ✅ COMPLETE

### Workflow Module

- [x] [T025] Implement `workflow/schema.rs`: define `Workflow`, `Step`, `StepType` (prompt / action / write) structs with full `serde` deserialization from YAML schema defined in plan.md — M (REQ-004)
  > Equivalent CLI SDD: C024
- [x] [T026] Implement `workflow/loader.rs`: read `.dec/workflows/<name>.yaml`, parse into `Workflow`, validate all required fields present, return structured error on malformed YAML — M (REQ-004)
  > Equivalent CLI SDD: C025
- [x] [T027] Implement `workflow/trust.rs`: read/write `~/.dectl/trust.toml`; check if `<project_path>/<workflow_name>` is trusted; if not and workflow has `action` steps, prompt user Y/n once and persist decision — M (REQ-004)
  > Equivalent CLI SDD: C026
- [x] [T028] Implement `workflow/runner.rs`: execute steps in order — `prompt` prints content and pauses, `action` runs via `std::process::Command` (or `sh -c` if `shell: true`), `write` writes content to target path; capture and display step output; stop and report on failure — L (REQ-004)
  > Equivalent CLI SDD: C028
- [x] [T029][P] Implement `dectl workflow list`: scan `.dec/workflows/`, parse each YAML for `name` and `description`, display as table — S (REQ-005)
  > Equivalent CLI SDD: C029 ✅
- [x] [T030][P] Implement `dectl workflow describe <name>`: load workflow, display all steps with type, content preview, and command — S (REQ-005)
  > Equivalent CLI SDD: C030 ✅
- [x] [T031] Implement `dectl workflow run <name>`: orchestrate loader → trust check → runner; support `--dry-run` flag to print steps without executing `action` steps — M (REQ-004)
  > Equivalent CLI SDD: C031 ✅
- [x] [T032] Write unit tests for workflow schema parsing: valid YAML, missing required fields, unknown step types, `shell: true` flag — M
  > Part of CLI SDD tests
- [x] [T033] Write integration tests for workflow runner: dry-run mode, prompt step pauses, action step captures output, write step creates file, failure stops execution — L
  > Part of CLI SDD tests

---

### Embeddings (optional, Phase 2)

- [ ] [T034] Implement migration `0002_embeddings`: create `embeddings` table with schema from data-model.md — S
- [ ] [T035] Implement `dectl memory embed-all --model <name>`: call local embedding model via Ollama HTTP API for each memory entry lacking an embedding; store serialized float32 vector as BLOB — L
- [ ] [T036] Implement `dectl memory search-semantic "<query>"`: embed query, load all vectors from `embeddings`, compute cosine similarity in Rust, return top-N results sorted by score — L

---

### Agent Module (Phase 6)

> Full specification in `specs/agents/`. Tasks A001-A019.

- [ ] See `specs/agents/tasks.md` for complete agent implementation tasks
- [ ] A001-A009: Individual agents (schema, loader, built-ins, list, describe, log, runner, run, tests)
- [ ] A010-A014: Parallelism + custom agents + workflow integration
- [ ] A015-A019: Session end integration + polish

---

## Phase 3 — Polish + Ecosystem

### CLI Polish

- [x] [T039] Generate shell completions for bash, zsh, and fish via `clap_complete`; document installation in README — M
  > Implemented as `dectl generate-completions bash|zsh|fish`
- [x] [T040] Implement project type templates: `dectl project init --type api|cli|microservice` creates type-specific `.dec/` files (workflows, prompts, ISA structure) — L
  > Implemented: `--type` flag with api/cli/microservice/other; type-specific workflows and prompts for each type

### Memory Enhancements

- [x] [T041] Implement `dectl memory delete <id>`: soft-delete (mark as deleted, not removed) with `--hard` flag for permanent removal — S
  > Implemented: soft-delete by default, `--hard` for permanent, `--non-interactive` support
- [x] [T042] Implement `dectl memory edit <id>`: open memory entry content in `$EDITOR` for modification — S
  > Implemented: opens in `$EDITOR` (respects `GlobalConfig.core.default_editor`)

### Platform & Documentation

- [x] [T043] Investigate and document Windows support: test Phase 1 binary on Windows (WSL and native); document gaps in `research.md` — M
  > Investigated: critical issue in `runner.rs` (sh -c), high in `config.rs` and `db.rs` (HOME env). See `specs/master/research.md` for details.

### Platform & Documentation
- [x] [T044] Write user-facing README with quickstart, command reference, and workflow authoring guide — L
  > Implemented: `README.md` with all commands documented

### Auto-fill + Interactive Init

- [x] [T045] Implement auto-fill in `dectl project init`:
  - `is_project_empty()` — detect if project has existing code
  - `detect_stack()` — detect language from config files
  - `scan_docs_for_context()` — extract project name from README (basic fallback)
  - `fill_project_files()` — update `.dec/` with detected context
  > Implemented in `project/auto_fill.rs`
- [x] [T046] Interactive init for empty projects: prompt for project name, type, languages, description, vision — M
- [x] [T047] Memory context-aware: auto-detect project from `.dec/config/project.toml`, filter `memory list/search` by project by default, `--global` flag to bypass filter — M

**Phase 3 COMPLETE** (except T040 and T043) ✅

---

## Phase 7 — Polish & Adopción ✅ COMPLETE

| ID | Tarea | Archivos | Status |
|----|-------|----------|--------|
| P001 | Proportional token budget for project context | `context.rs` | ✅ |
| P002 | `--format compact` for project context | `context.rs` | ✅ |
| P003 | Recent changes prioritization | `context.rs` | ✅ |
| P004 | E2E anchor moment test script | `tests/e2e_anchor.rs` | ✅ |
| P005 | Onboarding quickstart + README update | `README.md`, `docs/user/quickstart.md` | ✅ |
| P006 | Update specs, tasks.md, CLAUDE.md | varios | ✅ |

**Phase 7 COMPLETE** ✅

---

## Phase 8 — SDD Spec Generator ✅ COMPLETE

| ID | Tarea | Archivos | Esfuerzo | Depende de | Estado |
|----|-------|----------|----------|------------|--------|
| SPC001 | spec module scaffold + CLI command | spec/mod.rs, spec/init.rs, main.rs | S | Phase 7 | ✅ |
| SPC002 | SDD templates embebidos + bridge | spec/templates.rs, spec/bridge.rs, spec/init.rs | M | SPC001 | ✅ |
| SPC003 | Integrar sdd/ en project init --standard | project/templates.rs | S | SPC002 | ✅ |
| SPC004 | Tests + polish | tests/spec_init.rs, config.rs | M | SPC003 | ✅ |

**Total tasks**: 4/4 ✅
**111 tests passing** (25 unit + 86 integration)

---

## Phase 9 — Memory Improvements ✅ COMPLETE

*FTS5 full-text search, type categorization, agent→memory auto-link, query language.*

| ID | Tarea | Archivos | Esfuerzo | Estado |
|----|-------|----------|----------|--------|
| T119 | Migration v2 (FTS5 + type column): bundled→bundled-fts5, FTS5 virtual table + triggers, ALTER TABLE ADD type | `memory/db.rs` | M | ✅ |
| T120 | Migration v3 (agent_outputs) + auto-link: CREATE TABLE agent_outputs, auto-INSERT resumen en memories al completar agente | `memory/db.rs`, `agent/runner.rs`, `agent/log.rs` | M | ✅ |
| T121 | Migration v4 (tag_taxonomy) + query language: CREATE TABLE tag_taxonomy + 9 seed tags, tokenizer → parser → SQL builder parametrizado | `memory/db.rs`, `memory/query.rs` | L | ✅ |
| T122 | Integration tests (139 total): query language (5 integration + 13 unit), FTS5 search, type system, auto-link tests | tests/ | M | ✅ |

**Total tasks**: 4/4 ✅
**139 tests passing** (44 unit + 95 integration)

---

## Phase 4 — Session Management ✅ COMPLETE

### Session Module

- [x] [S001] Create `session/` module structure: `mod.rs`, register in `main.rs` — S
- [x] [S002] Define session data types: `SessionSummary`, `GitChanges`, `CapturedDecision`, `SessionEndResult` — S (REQ-008)
- [x] [S003] Implement `session_summary.rs`: `generate_session_summary()` from git log + previous session — M (REQ-008)
- [x] [S004] Implement `write_last_session()`: format and write `.dec/state/last_session.md`, support `--dry-run` — S (REQ-008)
- [x] [S005] Implement `git_sync.rs`: `detect_git_changes()` via `git diff` + `git log` — M (REQ-008)
- [x] [S006] Implement `sync_progress()`: update `progress.json` from git changes, detect new features — M (REQ-008)
- [x] [S007] Implement `decision_capture.rs`: `capture_decisions()` via regex patterns on session text — M (REQ-008)
- [x] [S008] Implement `save_decisions()`: INSERT new decisions into memory.db, avoid duplicates — S (REQ-008)
- [x] [S009] Wire `dectl session end` in `main.rs` with `--dry-run`, `--skip-git` flags — S (REQ-008)
- [x] [S010] Implement `session/end.rs`: orchestrate 3 steps independently, collect results, output summary — M (REQ-008)
- [x] [S011] Write integration tests for session end: dry-run, skip-git, JSON output, no git repo, with .dec/ — M
- [x] [S012] Update documentation: README.md, CLAUDE.md, last_session.md — S

**Phase 4 COMPLETE** ✅

---

## Progress Tracking

| Phase | Total | Done | In Progress | Blocked |
|-------|-------|------|-------------|---------|
| Phase 1 — MVP | 24 | 24 | 0 | 0 |
| Phase 2 — Workflows | 10 | 10 | 0 | 0 |
| Phase 3 — Polish | 13 | 13 | 0 | 0 |
| Phase 4 — Session | 12 | 12 | 0 | 0 |
| Phase 5 — Config Sync | 15 | 15 | 0 | 0 |
| Phase 6 — Agents | 19 | 19 | 0 | 0 |
| Phase 7 — Polish & Adopción | 6 | 6 | 0 | 0 |
| Phase 8 — SDD Spec Generator | 4 | 4 | 0 | 0 |
| Phase 10 — Code Quality | 4 | 4 | 0 | 0 |
| Agent Cycle — SDD Pipeline | 11 | 11 | 0 | 0 |
| Phase 9 — Memory Improvements | 4 | 4 | 0 | 0 |
| **Total** | **122** | **122** | **0** | **0** |

**Phase 5 COMPLETE** ✅

---

## Remaining Tasks

| ID | Descripción | Prioridad | Status |
|----|-------------|-----------|--------|
| T034-T036 | Embeddings (semantic search) — opcional, requiere Ollama | Low | Pending |
| A001-A019 | Agent module (full spec in `specs/agents/tasks.md`) | Medium | ✅ Complete (19/19) |
| T043 | Windows full support — documentado, ~5h estimado | Low | Pending (documented) |

**All phases 1-8 COMPLETE** ✅

---

*Last sync: 2026-06-12 — All phases complete, 139 tests passed, FTS5, type system, query language, auto-link agent→memory, tag taxonomy*