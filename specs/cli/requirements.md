# Requirements Validation Checklist — dectl CLI
> *Valida que spec.md está completo, sin ambigüedad y listo para planificar.*
> *Last updated: 2026-06-02*

---

## Completeness

- [x] Todos los comandos identificados en master/plan.md tienen su requisito en este spec
- [x] Todos los usuarios están definidos (developer, modelo, scripts/automatización)
- [x] Todos los flags globales están especificados (--json, --help, --version, --non-interactive)
- [x] Requisitos no funcionales especificados (tiempo de respuesta, tamaño binario, portabilidad, señales)
- [x] Fuera de alcance explícitamente declarado
- [x] Todas las preguntas abiertas resueltas (stdin en memory add, --from-step en workflow run)

---

## Claridad

- [x] Cada criterio de aceptación usa SHALL
- [x] Sin términos ambiguos — tiempos en ms, tamaños en MB
- [x] Todos los exit codes definidos con sus significados (0, 1, 2, 3, 130)
- [x] Todos los JSON shapes tienen estructura definida con campos nombrados
- [x] Comportamiento de stdin especificado (TTY vs pipe)
- [x] Comportamiento de colores especificado (auto-desactivar si no es TTY)

---

## Consistencia

- [x] Sin requisitos contradictorios
- [x] Requisitos numerados secuencialmente (REQ-C-001 a REQ-C-020)
- [x] Sin requisitos duplicados
- [x] --json definido consistentemente en todos los comandos con mismo envelope
- [x] Exit codes consistentes entre todos los comandos

---

## Trazabilidad

- [x] REQ-C-001 → REQ-001 del maestro (project init)
- [x] REQ-C-002 → REQ-002 del maestro (project info / context reading)
- [x] REQ-C-003 → REQ-005 del maestro (project scan)
- [x] REQ-C-004 → REQ-003 del maestro (memory add)
- [x] REQ-C-005 → REQ-003 del maestro (memory list)
- [x] REQ-C-006 → REQ-003 del maestro (memory search)
- [x] REQ-C-007 → REQ-003 del maestro (memory show)
- [x] REQ-C-008 → REQ-005 del maestro (workflow list)
- [x] REQ-C-009 → REQ-005 del maestro (workflow describe)
- [x] REQ-C-010 → REQ-004 del maestro (workflow run)
- [x] REQ-C-011 → REQ-005 del maestro (exec-from-file)
- [x] REQ-C-012 → REQ-006 del maestro (--json global)
- [x] stdin en memory add → extiende REQ-C-004 sin contradecir REQ-003 del maestro
- [x] --from-step en workflow run → extiende REQ-C-010 sin contradecir REQ-004 del maestro
- [x] REQ-C-013 → REQ-002 del maestro (project context — context reading)
- [x] REQ-C-014 → REQ-008 del maestro (session end)
- [x] REQ-C-015 → REQ-006 del maestro (generate-completions)
- [x] REQ-C-016 → REQ-006 del maestro (version)
- [x] REQ-C-017 → REQ-008 del maestro (config sync)
- [x] REQ-C-018 → Nuevo — sistema de agentes (agents list/describe/run)
- [x] REQ-C-019 → REQ-009 del maestro (spec init)
- [x] REQ-C-020 → REQ-A-002 del maestro (agent trust)

---

## Alineación con Constitution del CLI

- [x] Todos los comandos tienen --help especificado (principio "cero configuración")
- [x] Todos los comandos de lectura son idempotentes según spec
- [x] Exit codes siguen el esquema definido en constitution (0/1/2/3/130)
- [x] --non-interactive especificado en REQ-C-012 (principio "predecible")
- [x] Mensajes de error incluyen qué/por qué/qué hacer (principio "fallar rápido y claramente")
- [x] Ningún comando asume un modelo específico (principio "el modelo puede leer cualquier output")
- [x] agent run implementa timeout y trust check (principio "predecible ante todo")
- [x] agent run --parallel usa threads con mpsc (principio "un comando hace una cosa")
- [x] session end integra 5 pasos independientes (principio "fallar rápido y claramente")

---

## Alineación con SDD Maestro y SDD de .dec/

- [x] project init soporta los tres niveles definidos en dot-dec/plan.md
- [x] workflow run implementa el schema de inputs/variables de dot-dec/data-model.md
- [x] --from-step no contradice el modelo de trust definido en master/data-model.md
- [x] memory add auto-detecta proyecto desde .dec/config/project.toml (consistente con dot-dec)
- [x] session end actualiza last_session.md, progress.json, memory.db (consistente con master/data-model.md)
- [x] agent list/describe/run/trust siguen el schema de agentes definido en agents/spec.md
- [x] generate-completions implementa bash/zsh/fish (consistente con constitution.md)

---

## Veredicto

- [x] ✅ LISTO PARA PLANIFICAR — Todos los checks pasaron. Continuar con `research.md` y `plan.md`.
