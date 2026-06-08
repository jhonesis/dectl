# AGENTS.md — [PROJECT NAME]

> This project uses **dectl** (Dev Environment Control) with a structured `.dec/`
> directory that persists context, decisions, memory and workflows between sessions.
> Read this file and the `.dec/` directory completely before responding to any task.

---

## Session Cycle — Run at the start of every session

1. Read `.dec/config/project.toml` → project name, type, stack, conventions
2. Read `.dec/isa/project.isa.md` → vision, objectives and scope
3. If `.dec/isa/architecture.isa.md` exists → read before any architectural decision
4. Read `.dec/state/last_session.md` → resume from where you left off
5. If `.dec/decisions/` has files → read them before proposing structural changes
6. If `.dec/prompts/system/base.md` exists → read it for behavioral guidelines
7. Run `dectl project info --json` → verify schema compliance and project metadata

Do not skip these steps even for simple requests. Context is always required before acting.

---

## dectl Commands Reference

### Memory
```bash
dectl memory add "<text>" [--tags t1,t2]     # save a decision, note or fact
dectl memory list [--limit <n>]              # list all memories
dectl memory search "<query>"                 # search by keyword
dectl memory show <id>                        # show a specific entry
dectl memory delete <id> [--hard]             # soft-delete (or --hard for permanent)
dectl memory edit <id>                        # open entry in $EDITOR
```

### Project
```bash
dectl project init [--standard|--full]        # initialize .dec/ structure
dectl project info [--json]                   # show project metadata + warnings
dectl project scan [--depth <n>]              # file tree (gitignore-aware)
dectl project context [--max-tokens <n>]      # compact summary for stateless environments
```

### Workflows
```bash
dectl workflow list                           # list available workflows
dectl workflow describe <name>                # show workflow schema
dectl workflow run <name> [--var k=v] [--dry-run] [--from-step N]
```

### Protocol
```bash
dectl exec-from-file <path>                   # execute commands from a file
```

---

## When to Use dectl

| Situation | Command |
|-----------|---------|
| Architectural decision made | `dectl memory add "Decision: ..."` |
| Library or technology chosen | `dectl memory add "Stack: ..."` |
| Formal decision to record | create `.dec/decisions/XXXX-title.md` |
| Significant feature completed | `dectl memory add "Feature X done: ..."` |
| Run a structured process | `dectl workflow run <name>` |
| Need a compact project summary | `dectl project context` |

---

## Behavior Rules

- Read `.dec/` before acting, not after.
- Consult `.dec/decisions/` before proposing architecture changes.
- Follow `.dec/workflows/` as a thinking guide for complex tasks.
- After completing a significant task, update `.dec/state/progress.json`.
- At the end of every session, update `.dec/state/last_session.md`.

---

## Coding Discipline

> Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.
>
> **Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: every changed line should trace directly to the user's request.

### 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## Project Structure

```
.dec/
├── config/
│   └── project.toml          ← name, type, stack, conventions
├── isa/
│   ├── project.isa.md        ← vision, objectives, scope, non-goals
│   └── architecture.isa.md  ← modules, flows, trade-offs (if exists)
├── decisions/
│   └── *.md                  ← ADR-style decision records
├── workflows/
│   └── *.yaml                ← executable step-by-step processes
├── prompts/
│   ├── system/
│   │   └── base.md           ← behavioral guidelines (if exists)
│   └── tasks/
│       └── *.md              ← task-specific prompts (level 3)
├── knowledge/
│   ├── glossary.md           ← domain terms (if exists)
│   └── constraints.md        ← project constraints (if exists)
└── state/
    ├── progress.json         ← feature status tracking
    └── last_session.md      ← session continuity log
```

---

## If the Project Is New (First Session)

If `project.toml` has placeholder values (e.g. "project-name") or `project.isa.md`
has placeholder content (e.g. "[Project Name]"):

1. **Read `.dec/prompts/tasks/auto-fill.md`** for detailed fill instructions.
2. **Auto-detect the stack**: Read the project's source code, config files,
   dependencies, and imports to determine the full tech stack. Be thorough:
   languages, frameworks, databases, tools, testing frameworks, CI/CD.
3. **Analyze the project**: Read README.md, docs/, specs/, and any existing
   documentation to extract project name, description, vision, and objectives.
4. **Fill `.dec/config/project.toml`**: Update `[project].description`,
   `[stack].frameworks`, `[stack].databases`, `[stack].tools`. Never remove
   existing values — only add what is missing.
5. **Fill `.dec/isa/project.isa.md`**: Complete Vision, Main Objective, Scope,
   Non-Goals, Tech Stack, Known Constraints, and Main Risks.
6. **Log it**: Run `dectl memory add "Project initialized: [name] — [one-liner]"`.
7. **Update progress**: Set `updated_at` in `.dec/state/progress.json`.

Do NOT ask the user for what you can determine by reading the project code.
Only ask if something is genuinely ambiguous or requires human judgment.
