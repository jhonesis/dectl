# dectl — Reviewers Guide

Thanks for checking out dectl! This guide helps you record a review in under 30 minutes.

---

## What is dectl?

dectl is a **model-agnostic developer life OS**. A single 4.5MB Rust binary that gives any AI coding environment three things:

- **Persistent memory** — SQLite with WAL mode, tag-based search, per-project filtering
- **Executable workflows** — YAML-defined pipelines with variables, dry-run, and step recovery
- **Structured project context** — the `.dec/` directory system that any AI model reads instantly

No proprietary API. No telemetry. Zero network dependencies for core functionality. MIT license.

---

## Why it's unique

- **Model-agnostic**: works with Claude, Gemini, ChatGPT, Ollama, Qwen, Phi — or a human in a terminal
- **Local-first**: zero cloud dependencies, no vendor lock-in, no data leaves your machine
- **4 built-in agents**: researcher → coder → reviewer → documenter pipeline with `next_step_hint`
- **Session management**: `dectl session end` captures git changes, decisions, config diffs in one command
- **SDD Spec Generator**: generates 5 artifacts (constitution, spec, requirements, research, tasks) from one command
- **Auto-detect stack**: scans any project directory and auto-fills `.dec/` context on init
- **One static binary**: ~4.5MB, no Electron, no runtime dependencies, no JS
- **AI-first by design**: when opened in a conversational IDE (like opencode, Claude Code, Gemini CLI), the model runs `dectl project info --json` automatically and understands the full project — stack, architecture, decisions, next steps — without any manual context loading

---

## Preparation (before recording)

```bash
# 1. Install dectl
curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/scripts/install.sh | bash

# 2. Verify it works
dectl --version

# 3. (Optional) Run the auto-demo script
bash <(curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/scripts/dectl-demo.sh)
```

You'll also want a non-empty project to demo `project init --standard`. Any Rust, TypeScript, Python, or Go project works — dectl auto-detects the stack.

---

## 3 Demos for your video

### Demo 1 — Quick start (2 minutes)

Show how fast a new developer gets started:

```bash
# Init a project with full context
dectl project init --full

# See what was created
ls -la .dec/

# Show project summary
dectl project info --json
```

**What to highlight**: The `.dec/` directory structure is created in milliseconds. The AI auto-fills framework detection, language stack, and project description. Compare this to manually writing context files.

### Demo 2 — Memory in action (3 minutes)

Show persistent memory across sessions:

```bash
# Add context
dectl memory add "This project uses Rust with Axum for HTTP and SQLite for storage" --tags architecture

# Add decision
dectl memory add "Chose Axum over Actix for simpler middleware" --tags decisions

# Search later
dectl memory search "architecture"

# Show all memories
dectl memory list --limit 10
```

**What to highlight**: The anchor moment — open a project months later, run `dectl memory search`, and the AI already understands architecture decisions. No re-explaining.

### Demo 3 — Workflow + Agents (4 minutes)

Show the automation pipeline:

```bash
# List available workflows
dectl workflow list

# Run a workflow in dry-run mode (safe preview)
dectl workflow run execute_task --var task_id=42 --dry-run

# List built-in agents
dectl agent list

# Run an agent in dry-run mode
dectl agent run researcher --task "Analyze project structure" --dry-run

# Run the full SDD pipeline
dectl workflow run execute_task --var task_id=42 --description="Add user authentication" --auto
```

**What to highlight**: The trust system (auto-prompt on first action step), `--dry-run` for safe previews, and how `--auto` enables CI/CD pipelines. The runner shows exactly which step is executing and what comes next.

### Demo 4 — AI reads context automatically (2 minutes)

This is the **key differentiator**. Show how the AI model understands the project without any manual explanation:

```bash
# After dectl project init, open the project in any conversational IDE
# The AI model runs this automatically:
dectl project info --json

# The model sees: stack, frameworks, description, decisions, next steps
# No need to tell the AI "this is a Rust project with Axum" — it already knows
```

**What to highlight**: This is the "invisible integration" — the AI reads `.dec/` context automatically via `project info --json` at session start. The developer doesn't upload files, doesn't write system prompts, doesn't re-explain. Open the project and the model already understands everything.

---

## What to highlight in your review

| Angle | Hook | Why it works |
|-------|------|-------------|
| **"It's just files and a binary"** | Model-agnostic philosophy | Devs are tired of vendor lock-in |
| **The anchor moment** | Open a legacy project months later | Relatable pain: lost context |
| **Session automation** | `session end` saves hours | Manual bookkeeping is a universal pain |
| **4.5MB vs Electron** | No runtime, no JS, no bloat | Performance-conscious devs |
| **SDD workflow** | From idea to documentation in one command | Resonates with builders |

---

## Target audience for your video

Your viewers will love dectl if they:

- Code with AI assistants and re-explain context each session
- Maintain multiple projects and lose track of decisions
- Care about local-first, privacy-preserving tools
- Appreciate well-crafted CLI tools in Rust
- Want reproducible dev environments without Docker

---

## Resources

- **Landing**: https://dectl.dev
- **GitHub**: https://github.com/jhonesis/dectl
- **Documentation**: https://deepwiki.com/jhonesis/dectl
- **Install script**: `curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/scripts/install.sh | bash`
- **Auto-demo script**: `bash <(curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/scripts/dectl-demo.sh)`
- **Contact**: jhonesis@proton.me (questions, early access, collab)

---

*Questions? Feedback? Found a bug? Open an issue on GitHub or email directly. Happy to jump on a quick call if you need clarifications for your review.*
