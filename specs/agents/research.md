# Technical Research — Agent System

> Investigates technical decisions specific to the agent system.
> Last updated: 2026-05-21

---

## Context

The agent system introduces three new capabilities not in v1: role specialization, parallelism, and inter-agent communication. Research questions cover how to implement these in Rust without unnecessary complexity.

---

## RQ-A-001: Parallelism mechanism in Rust

**Context**: REQ-A-003 requires executing multiple agents in parallel. In Rust there are several options for concurrency — the choice affects complexity, safety, and compatibility with the rest of the CLI which is synchronous.

**Options Evaluated**:

| Option | Pros | Cons |
|--------|------|------|
| `std::thread::spawn` | Pure stdlib, no deps, simple | Requires `Arc<Mutex<>>` for shared state; manual join handling |
| `rayon` | Idiomatic data parallelism, thread pool | Designed for compute, not concurrent I/O |
| `tokio` (async) | Maximum control, rich ecosystem | Introduces async runtime to entire CLI — major architectural change |
| `std::thread` + `mpsc channels` | Clean inter-thread communication, pure stdlib | More verbose than alternatives |

**Decision**: `std::thread::spawn` + `std::sync::mpsc` channels for result communication.

**Rationale**: The CLI is synchronous by design. Introducing `tokio` only for agent parallelism would be disproportionate and affect the entire architecture. `std::thread` with channels is sufficient — agents are independent, do not share mutable state (they communicate via SQLite memory), and the maximum number of parallel agents is small (2-5 typically). The stdlib solution is easier to audit and maintain.

**Implementation pattern**:
```rust
let (tx, rx) = std::sync::mpsc::channel();

for agent_type in agent_types {
    let tx = tx.clone();
    let task = task.clone();
    std::thread::spawn(move || {
        let result = run_agent(agent_type, &task);
        tx.send((agent_type, result)).unwrap();
    });
}

let mut results = vec![];
for _ in 0..agent_types.len() {
    results.push(rx.recv().unwrap());
}
```

---

## RQ-A-002: YAML schema for custom agents

**Context**: REQ-A-004 requires developers to define agents in `.dec/agents/<name>.yaml`. The schema must be familiar (consistent with workflows), extensible, and CLI-validatable.

**Options Evaluated**:

| Option | Pros | Cons |
|--------|------|------|
| Identical schema to workflows | Zero learning curve, reuses parser | Mixes concepts — an agent is not a workflow |
| Own schema with no relation | Maximum flexibility | New learning curve, new parser |
| Superset of workflow schema | Familiar + extra agent fields | Requires discriminating in the parser |

**Decision**: Superset of workflow schema with additional role and context fields.

**Rationale**: A custom agent is still a sequence of steps — the difference is it has a `role` and `context_files` that prepare the model before execution. Reusing the workflow parser with additional optional fields is the lowest friction option for the developer and least new code in the CLI.

**Custom agent schema**:
```yaml
name: security-auditor
role: "Security auditor specialized in REST APIs"
description: Reviews code for security vulnerabilities
context_files:
  - .dec/knowledge/constraints.md
  - src/auth/

inputs:
  - name: target_file
    description: File or module to audit
    required: true

steps:
  - type: prompt
    description: Load security context
    content: |
      You are a security auditor. Read {{target_file}} and look for:
      injection vulnerabilities, weak authentication, data exposure.

  - type: action
    description: Search for previous security decisions
    cmd: ["dectl", "memory", "search", "security"]

  - type: prompt
    description: Report findings
    content: |
      Report findings with severity (critical/high/medium/low) and fix suggestion.
      Run: dectl memory add "Audit {{target_file}}: [summary]"
```

---

## RQ-A-003: New StepType `agent` in workflows

**Context**: REQ-A-005 requires workflows to invoke agents as steps. This means extending the `Step` schema in dot-dec and the runner in the CLI.

**Options Evaluated**:

| Option | Pros | Cons |
|--------|------|------|
| New StepType `agent` in enum | Clean, typed, consistent with architecture | Requires updating dot-dec/data-model.md |
| Use `action` step with `cmd: ["dectl", "agent", "run"]` | No schema change | Loses semantics — runner does not know it is an agent |
| Generic `workflow` step that calls any workflow/agent | More flexible | Too generic, hard to validate |

**Decision**: New StepType `agent` with fields `agent_type` and `parallel`.

**Rationale**: Semantics matter — the runner needs to know it is executing an agent to be able to launch it in parallel if `parallel: true`. Using `action` with `dectl agent run` would work but loses declarative parallelism capability. The schema change is minimal and backwards compatible (new optional type).

**Agent step schema**:
```yaml
- type: agent
  description: Review generated code
  agent_type: reviewer
  task: "Review {{feature_name}} for code smells"
  parallel: false
  # For parallel:
  # agent_types: [reviewer, documenter]
  # parallel: true
```

---

## RQ-A-004: Agent audit log in memory.db

**Context**: REQ-A-006 requires every agent execution to be recorded. Options: separate log.db or add table to existing memory.db.

**Options Evaluated**:

| Option | Pros | Cons |
|--------|------|------|
| New `log.db` file | Separation of concerns | New DB file, new migration system, more complexity |
| `agent_log` table in `memory.db` | Reuses existing SQLite infrastructure, single DB file, WAL mode already configured | Slightly larger memory.db |
| File-based log (JSON lines) | Simple, no DB dependency | Harder to query, no atomicity |

**Decision**: `agent_log` table in existing memory.db.

**Rationale**: The memory.db already has auto-migrations on startup, WAL mode configured, and a proven migration pattern. Adding an `agent_log` table is a simple migration (0003_agent_log) and keeps all persistence in one place. The table is independent from the `memories` table — no foreign key relationship needed.

**Table schema**:
```sql
CREATE TABLE IF NOT EXISTS agent_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    project TEXT,
    agent_type TEXT NOT NULL,
    task TEXT NOT NULL,
    status TEXT NOT NULL,
    steps_executed INTEGER DEFAULT 0,
    duration_ms INTEGER DEFAULT 0,
    error TEXT
);
```

---

## Decision Summary

| ID | Decision | Result |
|----|----------|--------|
| RQ-A-001 | Parallelism | `std::thread` + `mpsc` channels |
| RQ-A-002 | Custom agent schema | Superset of workflow YAML |
| RQ-A-003 | StepType agent | New type with `agent_type` and `parallel` |
| RQ-A-004 | Audit log | `agent_log` table in existing memory.db |
