# Technical Implementation Plan — Integration Layer
> *Define los patrones concretos, flujos y artefactos de integración entre los tres actores.*
> *La integración no tiene código propio — produce artefactos (templates, flujos, documentación).*
> *Version: 1.0 | Status: Draft | Last updated: 2026-05-13*

---

## Referencias

- Implementa: `specs/integration/spec.md`
- Constitution: `specs/integration/constitution.md`
- Research: `specs/integration/research.md`
- Modifica: `specs/dot-dec/plan.md` (agrega integration.md al nivel 2)
- Extiende: `specs/cli/spec.md` (agrega REQ-C-013: dectl project context)

---

## Artefactos que produce este SDD

```
1. Template: .dec/prompts/system/integration.md   ← nuevo en nivel 2
2. Comando:  dectl project context                ← nuevo en CLI Phase 2
3. Documentación: flujos de integración           ← en este plan.md
4. Actualización: CLAUDE.md                       ← al final con todos los SDDs
```

---

## Artefacto 1 — Template `prompts/system/integration.md`

Este archivo va en `.dec/prompts/system/` y se crea en nivel 2 (`--standard`). Define el ciclo de sesión personalizable por proyecto.

```markdown
# Instrucciones de Sesión — [Nombre del Proyecto]
> **Para el modelo**: Lee y sigue estas instrucciones en cada sesión de trabajo.
> Actualiza este archivo si el equipo quiere cambiar el comportamiento del modelo.

---

## Al iniciar sesión

1. Lee `.dec/config/project.toml` y `.dec/isa/project.isa.md` para entender el proyecto
2. Lee `.dec/state/last_session.md` y retoma desde "Próximo paso recomendado"
3. Ejecuta `dectl project info --json` y escala al developer si hay warnings
4. Confirma en 2-3 líneas qué entendiste antes de preguntar qué hacer hoy

## Antes de actuar

1. Para cambios de arquitectura: lee `.dec/decisions/` primero
2. Para implementar una feature: busca su workflow en `.dec/workflows/`
3. Para términos de dominio: consulta `.dec/knowledge/glossary.md` si existe
4. Describe lo que vas a hacer antes de hacerlo — nunca actúes en silencio

## Al completar una tarea

1. Si completaste o avanzaste una feature: actualiza `.dec/state/progress.json`
2. Para decisiones importantes: ejecuta `dectl memory add "[resumen de la decisión]"`
3. Para decisiones arquitectónicas: crea `.dec/decisions/XXXX-nombre.md`

## Al finalizar sesión

1. Escribe `.dec/state/last_session.md`:
   - Qué se hizo
   - Qué quedó pendiente
   - Decisiones tomadas
   - Próximo paso recomendado
2. Ejecuta `dectl memory add "Sesión [fecha]: [resumen en una línea]"`
```

---

## Artefacto 2 — Comando `dectl project context`

Comando nuevo en Phase 2 del CLI. Genera un resumen compacto del proyecto para entornos stateless o para pasar contexto a cualquier IA via clipboard.

**Comportamiento**:
```
dectl project context [--max-tokens <n>] [--format text|json]
```

**Lógica de construcción del contexto** (por orden de prioridad hasta llegar al límite de tokens):

```
1. config/project.toml          (siempre incluido)
2. isa/project.isa.md           (siempre incluido)
3. state/last_session.md        (si existe)
4. decisions/*.md               (los 3 más recientes por fecha)
5. state/progress.json          (si existe)
6. prompts/system/base.md       (si existe)
7. knowledge/constraints.md     (si existe)
```

**Output por defecto** (text, max 4000 tokens):
```
=== dectl project context ===
Project: my-api (api) | Schema: 1.0
Stack: python, fastapi, postgresql

--- Vision ---
[contenido de isa/project.isa.md — sección Visión]

--- Last Session ---
[contenido de state/last_session.md]

--- Recent Decisions ---
[últimas 3 decisiones resumidas]

--- Progress ---
Done: 2 | In progress: 1 | Pending: 3 | Blocked: 0
=============================
```

