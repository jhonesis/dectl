# Implementation Tasks — dectl CLI
> *Tareas atómicas derivadas de plan.md y data-model.md del SDD del CLI.*
> *Prefijo C para distinguir de tareas del maestro (T) y de .dec/ (D).*
> *Last updated: 2026-05-13*

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

- [ ] [C001] Implementar `core/output.rs`: struct `Output` con `json_mode` y `color`; métodos `success`, `error`, `info`, `step`; detección automática de TTY con `is-terminal`; respeto de variable `NO_COLOR` — M (REQ-C-012)
- [ ] [C002] Implementar envelope JSON `{"status":"ok"|"error",...}` usando `serde_json`; método `Output::success` serializa cualquier `T: Serialize`; método `Output::error` siempre va a stderr — S (REQ-C-012)
- [ ] [C003] Implementar `core/markdown.rs`: función `to_plain(input: &str) -> String` que elimina headers, bold, italic y backticks de inline code — S (REQ-C-007)

---

### Configuración

- [ ] [C004] Implementar `core/config.rs`: deserializar `GlobalConfig` desde `~/.dectl/config.toml` con `serde` + `toml`; crear archivo con defaults si no existe; exponer función `load_global() -> Result<GlobalConfig>` — M (REQ-C-012)
- [ ] [C005][P] Implementar `core/config.rs` carga de `ProjectConfig` desde `.dec/config/project.toml`; validar `schema_version`, `project.name`, `project.type`, `stack.languages`; retornar warnings en lugar de errores para campos no críticos — M (REQ-C-002)
- [ ] [C006][P] Implementar detección automática de proyecto: buscar `.dec/config/project.toml` en el directorio actual y padres hasta el home; retornar `Option<ProjectConfig>` — S (REQ-C-004)

---

### Stdin y Señales

- [ ] [C007] Implementar `core/stdin.rs`: función `read_content(arg: Option<String>) -> Result<String>`; leer de argumento, pipe stdin, o error si ninguno — según patrón en plan.md — S (REQ-C-004)
- [ ] [C008] Implementar handler SIGINT con `ctrlc`: limpiar línea de progreso en stderr, imprimir "Interrupted.", exit code 130 — S (constitution)

---

### Exit Codes

- [ ] [C009] Implementar enum `ExitCode` en `core/error.rs` con variantes `Success(0)`, `UserError(1)`, `SystemError(2)`, `StateError(3)`, `Interrupted(130)`; implementar función `classify_error(e: &anyhow::Error) -> ExitCode` — S (constitution)
- [ ] [C010] Configurar `main.rs`: setup ctrlc → parse CLI → ejecutar comando → output resultado o error → exit con código correcto — M

---

## Fase 2 — Comandos de Proyecto

- [ ] [C011] Implementar `dectl project init`: lógica de creación de niveles 1/2/3 usando templates de D005–D020; abortar si `.dec/` existe; mostrar archivos creados y próximo paso; `--json` retorna shape de plan.md — M (REQ-C-001)
  > → reemplaza T009 del maestro con implementación real basada en templates del SDD de .dec/

- [ ] [C012][P] Implementar `dectl project info`: cargar `ProjectConfig` + leer primera sección de `project.isa.md` + cargar `progress.json`; mostrar resumen con warnings por archivos faltantes; validar `schema_version`; `--json` retorna shape de data-model.md — M (REQ-C-002)
  > → reemplaza T010 del maestro

- [ ] [C013][P] Implementar `dectl project scan`: usar `ignore` crate para recorrer árbol respetando `.gitignore`; soportar `--depth <n>`; `--json` retorna árbol como array de nodos; excluir `.git/` y `target/` si no hay `.gitignore` — M (REQ-C-003)
  > → reemplaza T011 del maestro

- [ ] [C014] Escribir integration tests para comandos de proyecto: `init` crea estructura correcta por nivel, `init` aborta si `.dec/` existe, `info` maneja archivos faltantes con warnings, `scan` respeta `.gitignore`, todos soportan `--json` válido — M

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

- [ ] [C024] Implementar `workflow/schema.rs`: structs `Workflow`, `InputDef`, `Step`, `StepType` según data-model.md de este SDD — S
  > → reemplaza D004 / T025 del maestro con implementación completa incluyendo `inputs`

