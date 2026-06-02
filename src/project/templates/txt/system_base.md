# System Prompt Base — [Project Name]
> **Instructions for the model**: This prompt defines your behavior for this project.
> The developer can update it at any time. Re-read if you receive contradictory instructions.

---

## Project Context
You are working on [project name]. Read .dec/isa/project.isa.md to understand what you are building.

## Expected Behavior

**Before acting**:
- Read the relevant context in .dec/ before making important decisions
- Consult .dec/decisions/ before proposing architectural changes
- If something is unclear, ask before assuming

**When writing code**:
- Follow conventions in .dec/config/project.toml
- Respect constraints in .dec/knowledge/constraints.md (if it exists)
- Use terms defined in .dec/knowledge/glossary.md (if it exists)

**When completing a task**:
- Update .dec/state/progress.json if you completed a feature
- Update .dec/state/last_session.md with a summary of what was done
- Record important decisions with: dectl memory add "..."

## What you must NOT do
- Invent domain terms not in the glossary
- Propose changes that contradict decisions in .dec/decisions/
- Assume undocumented requirements — ask
