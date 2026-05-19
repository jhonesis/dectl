# User Guide — dectl

> *How to use dectl in your development project.*

---

## What is dectl?

dectl is a **Developer Life OS** that gives you (and any AI you use) persistent memory, executable workflows, and structured project context.

**Three key concepts:**
- `.dec/` — hidden folder where all the project context lives
- `dectl` — the binary that runs commands
- Global memory — knowledge shared across projects

---

## Quick Start for a New Project

### Step 1: Install dectl

```bash
# Option A: Download the pre‑compiled binary
# (move it to your PATH, e.g. ~/bin/dectl or /usr/local/bin/dectl)

# Option B: Build from source
git clone https://github.com/jhonesis/dectl.git
cd dectl
cargo build --release
sudo cp target/release/dectl /usr/local/bin/
```

Verify the installation:

```bash
dectl --version
# → dectl 0.1.0
```

---

### Step 2: Create a `.dec/` folder in your project

```bash
cd ~/projects/my-project

dectl project init --standard
```

**What does this command do?**
- Creates the hidden `.dec/` folder (does not interfere with your code)
- Generates configuration files, vision documents, workflows, etc.
- Does **not** modify any of your existing project files

**Initialization levels:**

| Level      | Command                              | When to use                              |
|------------|--------------------------------------|------------------------------------------|
| Minimal    | `dectl project init`                 | Quick prototypes                         |
| Standard   | `dectl project init --standard`      | **Recommended** for real projects        |
| Complete   | `dectl project init --full`           | Large projects with a defined architecture |

---

## Step 3: Configure Your Project

Edit the files created inside `.dec/`:

### 3.1 Configure `project.toml`

```bash
nano .dec/config/project.toml
```

**Example:**

```toml
[dec]
schema_version = "1.0"

[project]
name = "my-project"
type = "web-app"
description = "Brief description of your project"

[stack]
languages = ["typescript"]
frameworks = ["nextjs"]
databases = ["postgresql"]
tools = ["docker", "git"]
```

### 3.2 Define Vision in `project.isa.md`

```bash
nano .dec/isa/project.isa.md
```

**Example:**

```markdown
# Vision
Brief description of what the project does and for whom.

# Primary Goal
The most important problem the project solves.

# Technical Stack
- Frontend: ...
- Backend: ...
- DB: ...

# Conventions
- Commits in Spanish/English
- Coding conventions the team follows
```

---

## Step 4: Start Working

### View Project Status

```bash
dectl project info
```
Shows name, stack, vision, and warnings if anything is missing.

### View File Structure (respects `.gitignore`)

```bash
dectl project scan --depth 3
```
Shows a tree of files respecting `.gitignore`.

### Generate Context for AI (stateless)

```bash
dectl project context --max-tokens 3000 | pbcopy
```
Copies a compact summary ready to paste into ChatGPT/Claude.

---

## Using AI

### Flow 1 – AI with command access

```bash
# List available workflows

dectl workflow list

# Run a workflow

dectl workflow run implement_feature --var feature_name=user_auth

# Add a decision to memory

dectl memory add "Decision: use JWT for auth"
```

### Flow 2 – Stateless AI (no command access)

```bash
# 1️⃣ Generate context

dectl project context > context.txt

# 2️⃣ Paste into AI
# "Here is my project: [paste context]"
# I need to implement auth with JWT.

# 3️⃣ AI replies with a plan
# 4️⃣ You apply the changes manually
```

---

## Core Commands

### Project Management

```bash
# Create .dec/ folder

dectl project init [--standard|--full]

# Show status

dectl project info

# List files

dectl project scan [--depth N]

# Generate AI context

dectl project context [--max-tokens N] [--format text|json]
```

### Memory

```bash
# Add entry

dectl memory add "content"

# Add with tags

dectl memory add "content" --tags tag1,tag2

# List entries

dectl memory list

# Search

dectl memory search "keyword"

# Show specific entry

dectl memory show 1
```

### Workflows

```bash
# List workflows

dectl workflow list

# Describe a workflow

dectl workflow describe implement_feature

# Run a workflow with variables

dectl workflow run implement_feature --var name=value

# Dry run

dectl workflow run name --dry-run
```

### Protocol

```bash
# Execute commands from a file

dectl exec-from-file workflow.txt
```

### Global Flags

```bash
# JSON output

dectl --json

# Non‑interactive mode

dectl --non-interactive

# Help

dectl --help

# Version

dectl --version
```

---

## `.dec/` Directory Layout

```
.dec/
├── config/            # project.toml
├── isa/               # project.isa.md (vision)
├── workflows/         # YAML workflow definitions
├── prompts/
│   └── system/
│       ├── base.md    # Base AI instructions
│       └── integration.md
├── state/
│   ├── progress.json
│   └── last_session.md
├── decisions/         # Architectural decisions
├── knowledge/         # Glossary & constraints
├── .gitignore
└── README.md          # Context explanation
```

---

## Example Workflow

```bash
# Day 1 – Setup

git clone https://github.com/jhonesis/dectl.git
cd dectl

dectl project init --standard
nano .dec/config/project.toml
nano .dec/isa/project.isa.md

dectl project info
```

```bash
# Day 2 – First AI session

dectl project context > context.txt
# Paste into Claude/ChatGPT and ask for an auth plan
```

```bash
# Day 3 – Continue work
cat .dec/state/last_session.md

dectl workflow run implement_feature --var feature_name=inventory_crud --var module=src/inventory

dectl memory add "Decision: use PostgreSQL for inventory, not Redis"
```

---

## Commands That Work Without `.dec/`

- `dectl memory add "..."` — global memory
- `dectl --version`
- `dectl exec-from-file <path>`

## Commands Requiring `.dec/`

- `dectl project info`
- `dectl project scan`
- `dectl workflow list`

---

## Tips

1. **`.dec/` is not committed** – it’s excluded by a default `.gitignore`. Remove it from `.gitignore` if you want version control.
2. **Memory is global** – stored in `~/.dectl/memory.db`; works across all projects.
3. **Workflows are optional** – you can use dectl without them.
4. **The hidden `.dec/` folder never interferes with your code.**

---

*Version: 0.1.0* *Documentation: `/docs/user.md`*