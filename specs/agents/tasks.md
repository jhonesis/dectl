# Implementation Tasks — Agent System (v2)

> Atomic tasks for all agent features.
> Prefix A to distinguish from v1 tasks (T/D/C/I/S).
> Last updated: 2026-06-12

---

## Legend

- `[Axxx]` = Task ID
- `[P]` = Can run in parallel
- `S / M / L` = Complexity
- `(REQ-A-xxx)` = Traceability to spec

---

## Prerequisite

**v1 complete** — all tasks T, D, C, I, S, CS must be marked done before starting v2.

---

## Phase 1 — Individual Agents

- [x] [A001] Implement `agent/schema.rs`: structs `AgentDef`, `AgentSource`, `AgentResult`, `AgentRunStatus` per data-model.md — S (REQ-A-001, REQ-A-002) ✅
- [x] [A002] Implement `agent/loader.rs`: load built-in agents with `include_str!` from `builtins/`; scan `.dec/agents/*.yaml` for custom agents; custom takes priority over built-in of same name — M (REQ-A-001, REQ-A-004) ✅
- [x] [A003] Create built-in templates in English: `builtins/coder.yaml`, `builtins/reviewer.yaml`, `builtins/researcher.yaml`, `builtins/documenter.yaml` per plan.md — S (REQ-A-002) ✅
- [x] [A004][P] Implement `dectl agent list`: load all agents via loader; display table with name, role, description, and source; `--json` returns shape from data-model.md — S (REQ-A-001) ✅
- [x] [A005][P] Implement `dectl agent describe <type>`: load single agent, show role, description, inputs, steps; show file path for custom agents; `--json` returns full definition — S (REQ-A-008) ✅
- [x] [A006] Implement `agent/log.rs`: migration `0003_agent_log` creates table in memory.db; `record_agent_execution()` INSERTs row with timestamp, agent_type, task, status, steps_executed, duration_ms, error — M (REQ-A-006) ✅
- [x] [A007] Implement `agent/runner.rs`: load agent, resolve inputs, trust check, execute steps (reuse workflow runner), record in agent_log; support `--file` for additional context — L (REQ-A-002) ✅
- [x] [A008] Implement `dectl agent run <type> --task "<desc>"`: orchestrate loader → resolve inputs → runner → log; `--json` returns shape from data-model.md; `--timeout` support — M (REQ-A-002, REQ-A-009) ✅
- [x] [A009] Write integration tests for agents: `agent list` shows built-ins and custom, `agent run coder --dry-run` executes without side effects, custom agent from `.dec/agents/` takes priority, `agent describe` shows full definition — M ✅

**Phase 1 COMPLETE** ✅

---

## Phase 2 — Parallelism + Custom Agents

- [x] [A010] Implement `agent/parallel.rs`: launch agents in threads with `std::thread::spawn`; communicate results with `mpsc`; configurable timeout (default 5 min); each agent records in agent_log independently — L (REQ-A-003, REQ-A-009) ✅
- [x] [A011] Implement `dectl agent run --parallel <type1>,<type2>`: parse type list, invoke parallel runner, show consolidated summary; `status: "partial"` if any fails; `--timeout` applies to each agent — M (REQ-A-003) ✅
- [x] [A012] Extend `workflow/schema.rs`: add `Agent` variant to `StepType`; add fields `agent_type`, `agent_types`, `task`, `parallel` to `Step` — S (REQ-A-005) ✅
- [x] [A013] Extend `workflow/runner.rs`: handle agent-type step — invoke individual runner or parallel based on `parallel` flag — M (REQ-A-005) ✅
- [x] [A014][P] Write parallelism tests: two agents in parallel both complete, one agent fails and the other continues, final result is `"partial"`, timeout terminates hung agent — M ✅

**Phase 2 COMPLETE** ✅

---

## Phase 3 — Session End Integration + Polish

- [x] [A015] Add `agent_sessions` field to `SessionEndResult` in `session/types.rs` — XS (REQ-A-007) ✅
- [x] [A016] Implement `session/agent_sync.rs`: query agent_log for entries since last session timestamp; count executions per type; return count — S (REQ-A-007) ✅
- [x] [A017] Integrate agent_sync as Paso 5 in `session/end.rs`: independent step, does not block other steps; adds "agent_sync" to output; `--json` includes agent_sessions count — M (REQ-A-007) ✅
- [x] [A018] Update `CLAUDE.md`: add agent module to SDD index, update command reference with agent commands, update session end flow with Paso 5 — S ✅
- [x] [A019] Update specs: add agent SDD to master/plan.md Phase 2, add REQ-A references to cli/tasks.md — S ✅

**Phase 3 COMPLETE** ✅

---

## Phase 5 — Memory Auto-Link

*Auto-insert agent results into memories + agent_outputs tables.*

- [x] [A024] Crear tabla `agent_outputs` (migration v3 en memory.db) + auto-insert de resumen en memories al completar agente con `type='research'` para researcher, `type='note'` para otros — M ✅
- [x] [A025] Refactor `agent/log.rs` y `runner.rs` para usar `DbConn` compartido en vez de abrir conexión propia — S ✅

**Phase 5 COMPLETE** ✅

---

## Phase 4 — Agent Trust Command

- [x] [A020] Implementar `dectl agent trust <type>`: verificar que el agente existe, canonicalizar path, escribir en trust.toml — S (REQ-A-010) ✅
- [x] [A021] Canonicalización de paths en `runner.rs` y `workflow/trust.rs` — S ✅
- [x] [A022] Mejorar mensajes de error `--non-interactive`: sugerir `dectl agent trust <type> --project .` — XS ✅
- [x] [A023] Integration tests: trust built-in, trust nonexistent, trust invalid path, error message, trust-then-dry-run — M ✅

**Phase 4 COMPLETE** ✅

---

## Progress Tracking

| Phase | Total | Done | In Progress | Blocked |
|-------|-------|------|-------------|---------|
| Phase 1 — Individual agents | 9 | 9 | 0 | 0 |
| Phase 2 — Parallelism + Custom | 5 | 5 | 0 | 0 |
| Phase 3 — Session end + Polish | 5 | 5 | 0 | 0 |
| Phase 4 — Agent trust | 4 | 4 | 0 | 0 |
| Phase 5 — Memory Auto-Link | 2 | 2 | 0 | 0 |
| **Total** | **25** | **25** | **0** | **0** |
