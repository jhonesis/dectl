# Implementation Tasks — Integration Layer
> *Tareas atómicas derivadas de plan.md del SDD de integración.*
> *Prefijo I para distinguir de tareas del maestro (T), .dec/ (D) y CLI (C).*
> *La integración no tiene código propio — sus tareas producen templates, un comando nuevo y documentación.*
> *Last updated: 2026-05-13*

---

## Leyenda

- `[Ixxx]` = Task ID
- `[P]` = Puede correr en paralelo con otras `[P]` en la misma fase
- `S / M / L` = Complejidad estimada
- `(REQ-I-xxx)` = Trazabilidad al spec de integración

---

## Dependencias con Otros SDDs

```
dot-dec D001–D015  (templates nivel 1 y 2 base)
    ↓
I001–I003          (agregar integration.md al nivel 2)
    ↓
cli C001–C013      (infraestructura base del CLI completa)
    ↓
I004–I006          (comando dectl project context)
    ↓
I007–I012          (documentación, flujos y validación end-to-end)
```

---

## Fase 1 — Template `integration.md`

- [ ] [I001] Implementar template de `.dec/prompts/system/integration.md` con las 4 secciones definidas en plan.md (`Al iniciar sesión`, `Antes de actuar`, `Al completar una tarea`, `Al finalizar sesión`); máximo 5 ítems por sección; lenguaje imperativo directo; incluir encabezado de propósito para el modelo — S (REQ-I-001, REQ-I-004)

- [ ] [I002] Agregar `integration.md` al nivel 2 de `dectl project init --standard` en `project/templates.rs` del CLI: debe crearse junto a `prompts/system/base.md`; actualizar lista de archivos creados en el output del comando — S (REQ-I-001)
  > Modifica: C011 (dectl project init) — agregar un archivo al nivel 2

- [ ] [I003] Agregar criterio de aceptación en REQ-D-005 de dot-dec/spec.md: cuando existe `integration.md` el modelo lo lee al inicio de sesión con prioridad sobre comportamiento por defecto; actualizar dot-dec/tasks.md marcando D015 como extendido por I002 — S

---

## Fase 2 — Comando `dectl project context`

- [x] [I004] Agregar REQ-C-013 a `specs/cli/spec.md`: definir comportamiento de `dectl project context` con `--max-tokens <n>` (default 4000) y `--format text|json`; documentar orden de prioridad de archivos y shape del JSON output según plan.md — S (REQ-I-007)

- [x] [I005] Implementar `dectl project context` en el CLI: leer archivos `.dec/` en orden de prioridad definido en plan.md; respetar límite de tokens (conteo aproximado por palabras × 1.3); construir output text o JSON según `--format`; informar `tokens_used` y `tokens_limit` en ambos formatos — M (REQ-I-007)

- [x] [I006] Escribir integration test para `dectl project context`: proyecto con `.dec/` nivel 2 completo → verificar que output text incluye visión, última sesión y decisiones recientes; verificar que `--format json` es JSON válido con todos los campos definidos en plan.md; verificar que `--max-tokens 100` trunca respetando el límite — M

---

## Fase 3 — Validación de Flujos

- [ ] [I007][P] Implementar flujo 1 end-to-end como test de aceptación: directorio vacío → `dectl project init --standard` → modelo lee contexto → confirma entendimiento en 3 líneas → fixture de modelo simulado que sigue integration.md — M (REQ-I-001)

- [ ] [I008][P] Implementar flujo 2 end-to-end como test de aceptación: proyecto con `last_session.md` con "Próximo paso: implementar refresh token" → modelo retoma sin preguntar → verifica que el próximo paso propuesto coincide — S (REQ-I-001)

- [ ] [I009][P] Implementar flujo 5 como test: `dectl memory add ""` → exit code 1 → modelo corrige a `dectl memory add "contenido válido"` → exit code 0 → sin escalada al developer — S (REQ-I-006)

- [ ] [I010][P] Implementar flujo 7 como test: `dectl project context` en proyecto nivel 2 → output cabe en 4000 tokens → contiene los campos mínimos (project, vision, last_session, progress) — S (REQ-I-007)

---

## Fase 4 — Documentación Pública

- [x] [I011] Escribir sección "How it works" en el README del proyecto con los flujos 1, 2 y 3 de plan.md como casos de uso del quickstart; incluir el "momento ancla" definido en strategic-notes.md como el primer ejemplo — M (REQ-I-001)

- [x] [I012][P] Escribir sección "Integrating with your AI environment" en el README: instrucciones específicas para Claude Code, Gemini CLI y Ollama sobre cómo cargar `integration.md` automáticamente al abrir el proyecto; documentar el modo stateless con `dectl project context` — M (REQ-I-007)

---

## Fase 5 — Actualización de SDDs Dependientes

- [ ] [I013] Actualizar `specs/dot-dec/plan.md`: agregar `integration.md` al template de nivel 2 en la sección de estructura de carpetas y en la lista de archivos del nivel 2 — S

- [ ] [I014][P] Actualizar `specs/dot-dec/tasks.md`: agregar tarea `[D015b] Implementar template de prompts/system/integration.md según specs/integration/plan.md — S` entre D015 y D016 — S

- [ ] [I015][P] Actualizar `specs/cli/tasks.md`: agregar tarea `[C043] Implementar dectl project context según REQ-C-013 — M` en Fase 2 (Workflows + Agents); actualizar progress tracking (+1 en Phase 2) — S

- [ ] [I016] Actualizar `CLAUDE.md` del proyecto: agregar SDD de integración al índice, agregar checkpoint antes de T025 para leer specs/integration/, documentar `dectl project context` en el command reference — S

---

## Progress Tracking

| Fase | Total | Done | In Progress | Blocked |
|------|-------|------|-------------|---------|
| Fase 1 — Template integration.md | 3 | 0 | 0 | 0 |
| Fase 2 — dectl project context | 3 | 0 | 0 | 0 |
| Fase 3 — Validación de flujos | 4 | 0 | 0 | 0 |
| Fase 4 — Documentación pública | 2 | 0 | 0 | 0 |
| Fase 5 — Actualización de SDDs | 4 | 0 | 0 | 0 |
| **Total** | **16** | **0** | **0** | **0** |

---

## Nota sobre tareas de documentación

Las tareas I011 e I012 (README) son las más visibles del proyecto hacia la comunidad. El "momento ancla" de strategic-notes.md debe quedar claro en el README antes de cualquier publicación — es lo que convierte a un visitante en usuario.
