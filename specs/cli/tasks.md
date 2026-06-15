# Implementation Tasks — dectl CLI
> *Tareas atómicas derivadas de plan.md y data-model.md del SDD del CLI.*
> *Prefijo C para distinguir de tareas del maestro (T) y de .dec/ (D).*
> *Last updated: 2026-06-12*

---

## Leyenda

- `[Cxxx]` = Task ID
- `[P]` = Puede correr en paralelo con otras `[P]` en la misma fase
- `S / M / L` = Complejidad estimada
- `(REQ-C-xxx)` = Trazabilidad al spec del CLI
- `→ reemplaza Txxx` = Reemplaza o refina una tarea del maestro

---

## Dependencias con Otros SDDs

```
Master T001–T007  (setup, core básico)
    ↓
dot-dec D001–D015 (structs de .dec/ + templates nivel 1 y 2)
    ↓
C001–C040         (implementación completa del CLI)
    ↓
dot-dec D016–D025 (templates nivel 3 + motor de interpolación)
    ↓
C041–C050         (workflow runner con variables + --from-step)
```

---

## Fase 1 — Infraestructura Base del CLI

### Sistema de Output

- [x] [C001] Implementar `core/output.rs`: struct `Output` con `json_mode` y `color`; métodos `success`, `error`, `info`, `step`; detección automática de TTY con `is-terminal`; respeto de variable `NO_COLOR` — M (REQ-C-012)
- [x] [C002] Implementar envelope JSON `{"status":"ok"|"error",...}` usando `serde_json`; método `Output::success` serializa cualquier `T: Serialize`; método `Output::error` siempre va a stderr — S (REQ-C-012)
- [x] [C003] Implementar `core/markdown.rs`: función `to_plain(input: &str) -> String` que elimina headers, bold, italic y backticks de inline code — S (REQ-C-007)

---

### Configuración

- [x] [C004] Implementar `core/config.rs`: deserializar `GlobalConfig` desde `~/.dectl/config.toml` con `serde` + `toml`; crear archivo con defaults si no existe; exponer función `load_global() -> Result<GlobalConfig>` — M (REQ-C-012)
- [x] [C005][P] Implementar `core/config.rs` carga de `ProjectConfig` desde `.dec/config/project.toml`; validar `schema_version`, `project.name`, `project.type`, `stack.languages`; retornar warnings en lugar de errores para campos no críticos — M (REQ-C-002)
- [x] [C006][P] Implementar detección automática de proyecto: buscar `.dec/config/project.toml` en el directorio actual y padres hasta el home; retornar `Option<ProjectConfig>` — S (REQ-C-004)

---

### Stdin y Señales

- [x] [C007] Implementar `core/stdin.rs`: función `read_content(arg: Option<String>) -> Result<String>`; leer de argumento, pipe stdin, o error si ninguno — según patrón en plan.md — S (REQ-C-004)
- [x] [C008] Implementar handler SIGINT con `ctrlc`: limpiar línea de progreso en stderr, imprimir "Interrupted.", exit code 130 — S (constitution)

---

### Exit Codes

- [x] [C009] Implementar enum `ExitCode` en `core/error.rs` con variantes `Success(0)`, `UserError(1)`, `SystemError(2)`, `StateError(3)`, `Interrupted(130)`; implementar función `classify_error(e: &anyhow::Error) -> ExitCode` — S (constitution)
- [x] [C010] Configurar `main.rs`: setup ctrlc → parse CLI → ejecutar comando → output resultado o error → exit con código correcto — M

---

## Fase 2 — Comandos de Proyecto

- [x] [C011] Implementar `dectl project init`: lógica de creación de niveles 1/2/3 usando templates de D005–D020; abortar si `.dec/` existe; mostrar archivos creados y próximo paso; `--json` retorna shape de plan.md — M (REQ-C-001)
  > → reemplaza T009 del maestro con implementación real basada en templates del SDD de .dec/

- [x] [C012][P] Implementar `dectl project info`: cargar `ProjectConfig` + leer primera sección de `project.isa.md` + cargar `progress.json`; mostrar resumen con warnings por archivos faltantes; validar `schema_version`; `--json` retorna shape de data-model.md — M (REQ-C-002)
  > → reemplaza T010 del maestro

