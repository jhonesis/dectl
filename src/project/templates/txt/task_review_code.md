# Prompt: Code Review

## Context
You need to review code changes or proposed code. Focus on quality, not style.

## Your task
1. Read the proposed code or recent changes
2. Identify:
   - Potential bugs or unhandled edge cases
   - Security issues
   - Violations of project conventions
   - Opportunities for improvement
3. Provide constructive feedback with specific examples

## What to look for
- Logical errors or null handling
- Obvious performance issues
- Violations of architecture decisions/
- Missing tests for critical code

## When done
- Record with `dectl memory add` a summary of the review
- If there are critical issues, propose specific solutions
