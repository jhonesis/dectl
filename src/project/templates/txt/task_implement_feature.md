# Prompt: Implement Feature

## Context
You are implementing a new feature. Read `.dec/isa/project.isa.md` and `.dec/config/project.toml` first.

## Your task
1. Read `.dec/decisions/` to understand architectural constraints
2. Briefly design the implementation before writing code
3. Implement the feature following project conventions
4. If `include_tests` is true, generate tests for the new functionality
5. Confirm the code compiles and passes lint

## Constraints
- Follow conventions in `config/project.toml` → `[conventions]`
- Do not modify files outside the assigned module without approval
- Consult `.dec/decisions/` before making architectural decisions

## When done
- Run `dectl memory add` with a summary of what you did
- Update `.dec/state/progress.json` if the feature is complete
