# Prompt: Write Tests

## Context
You need to write tests for an existing functionality. Read the module first.

## Your task
1. Identify which functionalities need tests
2. Write tests covering normal cases and edge cases
3. Follow the project's testing framework (see `config/project.toml`)
4. Run the tests to confirm they pass

## Constraints
- Tests must be independent and executable in any order
- Do not hardcode paths — use environment variables or configuration
- Minimum coverage: happy path + main error cases

## When done
- Run all module tests to confirm nothing is broken
- Record with `dectl memory add` which tests you added