**Output JSON** (`--format json`):
```json
{
  "status": "ok",
  "project": { "name": "my-api", "type": "api", "schema_version": "1.0" },
  "vision": "...",
  "last_session": "...",
  "recent_decisions": [...],
  "progress": { "done": 2, "in_progress": 1, "pending": 3, "blocked": 0 },
  "tokens_used": 1842,
  "tokens_limit": 4000
}
```

---

## Flujos de Integración

### Flujo 1 — Proyecto nuevo, primera sesión

```
Developer abre proyecto en entorno de IA
    │
    ├── Entorno NO tiene .dec/ → modelo detecta ausencia
    │     modelo: "No encontré .dec/ en este proyecto."
    │     modelo: "Ejecuta: dectl project init --standard"
    │     developer ejecuta → .dec/ creado
    │     modelo: "Ahora lee .dec/config/project.toml y completa los datos del proyecto"
    │
    └── Entorno SÍ tiene .dec/ → modelo ejecuta protocolo de inicio (REQ-I-001)
          1. lee config/project.toml
          2. lee isa/project.isa.md
          3. lee state/last_session.md
          4. ejecuta: dectl project info --json → verifica warnings
          5. confirma al developer: "Entendí que este proyecto es [X], última sesión fue [Y]..."
          6. pregunta: "¿Qué trabajamos hoy?"
```

---

### Flujo 2 — Sesión de retoma (caso más común)

```
Developer: "Continuemos donde quedamos"
    │
    modelo lee last_session.md
    modelo: "La última sesión implementamos auth JWT. Quedó pendiente el refresh token.
             ¿Continúo con eso?"
    │
    developer: "sí"
    │
    modelo busca workflow: dectl workflow list --json
    ├── existe "implement_feature" → lo sigue con vars feature_name=refresh_token
    └── no existe → sigue el prompt base en prompts/system/base.md
    │
    [trabaja...]
    │
    modelo al finalizar:
    ├── actualiza progress.json: refresh_token → "done"
    ├── dectl memory add "Refresh token implementado con rotación de tokens"
    └── escribe last_session.md con resumen y próximo paso
```

---

### Flujo 3 — Implementar feature con workflow

```
Developer: "Implementa el módulo de pagos con Stripe"
    │
    modelo: "Voy a ejecutar el workflow implement_feature"
    modelo ejecuta:
    │
    dectl workflow run implement_feature \
      --var feature_name=payment_processing \
      --var module=src/payments
    │
    [paso 1 — prompt]: modelo lee contexto del proyecto
    [paso 2 — action]: dectl memory search payment_processing
                       → modelo lee resultados, encuentra decisión previa de usar Stripe
    [paso 3 — prompt]: modelo diseña implementación respetando la decisión de Stripe
    [paso 4 — acción de codificación por el modelo]
    [paso 5 — action]: dectl memory add "Módulo de pagos con Stripe implementado"
    [paso 6 — write]:  actualiza last_session.md
    │
    workflow completa → modelo reporta al developer
```

---

### Flujo 4 — Decisión arquitectónica

```
Developer: "Necesito decidir si usar Redis o PostgreSQL para las sesiones"
    │
    modelo: "Voy a revisar las decisiones previas antes de opinar"
    modelo lee .dec/decisions/ → encuentra 0001-db-choice.md (PostgreSQL elegido)
    │
    modelo: "Ya tenemos PostgreSQL en el stack (ver 0001-db-choice.md).
             Para sesiones, recomiendo usar la misma base para reducir dependencias.
             ¿Quieres que documente esta decisión?"
    │
    developer: "sí"
    │
    modelo crea .dec/decisions/0002-session-storage.md con formato ADR
    modelo: dectl memory add "Decisión: sesiones en PostgreSQL, consistente con 0001"
    │
    modelo reporta: "Decisión documentada en 0002-session-storage.md"
```

