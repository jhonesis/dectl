# Developer Documentation — dectl

> *Guide for contributing to dectl development.*

## Project Structure

```
dectl/
├── src/
│   ├── main.rs           # Full CLI application
│   ├── core/             # Configuration and output
│   ├── project/          # Project commands
│   ├── memory/           # Memory CRUD operations
│   ├── protocol/         # exec-from-file protocol
│   └── workflow/         # Workflow module
├── tests/                # Integration tests
├── docs/                 # Documentation
└── specs/                # SDD (Software Design Documents)
    ├── master/           # Overall vision
    ├── dot-dec/          # .dec/ system
    ├── cli/              # CLI binary
    ├── integration/      # Actor integration
    └── development/       # This documentation
```

## Development Commands

```bash
# Build
cargo build
cargo build --release

# Tests
cargo test                    # All tests
cargo test --test e2e_integration  # E2E tests
cargo test --test project_commands # Project module tests
cargo test --test memory_commands  # Memory module tests
cargo test --test protocol_commands # Protocol tests

# Linting
cargo fmt
cargo clippy -- -D warnings

# Binary size
du -sh target/release/dectl  # must be < 20MB
```

## Current Status — ALL PHASES COMPLETE

### Phase 1 — MVP ✅
- [x] Project setup
- [x] Core module (config, output, errors)
- [x] Project commands (init, info, scan)
- [x] Memory module (add, list, search, show, delete, edit)
- [x] Protocol (exec-from-file)
- [x] Tests (51+ passing)
- [x] Binary (~4.5MB)

### Phase 2 — Workflows ✅
- [x] Workflow schema and loader
- [x] Trust system for action steps
- [x] Workflow runner with variable interpolation
- [x] Commands: list, describe, run

### Phase 3 — Polish ✅
- [x] Memory delete with --hard flag
- [x] Memory edit with $EDITOR
- [x] Shell completions (bash, zsh, fish)
- [x] --non-interactive validation

### Phase 4 — Integration ✅
- [x] project context command
- [x] Integration tests
- [x] Public README.md
- [x] AGENTS.md auto-generation

### Phase 5 — Auto-fill + Interactive Init ✅
- [x] Stack detection (Rust, Node.js, Go, Python, Java, etc.)
- [x] Docs scanning for context
- [x] Interactive prompts for empty projects
- [x] Type templates (API, CLI, Microservice)
- [x] Memory per-project with auto-detection

## Getting Started

1. Choose a task from `specs/*/tasks.md`
2. Read the corresponding spec and plan
3. Run `cargo test` to verify everything works

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   .dec/     │    │    dectl    │    │   Model     │
│  (files)    │───▶│  (binary)   │◀───│  (AI/CLI)   │
└─────────────┘    └──────┬──────┘    └─────────────┘
                          │
                   ┌──────▼──────┐
                   │ ~/.dectl/   │
                   │  memory.db │
                   └─────────────┘
```

### The Three Actors

1. **`.dec/`** — Project context (Markdown + YAML + TOML + JSON)
   - Readable by any AI without installation
   - Structured schemas for consistency

2. **`dectl` binary** — Executor (Rust, SQLite bundled)
   - ~4.5MB static binary
   - No runtime dependencies
   - Model-agnostic

3. **Model/Environment** — Thinks, generates code, invokes `dectl`
   - No vendor lock-in
   - Works with Claude Code, Gemini CLI, Qwen CLI, Ollama, etc.

## Testing Strategy

| Test Type | Coverage | Run Command |
|-----------|----------|-------------|
| Unit | Core logic | `cargo test --lib` |
| Integration | Modules | `cargo test --test *_commands` |
| E2E | Full workflow | `cargo test --test e2e_integration` |

## Contributing

1. Choose a task from `specs/*/tasks.md`
2. Read the corresponding spec and plan
3. Implement following the checkpoints
4. Ensure all tests pass
5. Verify `cargo fmt && cargo clippy` are clean

## Requirements

- Rust 1.70+
- Linux/macOS (Windows via WSL)

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point with clap |
| `src/core/` | Config, output, error handling |
| `src/project/` | Project init, info, scan, context |
| `src/memory/` | SQLite operations |
| `src/workflow/` | YAML workflow execution |
| `src/protocol/` | exec-from-file command |

See: [CONTRIBUTING.md](../../CONTRIBUTING.md)