- [ ] [C025] Implementar `workflow/loader.rs`: leer `.dec/workflows/<name>.yaml`; parsear con `serde_yaml`; validar que variables referenciadas en steps estén declaradas en `inputs`; error descriptivo por campo faltante — M (REQ-C-009)
  > → reemplaza T026 del maestro

- [ ] [C026][P] Implementar `workflow/trust.rs`: leer/escribir `~/.dectl/trust.toml`; comprobar si `<project_path>/<workflow_name>` es trusted; si no y hay `action` steps: mostrar comandos + pedir Y/n + guardar; si `--non-interactive` y no trusted: error — M (REQ-C-010)
  > → reemplaza T027 del maestro

- [ ] [C027] Implementar motor de interpolación en `workflow/runner.rs`: función `interpolate(template: &str, vars: &HashMap<String,String>) -> Result<String>`; soporte de escape `\{{`; error si variable no declarada — M
  > → reemplaza D021

- [ ] [C028][P] Implementar resolución de inputs en `workflow/runner.rs`: extraer vars de flags `--var nombre=valor`; verificar obligatorias presentes; aplicar defaults para opcionales; error antes del primer paso si faltan requeridas — M (REQ-C-010)
  > → reemplaza D023

- [ ] [C029] Implementar ejecución de steps en `workflow/runner.rs`: iterar steps con `--from-step` skip; ejecutar según tipo (prompt/action/write); manejar `--dry-run`; capturar output de `action` en tiempo real; escribir archivo en `write`; parar y reportar en fallo con hint de `--from-step N` — L (REQ-C-010)
  > → reemplaza T028 del maestro

- [ ] [C030][P] Implementar `dectl workflow list`: escanear `.dec/workflows/`; cargar cada YAML con loader; mostrar tabla con nombre, descripción e inputs requeridos; advertir por YAMLs inválidos sin abortar; `--json` retorna shape de data-model.md — S (REQ-C-008)
  > → reemplaza T029 del maestro

- [ ] [C031][P] Implementar `dectl workflow describe <nombre>`: cargar workflow con loader; mostrar inputs en sección separada; mostrar steps con índice, tipo, descripción y contenido/comando; `--json` retorna shape de data-model.md — S (REQ-C-009)
  > → reemplaza T030 del maestro

- [ ] [C032] Implementar `dectl workflow run <nombre>`: orquestar loader → validar inputs → trust check → ejecutar runner; soportar `--var`, `--dry-run`, `--from-step`; `--json` retorna shape de data-model.md incluyendo `failed_step` en error — M (REQ-C-010)
  > → reemplaza T031 del maestro

---

## Fase 6 — Tests de Integración y Validación

- [ ] [C033] Escribir integration tests para workflow: `list` con workflows válidos e inválidos, `describe` workflow existente e inexistente, `run --dry-run` muestra pasos sin ejecutar, `run` con variable faltante aborta antes del primer paso — L
  > → reemplaza T032, T033 del maestro

- [ ] [C034][P] Escribir test end-to-end completo: `project init` → `memory add` x3 → `memory list` → `memory search` → `memory show` → todo con `--json`; verificar JSON válido parseable en cada comando — M
  > → reemplaza T021 del maestro

- [ ] [C035][P] Verificar tamaño del binario release: `cargo build --release && strip target/release/dectl && du -sh`; documentar resultado en research.md; abortar CI si supera 20MB — S
  > → reemplaza T022 del maestro

- [ ] [C036][P] Revisar todos los `--help`: ejecutar cada comando con `--help`; verificar descripción, flags y ejemplo presentes; actualizar cualquier texto incompleto — S
  > → reemplaza T023 del maestro

- [ ] [C037] Escribir tests unitarios del motor de interpolación: sustitución simple, múltiples variables, escape `\{{`, variable no declarada, requerida sin valor, opcional con y sin default — S
  > → reemplaza D025

- [ ] [C038][P] Escribir tests de validación de schema de workflow: YAML válido, step sin `type`, `action` sin `cmd`, `write` sin `path`, variable en step no declarada en `inputs` — S
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
| **Total** | **42** | **0** | **0** | **0** |
