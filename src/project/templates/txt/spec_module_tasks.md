# Tasks: [MODULE_NAME]
> *Atomic, ordered, trackable tasks for this module.*

## Legend
- `[Txxx]` = Task ID | `S/M/L` = Complexity
- **Build**: compile command | **Verify**: confirmation step | **Gate**: must pass before next task

---

## Phase 1: Foundation

**Build Gate**: `[build command]` — must pass with 0 errors

- [ ] [T001] [Setup] Implement core data structures for [MODULE_NAME] — S (REQ-[MODULE]-001)
  **Build**: `cargo build` passes without errors
  **Verify**: unit tests for core types pass
  **Gate**: must pass before T002

- [ ] [T002] [Core] Implement primary business logic — M (REQ-[MODULE]-001)
  **Build**: `cargo build` passes without errors
  **Verify**: `cargo test` passes for the module
  **Gate**: must pass before T003

- [ ] [T003] [Tests] Write unit + integration tests — M (REQ-[MODULE]-001)
  **Build**: `cargo build` passes without errors
  **Verify**: `cargo test` passes all tests
  **Gate**: must pass before phase gate

---

## Progress
- Total: 3
- Completed: 0
