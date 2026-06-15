# Specification — Agent System

> Technology-agnostic. Defines WHAT the agent system does.
> Version: 1.1 | Status: Updated | Last updated: 2026-06-12

---

## Overview

The agent system extends dectl with specialized roles that the model can invoke for specific tasks. Agents can run individually, in sequence, or in parallel. They communicate via shared memory. All executions are recorded in the agent_log table in memory.db. On successful completion, agents automatically persist their results into the memory system (auto-link), creating a traceable connection between agent outputs and memory entries via the agent_outputs table.

---

## Users

- **Developer**: invokes agents for specific project tasks
- **Model**: generates `dectl agent run` commands to delegate specialized tasks
- **Workflow**: a workflow can launch agents as `agent`-type steps

---

## Functional Requirements

### REQ-A-001: List available agents

**User Story**:
> As a developer or model, I want to see all available agents to know which roles I can invoke.

**Acceptance Criteria**:
- WHEN `dectl agent list` is executed THEN SHALL show built-in and custom agents from .dec/agents/ with name, role, and description
- WHEN a custom agent has the same name as a built-in THEN the custom SHALL take priority
- WHEN `--json` is used THEN SHALL return `{status:"ok", agents:[{name, role, description, source:"builtin"|"custom"}]}`

---

### REQ-A-002: Execute individual agent

**User Story**:
> As a developer or model, I want to execute a specialized agent for a concrete task without defining context from scratch.

**Acceptance Criteria**:
- WHEN `dectl agent run <type> --task "<description>"` is executed THEN SHALL load the agent template, inject the task, and prepare context for the model
- WHEN the agent has action steps THEN SHALL apply the trust system like workflows
- WHEN the agent completes THEN SHALL record the execution in the agent_log table with type, task, duration, and result
- WHEN `--file <path>` is used THEN SHALL use the indicated file as additional context for the agent
- WHEN `--json` is used THEN SHALL return `{status:"ok"|"error", agent, task, steps_executed, log_id}`

---

### REQ-A-003: Execute agents in parallel

**User Story**:
> As a developer or model, I want to execute multiple agents in parallel to speed up tasks that are independent of each other.

**Acceptance Criteria**:
- WHEN `dectl agent run --parallel <type1>,<type2> --task "<description>"` is executed THEN SHALL launch each agent in a separate thread with the same task
- WHEN all agents complete THEN SHALL show consolidated summary of results per agent
- WHEN an agent fails in parallel THEN SHALL record the failure and continue with the rest — do not abort all
- WHEN `--json` is used THEN SHALL return `{status:"ok"|"partial"|"error", results:[{agent, status, log_id}]}`

---

### REQ-A-004: Custom agents in .dec/agents/

**User Story**:
> As a developer, I want to define my own agents for my project without modifying the CLI to adapt roles to my stack and conventions.

**Acceptance Criteria**:
- WHEN .dec/agents/<name>.yaml exists THEN `dectl agent list` SHALL include it with source: "custom"
- WHEN the file schema is invalid THEN `dectl agent list` SHALL warn without aborting
- WHEN a custom agent defines steps THEN SHALL support the same types as workflows (prompt, action, write)
- WHEN a custom agent defines inputs THEN SHALL support variables with `--var name=value` like workflows

---

### REQ-A-005: Agents in workflows

**User Story**:
> As a developer, I want to invoke agents from within a workflow to compose complex tasks with specialized roles.

**Acceptance Criteria**:
- WHEN a workflow has an agent-type step THEN SHALL execute the indicated agent with the step's parameters
- WHEN the agent completes successfully THEN the workflow SHALL continue with the next step
- WHEN the agent fails THEN the workflow SHALL report the failure with the same mechanism as a failed action step
- WHEN the agent step has parallel: true and multiple types THEN SHALL execute them in parallel before continuing

---

### REQ-A-006: Agent execution audit (agent_log)

**User Story**:
> As a developer, I want every agent execution to be recorded in memory.db so I can audit what agents did and when.

**Acceptance Criteria**:
- WHEN any agent completes or fails THEN SHALL INSERT a row in the agent_log table in memory.db
- WHEN the agent_log table does not exist THEN SHALL auto-create it on first agent execution (migration)
- WHEN `dectl agent list --json` is used THEN SHALL NOT include log data (only agent definitions)

---

### REQ-A-007: Session end integration

**User Story**:
> As a developer, I want `dectl session end` to report agent activity from the current session so I know which agents were used.

**Acceptance Criteria**:
- WHEN `dectl session end` is executed THEN SHALL query agent_log for entries since the last session
- WHEN agent sessions are found THEN SHALL add a step "agent_sync" to the session end output
- WHEN no agent sessions are found THEN SHALL skip the agent_sync step silently
- WHEN `--json` is used THEN SHALL include `agent_sessions` count in the output

---

### REQ-A-008: Agent describe

**User Story**:
> As a developer or model, I want to see the full definition of an agent (role, steps, inputs) before running it.

**Acceptance Criteria**:
- WHEN `dectl agent describe <type>` is executed THEN SHALL show the agent's role, description, inputs, and steps
- WHEN the agent is custom THEN SHALL show the file path where it is defined
- WHEN `--json` is used THEN SHALL return the full agent definition as JSON