- [x] [C013][P] Implementar `dectl project scan`: usar `ignore` crate para recorrer árbol respetando `.gitignore`; soportar `--depth <n>`; `--json` retorna árbol como array de nodos; excluir `.git/` y `target/` si no hay `.gitignore` — M (REQ-C-003)
  > → reemplaza T011 del maestro

- [x] [C014] Escribir integration tests para comandos de proyecto: `init` crea estructura correcta por nivel, `init` aborta si `.dec/` existe, `info` maneja archivos faltantes con warnings, `scan` respeta `.gitignore`, todos soportan `--json` válido — M

---

## Fase 3 — Comandos de Memoria

- [x] [C015] Implementar `memory/db.rs`: abrir/crear `~/.dectl/memory.db`; habilitar WAL mode; ejecutar migraciones pendientes al abrir; exponer `DbConn` — M
  > → reemplaza T013 del maestro

- [x] [C016] Implementar migración `0001_initial`: crear tabla `memories` con índices según master/data-model.md; tabla `migrations` para tracking de versiones — S
  > → reemplaza T014 del maestro

- [x] [C017][P] Implementar `dectl memory add`: usar `core/stdin.rs` para leer contenido; auto-detectar proyecto con C006; INSERT en SQLite; `--json` retorna shape de data-model.md — M (REQ-C-004)
  > → reemplaza T015 del maestro

- [x] [C018][P] Implementar `dectl memory list`: SELECT con ORDER BY `created_at DESC`; filtro `--project`; límite configurable `--limit` con default desde `GlobalConfig.memory.max_results`; `--json` retorna shape de data-model.md — M (REQ-C-005)
  > → reemplaza T016 del maestro

- [x] [C019][P] Implementar `dectl memory search`: LIKE case-insensitive en `content` y `tags`; filtro `--project`; mostrar fragmento con coincidencia resaltada en modo human; `--json` incluye `query` en envelope — M (REQ-C-006)
  > → reemplaza T017 del maestro

- [x] [C020][P] Implementar `dectl memory show`: SELECT por ID; renderizar content con `core/markdown.rs` en modo human, raw en modo JSON; error claro si ID no existe; `--json` retorna entry completo — S (REQ-C-007)
  > → reemplaza T018 del maestro

- [x] [C021] Escribir unit tests de memoria contra SQLite in-memory: add con y sin tags, add desde stdin simulado, list con filtros, search con coincidencia y sin resultados, show con ID válido e inválido — M
  > → reemplaza T019 del maestro

---

## Fase 4 — Protocolo

- [x] [C022] Implementar `dectl exec-from-file <path>`: leer archivo línea a línea; ignorar líneas vacías y comentarios (`#`); ejecutar cada línea como subcomando `dectl`; parar y reportar en fallo con número de línea; `--json` retorna shape de data-model.md — M (REQ-C-011)
  > → reemplaza T020 del maestro

- [x] [C023] Escribir integration test de `exec-from-file`: archivo con comandos válidos, archivo con comentarios, archivo con comando inválido en línea N verifica que para en N — S

---

## Fase 5 — Comandos de Workflow

*Depende de: D001–D025 (structs + templates + motor de interpolación)*

- [x] [C024] Implementar `workflow/schema.rs`: structs `Workflow`, `InputDef`, `Step`, `StepType` según data-model.md de este SDD — S
  > → reemplaza D004 / T025 del maestro con implementación completa incluyendo `inputs`

- [x] [C025] Implementar `workflow/loader.rs`: leer `.dec/workflows/<name>.yaml`; parsear con `serde_yaml`; validar que variables referenciadas en steps estén declaradas en `inputs`; error descriptivo por campo faltante — M (REQ-C-009)
  > → reemplaza T026 del maestro

- [x] [C026][P] Implementar `workflow/trust.rs`: leer/escribir `~/.dectl/trust.toml`; comprobar si `<project_path>/<workflow_name>` es trusted; si no y hay `action` steps: mostrar comandos + pedir Y/n + guardar; si `--non-interactive` y no trusted: error — M (REQ-C-010)
  > → reemplaza T027 del maestro

