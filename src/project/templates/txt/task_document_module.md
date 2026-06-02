# Prompt: Document Module

## Context
You need to document an existing module of the project.

## Your task
1. Read the complete module code
2. Identify:
   - The module's main responsibility
   - Public functions/methods and their contracts
   - Dependencies and side effects
3. Write clear documentation:
   - README.md in the module folder or docs/
   - Doc comments (/// in Rust, docstrings in Python)
   - Usage examples where helpful

## Constraints
- Documentation must be useful for someone who didn't write the code
- Don't document the what (the code already says it), document the why and how
- Keep documentation close to the code (comments, docstrings)

## When done
- Record with `dectl memory add` what you documented
