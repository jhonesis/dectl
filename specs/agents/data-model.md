# Data Model — Agent System

> Defines agent schemas, agent_log table, and extensions to existing model.
> Last updated: 2026-06-12

---

## 1. Agent Tables in memory.db

### 1.1 — agent_log

Execution log for all agent runs. Created in migration `0003_agent_log`.

```sql
CREATE TABLE IF NOT EXISTS agent_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    project TEXT,
    agent_type TEXT NOT NULL,
    task TEXT NOT NULL,
    status TEXT NOT NULL,           -- 'ok', 'error', 'timeout'
    steps_executed INTEGER DEFAULT 0,
    duration_ms INTEGER DEFAULT 0,
    error TEXT                      -- NULL if status = 'ok'
);

CREATE INDEX idx_agent_log_timestamp ON agent_log(timestamp);
CREATE INDEX idx_agent_log_type ON agent_log(agent_type);
```

### 1.2 — agent_outputs

Links successful agent executions to their auto-generated memory entries. Created in migration `0003_agent_outputs` (note: this is the same migration version as `agent_log`, both were created in the same phase).

```sql
CREATE TABLE IF NOT EXISTS agent_outputs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_type TEXT NOT NULL,
    task_id TEXT,
    task_description TEXT,
    output_file TEXT,
    memory_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE SET NULL
);

CREATE INDEX idx_agent_outputs_agent ON agent_outputs(agent_type);
CREATE INDEX idx_agent_outputs_task ON agent_outputs(task_id);
```

**Relationship with memories**:
```
memories 1 ──── 0..1 agent_outputs    (memory_id FK, ON DELETE SET NULL)
```

**Notes**:
- `memory_id` links to the memory summary auto-inserted when the agent completes successfully.
- `ON DELETE SET NULL` — if the memory entry is deleted, the link is preserved but nulled.
- `task_id` is an opaque identifier (e.g. "T119", "A020") provided by the caller.
- `output_file` is a path relative to `.dec/agent-output/<task_id>/`.

---

## 2. Custom Agent YAML Schema (`.dec/agents/*.yaml`)

```
name              string    required    Identifier. kebab-case.
role              string    required    Role description for the model. 1-2 sentences.
description       string    required    Shown in dectl agent list. Max 200 chars.
requires          array     optional    List of agent names that should run before this one.
                                        Informational only—does not block execution. Agents are
                                        fault-tolerant and run even if predecessors fail.
context_files     array     optional    Relative paths to project files to auto-load and expose
                                        as {{context_VARNAME}} variables in steps.
                                        Example: ".dec/config/project.toml" becomes
                                        "{{context_dec_config_project_toml}}". Files are loaded
                                        silently if readable; unreadable files are skipped.
inputs            array     optional    Same as workflows (InputDef).
steps             array     required    Same as workflows (Step).
                                        Valid types: prompt, action, write.
```

**Minimal valid example**:
```yaml
name: security-auditor
role: "Security auditor specialized in REST APIs"
description: Reviews code for security vulnerabilities
steps:
  - type: prompt
    description: Audit
    content: "Review {{task}} for common security vulnerabilities."
```

**Example with requires and context_files**:
```yaml
name: reviewer
role: "Code quality gate: build, test, lint, conventions"
description: Validates implementation against project rules
requires:
  - coder
context_files:
  - ".dec/config/project.toml"
  - ".dec/isa/project.isa.md"
steps:
  - type: action
    description: Read build configuration
    shell: true
    cmd: ["cat {{context_dec_config_project_toml}} | grep -A1 '\\[build\\]'"]
  - type: action
    description: Build project
    shell: true
    cmd: ["cargo build --release"]
```

**Note on fault-tolerance**: The `requires` field is informational only and does not block agent execution. All agents are designed to be fault-tolerant and will execute even if their predecessors fail. This ensures the workflow never stalls due to a single agent's errors.

---

## 3. StepType Extension in Workflows

The workflow schema from `dot-dec/data-model.md` is extended with a new type:

```
Step.type   "prompt" | "action" | "write" | "agent"   ← NEW in v2
```