- [x] [C027] Implementar motor de interpolación en `workflow/runner.rs`: función `interpolate(template: &str, vars: &HashMap<String,String>) -> Result<String>`; soporte de escape `\{{`; error si variable no declarada — M
  > → reemplaza D021

- [x] [C028][P] Implementar resolución de inputs en `workflow/runner.rs`: extraer vars de flags `--var nombre=valor`; verificar obligatorias presentes; aplicar defaults para opcionales; error antes del primer paso si faltan requeridas — M (REQ-C-010)
  > → reemplaza D023

- [x] [C029] Implementar ejecución de steps en `workflow/runner.rs`: iterar steps con `--from-step` skip; ejecutar según tipo (prompt/action/write); manejar `--dry-run`; capturar output de `action` en tiempo real; escribir archivo en `write`; parar y reportar en fallo con hint de `--from-step N` — L (REQ-C-010)
  > → reemplaza T028 del maestro

- [x] [C030][P] Implementar `dectl workflow list`: escanear `.dec/workflows/`; cargar cada YAML con loader; mostrar tabla con nombre, descripción e inputs requeridos; advertir por YAMLs inválidos sin abortar; `--json` retorna shape de data-model.md — S (REQ-C-008)
  > → reemplaza T029 del maestro

- [x] [C031][P] Implementar `dectl workflow describe <nombre>`: cargar workflow con loader; mostrar inputs en sección separada; mostrar steps con índice, tipo, descripción y contenido/comando; `--json` retorna shape de data-model.md — S (REQ-C-009)
  > → reemplaza T030 del maestro

- [x] [C032] Implementar `dectl workflow run <nombre>`: orquestar loader → validar inputs → trust check → ejecutar runner; soportar `--var`, `--dry-run`, `--from-step`; `--json` retorna shape de data-model.md incluyendo `failed_step` en error — M (REQ-C-010)
  > → reemplaza T031 del maestro

---

## Fase 6 — Tests de Integración y Validación

- [x] [C033] Escribir integration tests para workflow: `list` con workflows válidos e inválidos, `describe` workflow existente e inexistente, `run --dry-run` muestra pasos sin ejecutar, `run` con variable faltante aborta antes del primer paso — L
  > → reemplaza T032, T033 del maestro

- [x] [C034][P] Escribir test end-to-end completo: `project init` → `memory add` x3 → `memory list` → `memory search` → `memory show` → todo con `--json`; verificar JSON válido parseable en cada comando — M
  > → reemplaza T021 del maestro

- [x] [C035][P] Verificar tamaño del binario release: `cargo build --release && strip target/release/dectl && du -sh`; documentar resultado en research.md; abortar CI si supera 20MB — S
  > → reemplaza T022 del maestro

- [x] [C036][P] Revisar todos los `--help`: ejecutar cada comando con `--help`; verificar descripción, flags y ejemplo presentes; actualizar cualquier texto incompleto — S
  > → reemplaza T023 del maestro

- [x] [C037] Escribir tests unitarios del motor de interpolación: sustitución simple, múltiples variables, escape `\{{`, variable no declarada, requerida sin valor, opcional con y sin default — S
  > → reemplaza D025

- [x] [C038][P] Escribir tests de validación de schema de workflow: YAML válido, step sin `type`, `action` sin `cmd`, `write` sin `path`, variable en step no declarada en `inputs` — S
  > → reemplaza D032, D033

---

## Fase 7 — Polish (Phase 3 del maestro)

- [x] [C039] Implementar `dectl memory delete <id>`: soft-delete con flag `--hard` para eliminación permanente; confirmación antes de `--hard` a menos que `--non-interactive` — S
- [x] [C040][P] Implementar `dectl memory edit <id>`: abrir contenido en `$EDITOR` (desde `GlobalConfig.core.default_editor`); guardar al cerrar el editor — S
- [x] [C041] Generar shell completions para bash, zsh y fish con `clap_complete`; documentar instalación en README — M
- [x] [C042][P] Validar comportamiento con `--non-interactive` en todos los comandos que tienen prompts: `workflow run` sin trust, `memory delete --hard`; verificar exit code correcto — S

---

## Fase 8 — Auto-fill + Interactive Init

*Detecta stack y contexto automáticamente al hacer `dectl init`. Si proyecto vacío, ofrece preguntas interactivas.*

