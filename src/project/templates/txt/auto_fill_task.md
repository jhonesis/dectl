# Task: Auto-Fill Project Context

## Why this file exists
This project was initialized with `dectl project init`. The `.dec/` structure
is ready but some files contain placeholder values that need to be filled based
on the actual project code and documentation.

## Your task
Read this entire file. Then fill the missing context by analyzing the project:

### Step 1 — Detect the stack
Read the project source code, config files, and dependencies:
- Languages: detect from source files and config (Cargo.toml, package.json, go.mod, etc.)
- Frameworks: detect from imports, dependencies, and code patterns
- Databases: detect from ORM imports, migration files, connection strings
- Tools: detect from config files (Docker, CI/CD, linters, formatters, etc.)

### Step 2 — Analyze project context
Read documentation files for project intent:
- README.md → project name, description, what it does
- docs/ → architecture, design decisions, vision
- specs/ → requirements, acceptance criteria
- Other .md files → additional context

### Step 3 — Fill .dec/config/project.toml
Set:
- [project].description → one-sentence summary
- [stack].frameworks → detected from code (not just config files)
- [stack].databases → detected from imports/config
- [stack].tools → detected from config files

Do NOT modify [project].name, [project].type, [stack].languages,
[dec], or [conventions] — those are already set.

### Step 4 — Fill .dec/isa/project.isa.md
Complete:
- Vision → one sentence: what the project is and for whom
- Main Objective → what problem it solves and success metrics
- Scope → concrete list of what the project builds
- Non-Goals → what it explicitly does NOT do
- Tech Stack → main technologies (summary from project.toml)
- Known Constraints → technical, time, or resource limitations
- Main Risks → the 2-3 most important risks

### Step 5 — Log the initialization
```bash
dectl memory add "Project initialized: [name] — [one-line description]"
```

## What NOT to do
- Do NOT remove existing data from project.toml or project.isa.md
- Do NOT guess frameworks if unsure — leave as empty array or ask
- Do NOT modify files outside of .dec/ unless explicitly requested
- Do NOT run session end — the project is not ready for that yet

## Verification
After filling, run `dectl project info --json` to verify the setup is valid.