**Additional fields for `type: agent`**:

```
agent_type    string          Required if parallel=false.
                              Agent name to execute. E.g. "reviewer"

agent_types   array<string>   Required if parallel=true.
                              List of agents to execute in parallel.

task          string          Required. Task description.
                              Supports interpolation {{variable}}.

parallel      boolean         Optional. Default: false.
                              true = executes agent_types in parallel.
```

**Workflow example**:
```yaml
- type: agent
  description: Review implemented code
  agent_type: reviewer
  task: "Review {{feature_name}} in src/{{module}}"
  parallel: false

- type: agent
  description: Review and document in parallel
  agent_types: [reviewer, documenter]
  task: "{{feature_name}} in src/{{module}}"
  parallel: true
```

---

## 4. JSON Output Shapes (new commands)

### `dectl agent list --json`
```json
{
  "status": "ok",
  "agents": [
    { "name": "coder", "role": "Feature implementer", "description": "...", "source": "builtin" },
    { "name": "security-auditor", "role": "Security auditor", "description": "...", "source": "custom" }
  ]
}
```

### `dectl agent run --json`
```json
{
  "status": "ok",
  "agent": "reviewer",
  "task": "src/auth/jwt.rs",
  "steps_executed": 2,
  "log_id": 42
}
```

### `dectl agent run --parallel --json`
```json
{
  "status": "ok",
  "results": [
    { "agent": "reviewer", "status": "ok", "log_id": 43 },
    { "agent": "documenter", "status": "ok", "log_id": 44 }
  ]
}
```

### `dectl agent describe <type> --json`
```json
{
  "status": "ok",
  "agent": {
    "name": "reviewer",
    "role": "Code quality reviewer",
    "description": "Reviews code for bugs, code smells, and convention violations",
    "source": "builtin",
    "inputs": [],
    "steps": [
      { "type": "prompt", "description": "Load review criteria", "content": "..." },
      { "type": "prompt", "description": "Review and report", "content": "..." }
    ]
  }
}
```

### `dectl agent trust --json`

```json
{
  "status": "ok",
  "data": {
    "agent": "coder",
    "project": "/home/user/projects/myapp",
    "already_trusted": false
  }
}
```

**Error**:
```json
{
  "status": "error",
  "message": "Agent 'nonexistent' not found",
  "hint": "Run 'dectl agent list' to see available agents"
}
```

---

### `dectl session end --json` (extended)
```json
{
  "status": "ok",
  "data": {
    "steps": [...],
    "decisions_saved": 2,
    "config_changes": {...},
    "agent_sessions": 3
  }
}
```

---

## 5. Data Flow: Auto-Link agent → memory

When an agent completes successfully (`status = 'ok'`), `runner.rs` automatically:

1. **Inserts a summary into `memories`** with dynamic type:
   - `'research'` if agent_type is `"researcher"`
   - `'note'` for all other agent types
2. **Inserts a link into `agent_outputs`** with the FK pointing to the new memory entry
3. **Inserts a log entry into `agent_log`** with execution metrics

This ensures every agent execution is traceable from both the memory system and the agent log.

```rust
// Pseudocode of runner.rs auto-link on completion:
let memory_id = insert_memory(
    conn,
    format!("Agent {}: {}", agent_type, task_description),
    type_, // "research" or "note"
    project,
)?;
insert_agent_output(conn, agent_type, task_id, task_description, &artifact, memory_id)?;
insert_agent_log(conn, agent_type, task, "ok", steps, duration)?;
```

---

## 6. Relationships with Existing Data Models

```
.dec/agents/*.yaml
    │
    └── steps[] ─────────────────► same schema as .dec/workflows/*.yaml steps
                                     + new StepType "agent"

memory.db
    ├── memories table             (existing, with new type column)
    ├── memories_fts table         (FTS5 — migration v2)
    ├── agent_log table            (migration 0003)
    ├── agent_outputs table        (migration 0003 — FK to memories.id)
    ├── tag_taxonomy table         (migration v4)
    └── migrations table           (existing)

session end
    └── Paso 5: agent_sync ───────► queries agent_log + agent_outputs since last session
```