- [x] [C043] Implementar `project/auto_fill.rs`:
  - `is_project_empty() -> bool` — cuenta archivos excluyendo `.dec/`, `target/`, `node_modules/`, etc.
  - `detect_stack() -> DetectedStack` — detecta desde `package.json`, `Cargo.toml`, `go.mod`, `requirements.txt`, `pom.xml`, etc.
  - `scan_docs_for_context() -> OptionalContext` — lee `README.md`, `docs/`, `specs/` para extraer nombre y visión
  - `fill_project_files(detected, context)` — actualiza `.dec/config/project.toml` y `.dec/isa/project.isa.md`
  — M ✅

- [x] [C044] Modificar `project/init.rs` para integrar auto-fill:
  - Si proyecto no vacío → auto-detect + fill sin preguntar
  - Si proyecto vacío → prompt "¿Fill now / Manual / Cancel" (solo si TTY)
  - Si usuario elige "Fill now" → hacer preguntas y luego fill
  — M ✅

- [x] [C045] Implementar preguntas interactivas (solo proyecto vacío):
  - Project name (default: dir name)
  - Project type: (1) API (2) CLI (3) Web (4) Library (5) Other
  - Languages: separated by commas
  - Description: one line
  - Vision: what are we building?
  — M ✅

- [x] [C046] Tests para `detect_stack`: mock archivos de config (`package.json`, `Cargo.toml`, etc.) y verificar detección correcta — S ✅

- [x] [C047] Tests para `scan_docs_for_context`: mock `README.md` y specs, verificar extracción de nombre y visión — S ✅

---

## Fase 9 — Session Management

*Implementa `dectl session end` — automatización del cierre de sesión.*

- [x] [S001] Crear módulo `session/`: `mod.rs`, registrar en `main.rs` — S (REQ-C-014)
- [x] [S002] Definir structs de sesión: `SessionSummary`, `GitChanges`, `CapturedDecision`, `SessionEndResult` — S (REQ-C-014)
- [x] [S003] Implementar `session_summary.rs`: `generate_session_summary()` desde git log + sesión previa — M (REQ-C-014)
- [x] [S004] Implementar `write_last_session()`: formatear y escribir `.dec/state/last_session.md`, soportar `--dry-run` — S (REQ-C-014)
- [x] [S005] Implementar `git_sync.rs`: `detect_git_changes()` vía `git diff` + `git log` — M (REQ-C-014)
- [x] [S006] Implementar `sync_progress()`: actualizar `progress.json` desde cambios git, detectar nuevas features — M (REQ-C-014)
- [x] [S007] Implementar `decision_capture.rs`: `capture_decisions()` vía patrones regex sobre texto de sesión — M (REQ-C-014)
- [x] [S008] Implementar `save_decisions()`: INSERT nuevas decisiones en memory.db, evitar duplicados — S (REQ-C-014)
- [x] [S009] Wire `dectl session end` en `main.rs` con flags `--dry-run`, `--skip-git` — S (REQ-C-014)
- [x] [S010] Implementar `session/end.rs`: orquestar 5 pasos independientemente, recopilar resultados, output resumen — M (REQ-C-014)
- [x] [S011] Tests de integración para session end: dry-run, skip-git, JSON output, no git repo, con .dec/ — M
- [x] [S012] Actualizar documentación: README.md, CLAUDE.md, last_session.md — S

**Fase 9 COMPLETE** ✅

---

## Fase 10 — Config Sync en Session End

*Implementa detección automática de cambios en el stack del proyecto durante `dectl session end`.*