---

### Flujo 5 — Manejo de error en comando

```
modelo ejecuta: dectl memory add "" --tags auth
    │
    CLI retorna exit code 1:
    {"status":"error","message":"Content cannot be empty",
     "hint":"provide content as argument or pipe via stdin"}
    │
    modelo: detecta status=error, lee message y hint
    modelo corrige: dectl memory add "Implementado sistema de auth con JWT" --tags auth
    │
    CLI retorna exit code 0:
    {"status":"ok","id":15,"preview":"Implementado sistema de auth con JWT"}
    │
    modelo continúa sin escalar al developer
```

---

### Flujo 6 — Error de sistema, escalada al developer

```
modelo ejecuta: dectl workflow run design_architecture
    │
    CLI retorna exit code 2:
    {"status":"error","message":"Permission denied: .dec/decisions/",
     "hint":"Check directory permissions"}
    │
    modelo: detecta exit code 2 (error de sistema)
    modelo NO reintenta (es un error de sistema, no de usuario)
    │
    modelo al developer:
    "No puedo continuar — el directorio .dec/decisions/ no tiene permisos de escritura.
     Error: Permission denied
     Solución sugerida por el CLI: verifica los permisos del directorio.
     Puedes corregirlo con: chmod 755 .dec/decisions/"
```

---

### Flujo 7 — Entorno stateless (sin ejecución de comandos)

```
Developer corre en terminal:
    dectl project context | pbcopy

Pega el contexto en el chat de la IA:
    "Aquí está el contexto de mi proyecto: [contexto]
     Necesito diseñar el módulo de pagos."
    │
    IA (sin acceso a comandos):
    - lee el contexto pegado
    - ve visión, últimas decisiones, progreso actual
    - responde con propuesta de diseño informada
    - no puede actualizar .dec/ ni memoria
    │
    Developer aplica manualmente los cambios sugeridos
    o vuelve a un entorno con ejecución para aplicarlos
```

---

## Fases de Implementación

### Fase 1 — Template integration.md
Agregar template de `prompts/system/integration.md` al nivel 2 de `dectl project init --standard`. Tarea D015b en dot-dec/tasks.md.

### Fase 2 — Comando `dectl project context`
Implementar el comando en el CLI como parte de Phase 2. Tarea C043 en cli/tasks.md.

### Fase 3 — Documentación pública
Documentar los 7 flujos en el README del proyecto como casos de uso reales. Incluir los flujos 1, 2 y 3 como quickstart.

---

## Cambios Requeridos en Otros SDDs

### dot-dec/plan.md
Agregar al nivel 2 (`--standard`):
```
└── prompts/
    └── system/
        ├── base.md              (ya existía)
        └── integration.md       ← NUEVO
```

### cli/spec.md
Agregar REQ-C-013:
```
REQ-C-013: Comando dectl project context
Genera resumen compacto del proyecto para entornos stateless.
Soporta --max-tokens <n> (default 4000) y --format text|json.
Prioriza archivos según orden definido en integration/plan.md.
```

### cli/tasks.md
Agregar tarea C043:
```
[C043] Implementar dectl project context: leer archivos .dec/ en orden de prioridad,
       respetar límite de tokens, soportar --format text|json — M (REQ-C-013)
```

---

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|-----------|
| Modelo ignora integration.md en entornos que no lo cargan automáticamente | Alto | Documentar en README cómo configurar cada entorno para cargar el prompt |
| `dectl project context` excede el límite de contexto del modelo | Medio | `--max-tokens` configurable; default conservador de 4000 tokens |
| Flujo de retoma falla si last_session.md está vacío o mal formado | Bajo | El modelo detecta contenido ausente y pregunta al developer en lugar de asumir |
| Developer usa entorno sin ejecución de comandos sin saberlo | Bajo | `dectl project context` como primer comando en el quickstart del README |
