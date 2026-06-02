# Constitution — Agent System

Principles that govern the design and implementation of the dectl agent system.
Inherits and extends: specs/master/constitution.md
Last updated: 2026-05-21

---

## 1. Identity

An agent in dectl is a specialized workflow with a defined role. It is not a language model — it is a behavior and context template that prepares the model to execute a specific type of task consistently and repeatably.

The agent system extends workflows with three new capabilities: role specialization, parallel execution, and inter-agent communication via shared memory.

---

## 2. Fundamental Principles

1. **One agent = one role = one responsibility**
Each agent has a single purpose. coder implements. reviewer reviews. researcher investigates. documenter documents. An agent that does two things is a poorly designed agent.

2. **Agents do not contain models**
Like the CLI, an agent does not run a language model. It prepares context, runs dectl commands, and coordinates steps. The model invokes it from its environment.

3. **Inter-agent communication is asynchronous via memory**
Agents do not call each other directly. They communicate by writing to and reading from shared project memory (dectl memory). This eliminates coupling and makes the system robust against partial failures.

4. **Parallelism is opt-in and explicit**
A workflow launches agents in parallel only when the developer explicitly declares it. Sequential behavior is the default. Never implicit parallelism.

5. **Every agent is auditable**
Everything an agent executes is recorded in the agent_log table in memory.db. The developer can see exactly what each agent did, in what order, and with what result.

6. **Agents are extensible by the community**
Any developer can define a new agent in .dec/agents/ with a YAML file. No CLI modification or code changes required.

---

## 3. Agent Types

### Built-in agents (included in dectl)

| Type | Role | Typical Task |
|------|------|-------------|
| coder | Implements features | Write code following project conventions |
| reviewer | Reviews code | Verify quality, conventions, and potential bugs |
| researcher | Investigates | Search memory, decisions, and relevant context |
| documenter | Documents | Generate or update project documentation |

### Custom agents (defined in .dec/agents/)

Any agent defined by the developer in `.dec/agents/<name>.yaml`.

---

## 4. Implementation Rules

- Built-in agents are immutable templates in the binary
- Custom agents in `.dec/agents/` take priority over built-ins of the same name
- A parallel agent runs in a separate thread — never shares mutable state with other agents
- Memory is the only communication channel between agents
- If a parallel agent fails, others continue — the orchestrator reports the failure at the end
- Agents respect the trust system like workflows (action steps require confirmation)

---

## 5. Definition of Done for Agents

An agent is complete when:

- [ ] The YAML template is defined with clear role, description, and steps
- [ ] `dectl agent list` displays it correctly
- [ ] `dectl agent run <type>` works with --task and --json
- [ ] The output is recorded in the agent_log table in memory.db
- [ ] Integration test passes invoking the real binary
