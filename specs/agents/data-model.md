# Data Model — Agent System

> Defines agent schemas, agent_log table, and extensions to existing model.
> Last updated: 2026-06-02

---

## 1. agent_log Table (in memory.db)

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

**Migration**: `0003_agent_log` — creates table and indexes. Runs on first agent execution if not present.

**Relationship with existing tables**:
```
memories 1 ──── 0..* agent_log    (no FK, independent tables)
```

---

## 2. Custom Agent YAML Schema (`.dec/agents/*.yaml`)

```
name              string    required    Identifier. kebab-case.
role              string    required    Role description for the model. 1-2 sentences.
description       string    required    Shown in dectl agent list. Max 200 chars.
context_files     array     optional    Relative paths to project files the model reads
                                        before executing steps.
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

## 5. Relationships with Existing Data Models

```
.dec/agents/*.yaml
    │
    └── steps[] ─────────────────► same schema as .dec/workflows/*.yaml steps
                                     + new StepType "agent"

memory.db
    ├── memories table             (existing)
    ├── migrations table           (existing)
    └── agent_log table            (NEW — migration 0003)

session end
    └── Paso 5: agent_sync ───────► queries agent_log since last session
```
