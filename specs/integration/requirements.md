# Requirements Validation Checklist — Integration Layer
> *Valida que spec.md está completo, sin ambigüedad y listo para planificar.*
> *Last updated: 2026-05-13*

---

## Completeness

- [x] Todos los actores definidos (modelo, developer, entorno de codificación)
- [x] El ciclo completo de sesión está cubierto (inicio, trabajo, cierre)
- [x] Protocolo de carga de contexto especificado con orden explícito
- [x] Protocolo de invocación de comandos especificado
- [x] Manejo de errores por exit code cubierto (1, 2, consecutivos)
- [x] Compatibilidad con entornos sin ejecución de comandos cubierta
- [x] exec-from-file como mecanismo de batch especificado
- [x] Requisitos no funcionales definidos (latencia, resiliencia, observabilidad, idempotencia)
- [x] Fuera de alcance explícitamente declarado
- [x] Todas las preguntas abiertas resueltas (integration.md en nivel 2, schema check vía warnings)

---

## Claridad

- [x] Cada criterio de aceptación usa SHALL
- [x] Sin términos ambiguos — latencia definida en ms
- [x] El orden de lectura de contexto está explícito (project.toml → project.isa.md → last_session.md)
- [x] El comportamiento ante cada exit code está diferenciado (1 = reintentar, 2 = escalar inmediatamente)
- [x] La personalización vía integration.md tiene prioridad explícita sobre comportamiento por defecto
- [x] El mecanismo de schema check está delegado claramente al CLI (no al modelo directamente)

---

## Consistencia

- [x] Sin requisitos contradictorios
- [x] Requisitos numerados secuencialmente (REQ-I-001 a REQ-I-008)
- [x] Sin requisitos duplicados
- [x] REQ-I-007 (compatibilidad) no contradice REQ-I-003 (invocación de comandos) — modo lectura es un subset válido

---

## Trazabilidad

- [x] REQ-I-001 → developer (retomar sesión sin repetir contexto)
- [x] REQ-I-002 → modelo (consulta de contexto específico)
- [x] REQ-I-003 → entorno de codificación (patrón de invocación)
- [x] REQ-I-004 → developer (ciclo de sesión predecible)
- [x] REQ-I-005 → developer (uso consistente de workflows)
- [x] REQ-I-006 → developer (manejo de errores transparente)
- [x] REQ-I-007 → developer (compatibilidad con cualquier entorno)
- [x] REQ-I-008 → modelo/scripts (batch via exec-from-file)

---

## Alineación con Otros SDDs

- [x] REQ-I-001 es consistente con los archivos de `.dec/` definidos en dot-dec/spec.md (REQ-D-007, REQ-D-008)
- [x] REQ-I-003 usa `--json` y `status` definidos en cli/spec.md (REQ-C-012)
- [x] REQ-I-004 referencia `progress.json` y `last_session.md` definidos en dot-dec/data-model.md
- [x] REQ-I-005 es consistente con el schema de workflows definido en dot-dec/data-model.md
- [x] REQ-I-006 usa exit codes definidos en cli/constitution.md (0/1/2/3/130)
- [x] REQ-I-008 usa `dectl exec-from-file` definido en cli/spec.md (REQ-C-011)
- [x] integration.md añadido al nivel 2 de `.dec/` — debe reflejarse en dot-dec/plan.md (templates nivel 2)
- [x] Schema check vía `warnings` en `dectl project info --json` — consistente con cli/data-model.md

---

## Alineación con Constitution

- [x] REQ-I-002 y REQ-I-005 implementan "El modelo lidera, el CLI sigue"
- [x] REQ-I-007 implementa "La integración no requiere instalación extra"
- [x] REQ-I-006 implementa "Los errores de integración son recuperables"
- [x] REQ-I-003 implementa "Todo output del CLI es consumible por el modelo"
- [x] Ningún requisito crea acoplamiento entre actores fuera del contrato

---

## Acción Pendiente antes de plan.md

- [ ] Agregar template de `prompts/system/integration.md` al nivel 2 en `specs/dot-dec/plan.md`
  *(cambio menor — no invalida las tareas D ya definidas, solo agrega D015b)*

---

## Veredicto

- [x] ✅ LISTO PARA PLANIFICAR — Todos los checks pasaron. Continuar con `research.md` y `plan.md`.
