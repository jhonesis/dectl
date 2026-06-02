# Specification — Agent System

> Technology-agnostic. Defines WHAT the agent system does.
> Version: 1.0 | Status: Final | Last updated: 2026-06-02

---

## Overview

The agent system extends dectl with specialized roles that the model can invoke for specific tasks. Agents can run individually, in sequence, or in parallel. They communicate via shared memory. All executions are recorded in the agent_log table in memory.db.

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

## Non-Functional Requirements

- **Latency**: agent startup overhead (without model execution) must not exceed 100ms
- **Resilience**: a parallel agent failure must not corrupt memory.db or other agents
- **Observability**: every agent execution is visible in agent_log with timestamp and duration
- **Thread safety**: agent_log writes use SQLite WAL mode for safe concurrent writes

---

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
