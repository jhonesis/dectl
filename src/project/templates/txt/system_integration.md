# Session Instructions — [Project Name]
> **For the model**: Read and follow these instructions at the start of every working session.
> Update this file if the team wants to change the model's behavior.

---

## At session start

1. Read `.dec/config/project.toml` and `.dec/isa/project.isa.md` to understand the project
2. Read `.dec/state/last_session.md` and resume from "Recommended next step"
3. Run `dectl project info --json` and escalate to the developer if there are warnings
4. Confirm in 2-3 lines what you understood before asking what to do today

## Before acting

1. For architecture changes: read `.dec/decisions/` first
2. To implement a feature: look up its workflow in `.dec/workflows/`
3. For domain terms: consult `.dec/knowledge/glossary.md` if it exists
4. Describe what you are going to do before doing it — never act silently

## Available agents

Use `dectl agent list` to see all agents (built-in + custom).

The project includes these built-in agents:
- **coder**: implements code following the stack conventions
- **reviewer**: reviews code for bugs and deviations
- **researcher**: searches context in memory and prior decisions
- **documenter**: generates or updates technical documentation

To invoke an agent:
```
dectl agent run <type> --task "<task description>"
dectl agent describe <type>     # view full definition
dectl agent run --parallel <t1>,<t2> --task "<desc>"  # run in parallel
```

Use agents when the task is autonomous and specialized. The main model keeps the global context while the agent executes.

## When completing a task

1. If you completed or advanced a feature: update `.dec/state/progress.json`
2. For important decisions: run `dectl memory add "[decision summary]"`
3. For architectural decisions: create `.dec/decisions/XXXX-name.md`

## At session end

1. Run `dectl session end` to automate closing:
   - Generates `.dec/state/last_session.md` automatically
   - Syncs git changes to `progress.json`
   - Captures decisions and saves them to memory
   - Syncs stack changes with `project.toml`
   - Records agent activity
2. Or manually:
   - Write `.dec/state/last_session.md` (what was done, what's pending, decisions, next step)
   - Run `dectl memory add "Session [date]: [one-line summary]"`