- [x] [CS001] Agregar tipos `ConfigDiff` y `ConfigSyncResult` en `session/types.rs` — S (REQ-C-017)
- [x] [CS002] Agregar campo `config_changes` a `SessionEndResult` — S (REQ-C-017)
- [x] [CS003] Crear módulo `session/config_sync.rs` con `sync_config()` — M (REQ-C-017)
- [x] [CS004] Implementar `compare_stacks()`: detectar nuevos languages/frameworks/tools y cambios de tipo — M (REQ-C-017)
- [x] [CS005] Implementar `merge_stack_into_toml()`: merge sin overwrite, preservar existentes — M (REQ-C-017)
- [x] [CS006] Implementar `check_isa_coherence()`: generar warnings si isa.md no menciona stack detectado — S (REQ-C-017)
- [x] [CS007] Integrar config_sync como Paso 4 en `session/end.rs` — M (REQ-C-017)
- [x] [CS008] Actualizar output humano y JSON para incluir config_changes — S (REQ-C-017)
- [x] [CS009] Registrar `config_sync` en `session/mod.rs` — XS (REQ-C-017)
- [x] [CS010] Tests unitarios para `compare_stacks()` — M
- [x] [CS011] Tests unitarios para `merge_stack_into_toml()` — M
- [x] [CS012] Tests de integración para config_sync — M
- [x] [CS013] Test de integración session end + config_sync — M
- [x] [CS014] Actualizar specs con REQ-C-017 — M
- [x] [CS015] Actualizar CLAUDE.md — S

**Fase 10 COMPLETE** ✅

---

## Fase 12 — Context Improvements (Phase 7 Polish)

*Mejoras a `dectl project context`: budget proporcional, formato compacto, priorización de cambios.*

- [x] [P001] Implementar proportional token budget en `project/context.rs`: `SectionDef` con pesos, `calculate_budgets()` con redistribución iterativa, `truncate_to_budget()` con footer indicador — M ✅
- [x] [P002] Implementar `--format compact`: `CompactOutput` struct, extractores de stack/session/progress/decisions/memory_hits — S ✅
- [x] [P003] Implementar recent changes prioritization: `parse_session_date()`, weight multipliers (×2 cambiado, ×0.5 no cambiado) basados en mtime vs fecha de sesión — M ✅

**Fase 12 COMPLETE** ✅

---

## Fase 13 — Memory Improvements

*FTS5 full-text search, `--type` categorization, field query language.*

- [x] [C13-01] Migración v2 (bundled → bundled-fts5, FTS5 virtual table + triggers + type column); rewrite `search.rs` para usar FTS5 con ranking y fallback LIKE — M ✅
- [x] [C13-02] Flag `--type` en `dectl memory add` con validación de valores + columna type en list/show output humano y JSON — M ✅
- [x] [C13-03] Comando `dectl memory query` con field query parser (tokenizer → expression parser → SQL builder parametrizado) — L ✅
- [x] [C13-04] Tests de query language: 5 integration + 13 unit tests — M ✅

**Fase 13 COMPLETE** ✅

---

## Fase 11 — Agent Commands

> Full specification in `specs/agents/`. CLI tasks A004, A005, A008, A011.

- [x] [A004] `dectl agent list` — load agents, display table, `--json` support — S (REQ-A-001)
- [x] [A005] `dectl agent describe <type>` — show full definition, `--json` support — S (REQ-A-008)
- [x] [A008] `dectl agent run <type> --task` — orchestrate loader → runner → log — M (REQ-A-002)
- [x] [A011] `dectl agent run --parallel` — parse types, parallel runner, consolidated summary — M (REQ-A-003)

**Fase 11 COMPLETE** ✅

---

## Progress Tracking

| Fase | Total | Done | In Progress | Blocked |
|------|-------|------|-------------|---------|
| Fase 1 — Infraestructura base | 10 | 10 | 0 | 0 |
| Fase 2 — Comandos de proyecto | 4 | 4 | 0 | 0 |
| Fase 3 — Comandos de memoria | 7 | 7 | 0 | 0 |
| Fase 4 — Protocolo | 2 | 2 | 0 | 0 |
| Fase 5 — Workflows | 9 | 9 | 0 | 0 |
| Fase 6 — Tests y validación | 6 | 6 | 0 | 0 |
| Fase 7 — Polish | 4 | 4 | 0 | 0 |
| Fase 8 — Auto-fill | 5 | 5 | 0 | 0 |
| Fase 9 — Session | 12 | 12 | 0 | 0 |
| Fase 10 — Config Sync | 15 | 15 | 0 | 0 |
| Fase 11 — Agent Commands | 4 | 4 | 0 | 0 |
| Fase 12 — Context Improvements | 3 | 3 | 0 | 0 |
| Fase 13 — Memory Improvements | 4 | 4 | 0 | 0 |
| **Total** | **80** | **80** | **0** | **0** |
