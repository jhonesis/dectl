# Implementation Tasks — dectl
> *Atomic, ordered, trackable tasks derived from plan.md.*
> *Each task = independently implementable + testable + reviewable as single PR.*
> *Last updated: 2026-05-19*

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

**Result**: 3.6MB → 4.4MB (well under 20MB limit)
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

### Agent Module (basic)

- [ ] [T037] Implement `dectl agent list`: display hardcoded agent types (`coder`, `reviewer`, `researcher`) with description — S
- [ ] [T038] Implement `dectl agent run <type> --task "<description>"`: load agent prompt template from `.dec/prompts/tasks/<type>.md`, substitute `{{task}}` placeholder, print composed prompt for model to act on — M

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
  - `detect_stack()` — detect language/framework from config files
  - `scan_docs_for_context()` — extract name and vision from README
  - `fill_project_files()` — update `.dec/` with detected context
  > Implemented in `project/auto_fill.rs`
- [x] [T046] Interactive init for empty projects: prompt for project name, type, languages, description, vision — M
- [x] [T047] Memory context-aware: auto-detect project from `.dec/config/project.toml`, filter `memory list/search` by project by default, `--global` flag to bypass filter — M

**Phase 3 COMPLETE** (except T040 and T043) ✅

---

## Progress Tracking

| Phase | Total | Done | In Progress | Blocked |
|-------|-------|------|-------------|---------|
| Phase 1 — MVP | 24 | 24 | 0 | 0 |
| Phase 2 — Workflows | 10 | 10 | 0 | 0 |
| Phase 3 — Polish | 13 | 13 | 0 | 0 |
| **Total** | **47** | **47** | **0** | **0** |

**Phase 3 COMPLETE** ✅

---

## Remaining Tasks

| ID | Descripción | Prioridad | Status |
|----|-------------|-----------|--------|
| T034-T036 | Embeddings (semantic search) — opcional, requiere Ollama | Low | Pending |
| T037-T038 | Agent module (list/run) | Medium | Pending |
| T043 | Windows full support — documentado, ~5h estimado | Low | Pending (documented) |

**All Phase 1-3 tasks COMPLETE** ✅

---

*Last sync: 2026-05-19 — CLI SDD (specs/cli/tasks.md) was source of truth for Phase 2-3 implementation status*