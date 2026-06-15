# Technical Implementation Plan — Agent System

> Describes HOW to implement the agent system. Technology-specific.
> Version: 1.1 | Status: Updated | Last updated: 2026-06-12

---

## References

- Implements: `specs/agents/spec.md`
- Constitution: `specs/agents/constitution.md`
- Research: `specs/agents/research.md`
- Extends: `specs/cli/plan.md` (new `agent` modules)
- Modifies: `specs/dot-dec/data-model.md` (new StepType `agent`)
- Modifies: `specs/cli/data-model.md` (new JSON shapes)
- Modifies: `specs/master/data-model.md` (agent_log table, agent_outputs table)
- Modifies: `specs/master/data-model.md` (memories_fts, tag_taxonomy, migration v2-v4)

---

## New Dependencies (Cargo.toml)

```toml
# No new dependencies — uses std::thread, mpsc, existing serde_yaml, rusqlite
```

---

## New Modules in `src/`

```
src/
├── agent/
│   ├── mod.rs
│   ├── schema.rs        ← AgentDef, AgentSource, AgentResult structs
│   ├── loader.rs        ← loads built-ins + custom from .dec/agents/
│   ├── runner.rs        ← executes individual agent
│   ├── parallel.rs      ← executes agents in parallel with threads
│   ├── log.rs           ← agent_log table migration + INSERT
│   ├── builtins/
│   │   ├── coder.yaml   ← embedded in binary with include_str!
│   │   ├── reviewer.yaml
│   │   ├── researcher.yaml
│   │   └── documenter.yaml
│   ├── list.rs          ← dectl agent list
│   ├── run.rs           ← dectl agent run
│   ├── describe.rs      ← dectl agent describe
│   └── trust.rs         ← dectl agent trust (trust registry)
```

---

## Main Structs

### Agent Module

```rust
// agent/schema.rs
#[derive(Deserialize, Serialize)]
pub struct AgentDef {
    pub name:          String,
    pub role:          String,
    pub description:   String,
    #[serde(default)]
    pub requires:      Vec<String>,     // informational, non-blocking
    #[serde(default)]
    pub context_files: Vec<String>,
    #[serde(default)]
    pub inputs:        Vec<InputDef>,   // reuses from workflow
    pub steps:         Vec<Step>,       // reuses from workflow
}

pub enum AgentSource {
    Builtin,
    Custom(PathBuf),
}

pub struct AgentResult {
    pub agent_type:    String,
    pub status:        AgentRunStatus,
    pub steps_executed: usize,
    pub log_id:        Option<i64>,
}

pub enum AgentRunStatus {
    Ok,
    Error { message: String },
    Timeout,
}
```

---

## Built-in Agent Templates

Built-in agents are embedded in the binary with `include_str!`. No disk files required. All templates in English.

### `builtins/coder.yaml`

```yaml
name: coder
role: "Feature implementer following project conventions"
description: Implements code respecting the stack and conventions from .dec/
steps:
  - type: prompt
    description: Load project context
    content: |
      You are a developer implementing features in this project.
      Read .dec/config/project.toml for stack and conventions.
      Read .dec/decisions/ to respect prior architectural decisions.
      Task: {{task}}

  - type: action
    description: Search for relevant context in memory
    cmd: ["dectl", "memory", "search", "{{task}}"]

  - type: prompt
    description: Implement
    content: |
      Implement the task respecting the project stack and conventions.
      When done run: dectl memory add "Implemented: {{task}}"
```

### `builtins/reviewer.yaml`

```yaml
name: reviewer
role: "Code quality reviewer"
description: Reviews code for bugs, code smells, and convention violations
steps:
  - type: prompt
    description: Load review criteria
    content: |
      You are a senior code reviewer.
      Read .dec/config/project.toml for expected conventions.
      File or module to review: {{task}}

  - type: prompt
    description: Review and report
    content: |
      Review {{task}} and report:
      - Potential bugs
      - Convention violations
      - Code smells
      - Improvement suggestions
      Be specific with lines and files.
      Run: dectl memory add "Review {{task}}: [findings summary]"
```

### `builtins/researcher.yaml`

