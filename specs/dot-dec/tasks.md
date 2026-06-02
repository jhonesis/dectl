# Implementation Tasks — .dec/ System
> *Tareas atómicas derivadas de plan.md y data-model.md del SDD de .dec/*
> *Estas tareas extienden y refinan las tareas del maestro (specs/master/tasks.md).*
> *Se ejecutan en el contexto del CLI — son la implementación real de los templates y schemas.*
> *Last updated: 2026-05-26*

---

## Leyenda

- `[Dxxx]` = Task ID (prefijo D para distinguir del maestro)
- `[P]` = Puede correr en paralelo con otras `[P]` en la misma fase
- `S / M / L` = Complejidad estimada
- `(REQ-D-xxx)` = Trazabilidad al spec de `.dec/`
- `→ reemplaza Txxx` = Esta tarea reemplaza o refina una tarea del maestro

---

## Dependencia con el Maestro

Estas tareas deben ejecutarse **después de T007** (wire `--json` global flag) y **antes o en lugar de T008** (project/templates.rs). El maestro tiene un checkpoint en T007 — estas tareas son lo que define qué va en T008.

```
Master T001–T007 (setup + core)
    ↓
[READ] specs/dot-dec/plan.md y data-model.md
    ↓
D001–D015 (templates nivel 1 y 2)
    ↓
Master T009 (dec project init usa templates ya definidos)
    ↓
D016–D025 (templates nivel 3 + interpolación de variables)
    ↓
Master T025–T031 (workflow runner usa schema ya definido)
```

---

## Fase 1 — Templates y Schemas Base

### Estructuras de datos

- [x] [D001] Definir struct `ProjectConfig` en Rust que mapea `config/project.toml` según data-model.md: campos `dec`, `project`, `stack`, `conventions` con sus tipos y opcionalidad — S (REQ-D-001) ✅
- [x] [D002] Implementar deserialización de `ProjectConfig` desde TOML con `serde`; incluir validaciones: `schema_version` presente, `project.name` no vacío, `project.type` dentro del enum, `stack.languages` con mínimo un elemento — M (REQ-D-001, REQ-D-009) ✅
- [x] [D003] Definir struct `ProgressJson` y `Feature` en Rust que mapean `state/progress.json` según data-model.md; validar `status` dentro del enum (`pending`, `in_progress`, `done`, `blocked`) — S (REQ-D-007) ✅
- [x] [D004][P] Definir structs `Workflow`, `Input`, `Step`, `StepType` en Rust que mapean el schema YAML completo de workflows según data-model.md; incluir todos los campos condicionales — M (REQ-D-004) ✅
  > **Nota**: Este task refina y reemplaza al T025 del maestro con el schema completo incluyendo `inputs` y variables.

---

### Templates nivel 1 (mínimo)

- [x] [D005] Implementar template de `config/project.toml` como string embebida en el binario: campos con comentarios explicativos, valores placeholder claros — S (REQ-D-001, REQ-D-008) ✅
- [x] [D006][P] Implementar template de `isa/project.isa.md` como string embebida: todas las secciones con instrucciones internas para el modelo, ninguna sección vacía sin guía — S (REQ-D-002, REQ-D-008) ✅
- [x] [D007][P] Implementar template de `.dec/.gitignore` con los patrones definidos en plan.md — S (REQ-D-001) ✅
- [x] [D008] Implementar `dectl project init` (nivel 1): crear `.dec/`, escribir los tres templates, mostrar resumen de archivos creados y próximo paso recomendado; abortar si `.dec/` ya existe — M (REQ-D-001, REQ-D-010) ✅
  > **Nota**: Refina T009 del maestro con los templates reales definidos en este SDD.

---

### Templates nivel 2 (estándar)

- [x] [D009] Implementar template de `decisions/.gitkeep` y lógica de creación del directorio vacío — S (REQ-D-003) ✅
- [x] [D010][P] Implementar template de `workflows/implement_feature.yaml` con inputs y pasos completos según plan.md — S (REQ-D-004) ✅
- [x] [D011][P] Implementar template de `workflows/design_architecture.yaml` con inputs y pasos completos según plan.md — S (REQ-D-004) ✅
- [x] [D012][P] Implementar template de `prompts/system/base.md` con instrucciones base para el modelo según plan.md — S (REQ-D-005, REQ-D-008) ✅
- [x] [D013][P] Implementar template de `state/progress.json` con estructura inicial válida según data-model.md — S (REQ-D-007) ✅
- [x] [D014][P] Implementar template de `state/last_session.md` con estructura guiada y próximo paso inicial — S (REQ-D-007, REQ-D-008) ✅
- [x] [D015] Implementar flag `--standard` en `dectl project init`: crear nivel 1 + templates D009–D014; mostrar resumen diferenciado respecto al nivel mínimo — M (REQ-D-010) ✅
- [x] [D015b][P] Implementar template de `prompts/system/integration.md` con las 4 secciones definidas en integration/plan.md (`Al iniciar sesión`, `Antes de actuar`, `Al completar una tarea`, `Al finalizar sesión`); incluir referencia a `dectl session end` para cierre automatizado — S (REQ-D-005, REQ-I-001)

**Fase 1 COMPLETE** ✅

---

## Fase 2 — Templates Nivel 3 + Interpolación de Variables

### Templates nivel 3 (completo)

- [x] [D016] Implementar template de `isa/architecture.isa.md` con todas las secciones guiadas y tabla de componentes vacía — S (REQ-D-002, REQ-D-008) ✅
- [x] [D017][P] Implementar templates de `prompts/tasks/`: `implement_feature.md`, `write_tests.md`, `review_code.md`, `document_module.md` — cada uno con instrucciones específicas y referencias a archivos `.dec/` relevantes — M (REQ-D-005, REQ-D-008) ✅
- [x] [D018][P] Implementar template de `knowledge/glossary.md` con estructura guiada y ejemplo de entrada — S (REQ-D-006, REQ-D-008) ✅
- [x] [D019][P] Implementar template de `knowledge/constraints.md` con secciones para limitaciones técnicas, requisitos no funcionales y restricciones — S (REQ-D-006, REQ-D-008) ✅
- [x] [D020] Implementar flag `--full` en `dectl project init`: crear nivel 2 + templates D016–D019; mostrar resumen completo de todo lo creado — M (REQ-D-010) ✅

---

### Motor de interpolación de variables

- [x] [D021] Implementar función `interpolate(template: &str, vars: &HashMap<String, String>) -> Result<String>` en `workflow/runner.rs`: sustituye `{{nombre}}` por valores del mapa, soporte de escape `\{{`, error si variable referenciada no está en el mapa — M (REQ-D-004) ✅
- [x] [D022] Implementar validación de inputs al cargar un workflow: extraer todas las referencias `{{...}}` de todos los steps, verificar que cada una esté declarada en `inputs`, error descriptivo si hay referencias sin declarar — M (REQ-D-004) ✅
- [x] [D023] Implementar resolución de valores de inputs al ejecutar un workflow: verificar que todos los `required: true` tienen valor provisto vía flags o interacción; aplicar `default` para los opcionales sin valor; error antes del primer paso si falta algún requerido — M (REQ-D-004) ✅
- [x] [D024] Implementar soporte de `--var nombre=valor` como flag repetible en `dectl workflow run`: permite pasar variables de entrada desde la línea de comandos — S (REQ-D-004) ✅
- [x] [D025] Escribir tests unitarios para el motor de interpolación: sustitución simple, múltiples variables, escape `\{{`, variable no declarada, variable requerida sin valor, variable opcional con y sin default — M ✅

**Fase 2 COMPLETE** ✅

---

## Fase 3 — Validación y Comandos de Inspección

### Validación de `.dec/`

- [x] [D026] Implementar `dectl project info` con validación de schema: leer `config/project.toml`, validar según D002, mostrar warnings por campo inválido o ausente; mostrar resumen legible del proyecto — M (REQ-D-001, REQ-D-009) ✅
  > **Nota**: Refina T010 del maestro con validaciones reales de schema.
- [x] [D027][P] Implementar advertencia de `schema_version`: si la versión del proyecto es major mayor a la soportada por el CLI, abortar con mensaje; si es minor mayor, continuar con advertencia — S (REQ-D-009) ✅
- [x] [D028][P] Implementar validación de `state/progress.json` al leer: IDs únicos, status dentro del enum, `updated_at` en formato ISO 8601 cuando no está vacío — S (REQ-D-007) ✅

---

### Comandos de inspección de workflows

- [x] [D029] Implementar `dectl workflow list`: escanear `.dec/workflows/`, cargar cada YAML, mostrar tabla con nombre, descripción e inputs requeridos — S (REQ-D-004) ✅
  > **Nota**: Refina T029 del maestro mostrando también inputs requeridos.
- [x] [D030][P] Implementar `dectl workflow describe <nombre>`: mostrar schema completo del workflow — inputs con required/default, steps con tipo, descripción y contenido/comando — S (REQ-D-004) ✅
  > **Nota**: Refina T030 del maestro con display de inputs.

---

### Tests de integración

- [x] [D031] Escribir tests de integración para los tres niveles de `dectl project init`: verificar que cada nivel crea exactamente los archivos esperados, nada más — M ✅
- [x] [D032][P] Escribir tests de integración para interpolación de variables end-to-end: workflow con variables → `dectl workflow run --var nombre=valor` → verificar sustitución correcta en steps — M ✅
- [x] [D033][P] Escribir tests de validación de schema: `project.toml` con campos faltantes, tipos incorrectos, `schema_version` incompatible — S ✅

**Fase 3 COMPLETE** ✅

---

## Actualización del Maestro

Al completar este SDD, actualizar `specs/master/tasks.md`:

- [x] [D034] Marcar T008 como reemplazado por D005–D007 — S ✅
- [x] [D035] Marcar T009 como refinado por D008 — S ✅
- [x] [D036] Agregar checkpoints en `specs/master/tasks.md`:
  - Antes de T008: `[READ] specs/dot-dec/plan.md y data-model.md`
  - Antes de T025: `[READ] specs/dot-dec/data-model.md — schema Workflow/Input/Step`
  - Actualizar CLAUDE.md con referencia a specs/dot-dec/ — S ✅

**Actualización del Maestro COMPLETE** ✅

---

## Progress Tracking

| Fase | Total | Done | In Progress | Blocked |
|------|-------|------|-------------|---------|
| Fase 1 — Templates base | 15 | 15 | 0 | 0 |
| Fase 2 — Nivel 3 + Variables | 10 | 10 | 0 | 0 |
| Fase 3 — Validación + Inspección | 8 | 8 | 0 | 0 |
| Actualización maestro | 3 | 3 | 0 | 0 |
| **Total** | **36** | **36** | **0** | **0** |