---


### REQ-A-009: Agent timeout

**User Story**:
> As a developer, I want agents to have a configurable timeout so a hung agent does not block the session indefinitely.

**Acceptance Criteria**:
- WHEN an agent runs longer than the timeout (default 5 min) THEN SHALL terminate the agent thread and report timeout error
- WHEN `--timeout <seconds>` is used THEN SHALL override the default timeout
- WHEN a parallel agent times out THEN SHALL record the timeout and continue with other agents

---

## Fault-Tolerance & Non-Blocking Design

### REQ-A-011: Agents are non-blocking

**Design Principle**:
Agents are designed to be fault-tolerant and non-blocking. No agent failure stops the workflow or prevents other agents from running. This ensures task completion even when individual agents encounter errors.

**Acceptance Criteria**:
- WHEN an agent fails (action command returns non-zero, file write fails, timeout) THEN the agent logs the error but the workflow continues
- WHEN `requires: [coder]` is declared on an agent THEN this is informational only and does NOT block execution
- WHEN a workflow runs agents in sequence THEN a failed agent does not prevent subsequent agents from starting
- WHEN a parallel agent fails THEN remaining agents continue in parallel and results are aggregated
- WHEN documenter or similar agents have `run_always: true` THEN they execute even if all predecessors failed

**Examples**:
```yaml
# Coder runs even if researcher fails
requires: [researcher]  # Only informs the model; does not enforce

# Documenter always records the task
- type: agent
  agent_type: documenter
  task: "Document {{task_id}}"
  run_always: true  # Runs even if review failed
```

---

### REQ-A-012: Context files are auto-loaded

**User Story**:
> As a developer, I want agents to have automatic access to project config and state files without manual shell commands.

**Acceptance Criteria**:
- WHEN an agent declares `context_files: [".dec/config/project.toml"]` THEN the file content is auto-loaded before executing steps
- WHEN a context file is loaded THEN it is exposed as a variable `{{context_NORMALIZED_PATH}}` where path is normalized (dots/slashes→underscores, lowercase)
- WHEN a context file is unreadable or missing THEN it is silently skipped (no error)
- WHEN multiple context files are declared THEN all are loaded into separate variables
- WHEN a step uses `{{context_dec_config_project_toml}}` THEN the content of `.dec/config/project.toml` is substituted

**Examples**:
```yaml
context_files:
  - ".dec/config/project.toml"
  - ".dec/isa/project.isa.md"
  - ".dec/state/progress.json"

steps:
  - type: action
    description: Show build command
    shell: true
    cmd: ["echo '{{context_dec_config_project_toml}}' | grep '\\[build\\]'"]

  - type: prompt
    description: Review implementation
    content: |
      Based on the project ISA ({{context_dec_isa_project_isa_md}}),
      review the following code changes:
      {{step_previous_output}}
```

---

### REQ-A-013: Auto-link agent results to memory

**User Story**:
> As a developer, I want agent results to be automatically persisted in the memory system so that the outputs of agent executions remain available for future sessions without manual intervention.

**Acceptance Criteria**:
- WHEN an agent completes successfully (`status = 'ok'`) THEN SHALL auto-insert a summary into the `memories` table with:
  - `type = 'research'` if agent_type is `"researcher"`
  - `type = 'note'` for all other agent types
- WHEN an agent completes successfully THEN SHALL auto-insert a link into the `agent_outputs` table with agent_type, task_id, task_description, output_file path, and memory_id FK
- WHEN the agent fails THEN SHALL NOT insert into memories or agent_outputs (only agent_log is written)
- WHEN the agent_log is written THEN it SHALL always be written regardless of success or failure (logging is unconditional)
- WHEN `--dry-run` is used THEN SHALL NOT insert into any table (no side effects)
- WHEN the auto-insert happens THEN the memory summary SHALL use the format: `"Agent {agent_type}: {task_description}"`

**Referencia**: Ver `specs/agents/data-model.md` para el schema de `agent_outputs`.

---

### REQ-A-010: Agent trust command

**User Story**:
> As a developer, I want to trust an agent without running it so that subsequent `dectl agent run` commands skip the interactive prompt.

**Acceptance Criteria**:
- WHEN `dectl agent trust <type>` is executed THEN SHALL verify the agent exists (built-in or custom)
- WHEN `--project <path>` is used THEN SHALL trust for the specified project (default: current directory)
- WHEN the agent is already trusted THEN SHALL report "already trusted" (idempotent)
- WHEN completed THEN SHALL write to `~/.dectl/trust.toml` with canonical path and timestamp
- WHEN `--json` is used THEN SHALL return `{status:"ok", agent, project, already_trusted}`
- WHEN the agent does not exist THEN SHALL error with clear message
- WHEN `--non-interactive` causes trust to be needed in `agent run` THEN the error message SHALL suggest `dectl agent trust <type> --project .`

---

## Out of Scope

- Direct agent-to-agent communication (only via memory)
- Agent state persistence between runs (stateless by design)
- Model-specific agent implementations (agent is model-agnostic)