```yaml
name: researcher
role: "Context and prior decisions investigator"
description: Searches memory and decisions for all context relevant to a task
steps:
  - type: action
    description: Search memory
    cmd: ["dectl", "memory", "search", "{{task}}"]

  - type: prompt
    description: Analyze relevant decisions
    content: |
      Read .dec/decisions/ and find all decisions relevant to: {{task}}
      Summarize context found in memory and decisions.
      Run: dectl memory add "Context researched for {{task}}: [summary]"
```

### `builtins/documenter.yaml`

```yaml
name: documenter
role: "Technical documentation generator"
description: Generates or updates documentation for the indicated module or feature
steps:
  - type: prompt
    description: Load documentation context
    content: |
      You are a technical writer.
      Read .dec/isa/project.isa.md to understand the project.
      Module or feature to document: {{task}}

  - type: prompt
    description: Generate documentation
    content: |
      Generate clear and concise documentation for {{task}}.
      Include: purpose, inputs/outputs, usage examples, dependencies.
      Update .dec/state/last_session.md with a summary of what was documented.
```

---

## `dectl agent run` Flow

```
dectl agent run reviewer --task "src/auth/jwt.rs"
    │
    ├── loader: searches "reviewer" in .dec/agents/ → not found
    │           searches built-ins → found
    │
    ├── resolve inputs: task = "src/auth/jwt.rs"
    │
    ├── trust check: has action steps? → yes
    │               trusted? → verify ~/.dectl/trust.toml
    │
    ├── runner: executes steps in order
    │     step 1 (prompt): prints interpolated content → pause for model
    │     step 2 (prompt): prints interpolated content → pause for model
    │
    ├── on success (status = Ok):
    │   │   INSERT INTO memories (content, type, project, ...)
    │   │     → type = "research" if agent_type == "researcher", else "note"
    │   │     → content = "Agent {agent_type}: {task_description}"
    │   │   INSERT INTO agent_outputs (agent_type, task_id, task_desc, output_file, memory_id)
    │   │
    ├── log: INSERT INTO agent_log (agent_type, task, status, steps_executed, duration_ms)
    │       (always written, regardless of success/failure)
    │
    └── output: {status:"ok", agent:"reviewer", task:"src/auth/jwt.rs",
                 steps_executed:2, log_id:15}
```

---

## `dectl agent run --parallel` Flow

```
dectl agent run --parallel reviewer,documenter --task "src/payments/"
    │
    ├── load both agents
    ├── trust check for both
    │
    ├── launch thread for reviewer
    │     → executes reviewer steps with task
    │     → sends result via mpsc channel
    │
    ├── launch thread for documenter
    │     → executes documenter steps with task
    │     → sends result via mpsc channel
    │
    ├── main thread waits for both results (rx.recv() × 2)
    │
    ├── log: two entries in agent_log
    │
    └── consolidated output:
        {status:"ok", results:[
          {agent:"reviewer", status:"ok", log_id:16},
          {agent:"documenter", status:"ok", log_id:17}
        ]}
```

---

## Session End Integration (Paso 5)

```
dectl session end
  ├─ Paso 1: session_summary → last_session.md
  ├─ Paso 2: git_sync → progress.json
  ├─ Paso 3: decision_capture → memory.db
  ├─ Paso 4: config_sync → project.toml + isa.md coherence
  └─ Paso 5: agent_sync
       ├─ Query agent_log for entries since last session timestamp
       ├─ Count agent executions per type
       └─ Report: "3 agent sessions this cycle (2 reviewer, 1 coder)"
```

---

## Implementation Phases

### Phase 1 — Agent individual
Built-in agents + `dectl agent list` + `dectl agent run` individual + trust + agent_log.

### Phase 2 — Agent parallel + custom
`parallel.rs` + custom agents from `.dec/agents/` + agent step in workflows.

### Phase 3 — Session end integration + describe
`agent_sync` in session end + `dectl agent describe` + timeout support.

### Phase 4 — Agent trust command (post-v1)
`dectl agent trust <type>` — trust agent for a project without running it. Path canonicalization, improved `--non-interactive` error messages.

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Agent thread hangs in parallel | High | Configurable timeout per agent (default 5 min) |
| SQLite concurrent writes from parallel agents | Medium | WAL mode already configured; test concurrent INSERTs |
| Custom agent YAML schema mismatch | Low | Graceful error with clear message; skip invalid agents |
