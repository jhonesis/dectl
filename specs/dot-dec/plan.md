# Technical Implementation Plan — .dec/ System
> *Describe CÓMO construir lo que spec.md define. Technology-specific.*
> *Version: 1.0 | Status: Updated | Last updated: 2026-06-02*

---

## Referencias

- Implementa: `specs/dot-dec/spec.md`
- Constitution: `specs/dot-dec/constitution.md`
- Research: `specs/dot-dec/research.md`
- Depende de: `specs/master/plan.md` (el CLI que genera y manipula `.dec/`)

---

## Estructura Completa de `.dec/`

```
.dec/
├── .gitignore                        ← generado automáticamente (RQ-D-005)
├── config/
│   └── project.toml                  ← REQ-D-001 (nivel 1)
├── isa/
│   ├── project.isa.md                ← REQ-D-002 (nivel 1)
│   └── architecture.isa.md           ← REQ-D-002 (nivel 3)
├── decisions/                        ← REQ-D-003 (nivel 2)
│   └── .gitkeep
├── workflows/                        ← REQ-D-004 (nivel 2)
│   ├── implement_feature.yaml
│   └── design_architecture.yaml
├── prompts/                          ← REQ-D-005
│   ├── system/
│   │   └── base.md                   ← nivel 2
│   └── tasks/                        ← nivel 3
│       ├── implement_feature.md
│       ├── write_tests.md
│       ├── review_code.md
│       └── document_module.md
├── knowledge/                        ← REQ-D-006 (nivel 3)
│   ├── glossary.md
│   └── constraints.md
└── state/                            ← REQ-D-007
    ├── progress.json                 ← nivel 2
    └── last_session.md               ← nivel 2
```

---

## Niveles de Inicialización

### Nivel 1 — Mínimo (`dectl project init`)

Dos archivos. Suficiente para que cualquier modelo entienda el proyecto.

```
.dec/
├── .gitignore
├── config/
│   └── project.toml
└── isa/
    └── project.isa.md
```

### Nivel 2 — Estándar (`dectl project init --standard`)

Agrega el sistema de pensamiento: decisiones, workflows base, prompt del sistema, integración y estado.

```
.dec/
├── .gitignore
├── config/project.toml
├── isa/project.isa.md
├── decisions/.gitkeep
├── workflows/
│   ├── implement_feature.yaml
│   └── design_architecture.yaml
├── prompts/system/
│   ├── base.md
│   └── integration.md       ← instrucciones de ciclo de sesión
└── state/
    ├── progress.json
    └── last_session.md
```

### Nivel 3 — Completo (`dectl project init --full`)

Agrega arquitectura, prompts por tarea, knowledge base y ISA de arquitectura.

```
.dec/                          (todo lo de nivel 2, más:)
├── isa/architecture.isa.md
├── prompts/tasks/
│   ├── implement_feature.md
│   ├── write_tests.md
│   ├── review_code.md
│   └── document_module.md
└── knowledge/
    ├── glossary.md
    └── constraints.md
```

---

## Templates Completos

### `config/project.toml`

```toml
# dectl project configuration
# Este archivo define el proyecto para el modelo y las herramientas.
# El modelo debe leerlo al inicio de cada sesión.

[dec]
schema_version = "1.0"

[project]
name = "nombre-del-proyecto"
# Tipo de proyecto: api | cli | microservice | monolith | library | other
type = "api"
description = "Descripción breve del proyecto en una frase."

[stack]
# Lista de tecnologías principales del proyecto
languages = ["python"]
frameworks = ["fastapi"]
databases = ["postgresql"]
tools = ["docker", "git"]

[conventions]
# Convenciones especiales que el modelo debe seguir en este proyecto.
# Ejemplos: "siempre usar type hints en Python", "commits en inglés"
rules = []
```

---

### `isa/project.isa.md`

```markdown
# ISA: [Nombre del Proyecto]
> **Para el modelo**: Lee este documento antes de tomar cualquier decisión importante.
> Actualiza este archivo cuando la visión o el alcance cambien significativamente.
> Si algo aquí contradice lo que ves en el código, pregunta al developer antes de asumir.

---

## Visión
<!-- Una frase: qué es el proyecto y para quién. -->
<!-- Ejemplo: "Una API REST para gestionar inventario de pequeñas tiendas." -->


## Objetivo Principal
<!-- Qué problema concreto resuelve y cómo se mide el éxito. -->
<!-- Ejemplo: "Eliminar hojas de cálculo manuales. Éxito = 0 errores de stock en 30 días." -->


## Alcance (qué SÍ incluye)
<!-- Lista concreta de lo que el proyecto construye. -->
-
-

## No-Objetivos (qué NO incluye)
<!-- Lista explícita de lo que este proyecto NO hace. Evita scope creep. -->
-
-

## Stack Tecnológico
<!-- Tecnologías principales. El detalle está en config/project.toml. -->


## Restricciones Conocidas
<!-- Limitaciones técnicas, de tiempo o de recursos que el modelo debe respetar. -->
<!-- Ejemplo: "El servidor tiene 512MB RAM. Sin dependencias con más de 50MB." -->


## Riesgos Principales
<!-- Los 2-3 riesgos más importantes. Breve. -->
1.
2.
```

---

### `isa/architecture.isa.md`

```markdown
# ISA: Arquitectura — [Nombre del Proyecto]
> **Para el modelo**: Lee este documento antes de proponer cambios arquitectónicos.
> Consulta también decisions/ para entender por qué la arquitectura es como es.

---

## Visión de Arquitectura
<!-- Descripción en 2-3 frases de la arquitectura general. -->


## Módulos / Componentes Principales
<!-- Lista de componentes con una línea de responsabilidad cada uno. -->
| Componente | Responsabilidad |
|-----------|----------------|
|           |                |

## Flujos Principales
<!-- Los 2-3 flujos más importantes del sistema, descritos brevemente. -->

### Flujo 1: [nombre]
<!-- Descripción del flujo paso a paso. -->

## Decisiones de Diseño Clave
<!-- Las decisiones que más impactan la arquitectura. Referencia decisions/ para el detalle. -->
- [Decisión]: [consecuencia en una línea]

## Diagrama (opcional)
<!-- ASCII diagram o descripción textual de la arquitectura. -->

## Lo que NO debe cambiar sin revisar decisions/
<!-- Partes de la arquitectura que tienen restricciones fuertes. -->
-
```

---

### `decisions/0001-ejemplo.md` (template)

```markdown
# [0001] Título de la Decisión

**Fecha**: YYYY-MM-DD
**Estado**: activa
<!-- Estados válidos: activa | obsoleta | supersedida por [XXXX] -->

---

## Contexto
<!-- Qué situación o problema motivó esta decisión. Sé específico. -->


## Decisión
<!-- La decisión en una frase clara y directa. -->


## Alternativas Consideradas
- **Opción A**: descripción — descartada porque [razón]
- **Opción B**: descripción — descartada porque [razón]

## Justificación
<!-- Por qué esta opción es mejor que las alternativas para este proyecto. -->


## Consecuencias
<!-- Qué cambia, qué se gana, qué se sacrifica con esta decisión. -->
```

---

### `workflows/implement_feature.yaml`

```yaml
name: implement_feature
description: Implementa una nueva feature completa con tests y documentación
inputs:
  - name: feature_name
    description: Nombre de la feature (ej. "user_authentication", "payment_processing")
    required: true
  - name: module
    description: Módulo o carpeta donde se implementará
    required: true
  - name: include_tests
    description: Generar tests automáticamente (true/false)
    required: false
    default: "true"

steps:
  - type: prompt
    description: Cargar contexto del proyecto
    content: |
      Lee .dec/isa/project.isa.md y .dec/config/project.toml.
      Lee .dec/decisions/ para entender restricciones arquitectónicas.
      Confirma que entiendes el proyecto antes de continuar.

  - type: action
    description: Buscar decisiones relevantes en memoria
    cmd: ["dectl", "memory", "search", "{{feature_name}}"]

  - type: prompt
    description: Diseñar la implementación
    content: |
      Diseña la implementación de "{{feature_name}}" en el módulo "{{module}}".
      - Sigue las convenciones en .dec/config/project.toml
      - Respeta las decisiones en .dec/decisions/
      - Describe los archivos que vas a crear/modificar antes de hacerlo

  - type: prompt
    description: Implementar
    content: |
      Implementa "{{feature_name}}" en "{{module}}".
      Include tests: {{include_tests}}.
      Al terminar cada archivo, confirma que compila/pasa lint.

  - type: action
    description: Registrar decisiones tomadas durante la implementación
    cmd: ["dectl", "memory", "add", "Implementada feature {{feature_name}} en {{module}}"]

  - type: write
    description: Actualizar estado del proyecto
    path: .dec/state/last_session.md
    content: |
      # Última sesión
      **Fecha**: (completar)
      **Qué se hizo**: Implementada feature {{feature_name}} en {{module}}
      **Pendiente**: (completar)
      **Decisiones tomadas**: (completar)
```

---

### `workflows/design_architecture.yaml`

```yaml
name: design_architecture
description: Guía al modelo para diseñar o revisar la arquitectura del proyecto
inputs:
  - name: focus
    description: Aspecto específico a diseñar (ej. "auth", "database", "api") o "general" para arquitectura completa
    required: false
    default: "general"

steps:
  - type: prompt
    description: Cargar contexto completo
    content: |
      Lee .dec/isa/project.isa.md, .dec/config/project.toml y .dec/decisions/.
      Identifica los componentes principales y sus responsabilidades.
      Foco de esta sesión: {{focus}}.

  - type: action
    description: Buscar decisiones de arquitectura previas
    cmd: ["dectl", "memory", "search", "arquitectura"]

  - type: prompt
    description: Proponer arquitectura
    content: |
      Propón la arquitectura para "{{focus}}".
      Incluye: módulos, responsabilidades, flujos principales y trade-offs.
      Si hay decisiones previas en decisions/ que apliquen, referencialas.

  - type: write
    description: Documentar la decisión arquitectónica
    path: .dec/decisions/XXXX-architecture-{{focus}}.md
    content: |
      # [XXXX] Arquitectura: {{focus}}
      **Fecha**: (completar)
      **Estado**: activa

      ## Contexto
      (completar)

      ## Decisión
      (completar)

      ## Alternativas Consideradas
      (completar)

      ## Justificación
      (completar)

      ## Consecuencias
      (completar)

  - type: prompt
    description: Actualizar ISA de arquitectura
    content: |
      Actualiza .dec/isa/architecture.isa.md con la arquitectura diseñada.
      Luego ejecuta: dectl memory add "Arquitectura {{focus}} diseñada y documentada"
```

---

### `prompts/system/base.md`

```markdown
# System Prompt Base — [Nombre del Proyecto]
> **Instrucciones para el modelo**: Este prompt define tu comportamiento en este proyecto.
> El developer puede actualizarlo en cualquier momento. Reléelo si recibes instrucciones contradictorias.

---

## Contexto del Proyecto
Estás trabajando en [nombre del proyecto]. Lee .dec/isa/project.isa.md para entender qué construyes.

## Comportamiento Esperado

**Antes de actuar**:
- Lee el contexto relevante en .dec/ antes de tomar decisiones importantes
- Consulta .dec/decisions/ antes de proponer cambios arquitectónicos
- Si algo no está claro, pregunta antes de asumir

**Al escribir código**:
- Sigue las convenciones en .dec/config/project.toml
- Respeta las restricciones en .dec/knowledge/constraints.md (si existe)
- Usa los términos definidos en .dec/knowledge/glossary.md (si existe)

**Al terminar una tarea**:
- Actualiza .dec/state/progress.json si completaste una feature
- Actualiza .dec/state/last_session.md con un resumen de lo hecho
- Registra decisiones importantes con: dectl memory add "..."

## Lo que NO debes hacer
- Inventar términos del dominio que no están en el glosario
- Proponer cambios que contradigan decisiones en .dec/decisions/
- Asumir requisitos no documentados — pregunta
```

---

### `prompts/system/integration.md`

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

1. Ejecuta `dectl session end` para automatizar el cierre de sesión
2. O manualmente:
   - Escribe `.dec/state/last_session.md` con resumen
   - Ejecuta `dectl memory add "Sesión [fecha]: [resumen en una línea]"`
```

---

### `state/progress.json`

```json
{
  "_comment": "Estado de features del proyecto. Actualizar al completar tareas.",
  "schema_version": "1.0",
  "updated_at": "",
  "features": []
}
```

---

### `state/last_session.md`

```markdown
# Última Sesión
> Actualiza este archivo al finalizar cada sesión de trabajo.
> El modelo debe leerlo al inicio de una sesión nueva para retomar contexto.

**Fecha**: (sin sesiones aún)
**Qué se hizo**: —
**Qué quedó pendiente**: —
**Decisiones tomadas**: —
**Próximo paso recomendado**: Completar la inicialización del proyecto en .dec/isa/project.isa.md
```

---

### `.dec/.gitignore`

```gitignore
# dectl — archivos que no deben versionarse

# Estado local personal (no compartir con el equipo)
state/local_*.json

# Por si acaso — estos archivos NUNCA deben estar en .dec/
*.env
*.key
*.pem
*.secret
secrets.*
.env.*
```

---

## Workflow YAML — Schema Completo

```
workflow:
  name:         string, requerido — identificador único del workflow
  description:  string, requerido — mostrado en dectl workflow list
  inputs:       array opcional de InputDefinition
  steps:        array requerido de Step (mínimo 1)

InputDefinition:
  name:         string, requerido
  description:  string, requerido
  required:     boolean, requerido
  default:      string, opcional (solo si required = false)

Step:
  type:         "prompt" | "action" | "write", requerido
  description:  string, requerido — explica la intención del paso
  content:      string — requerido si type = prompt o write
  cmd:          array de strings — requerido si type = action
  path:         string — requerido si type = write
  shell:        boolean, opcional, default false — solo para type = action
```

---

## Fases de Implementación

### Fase 1 — Templates del nivel mínimo
Implementar en el CLI los templates de nivel 1: `config/project.toml`, `isa/project.isa.md`, `.gitignore`. Estos son los archivos que `dectl project init` crea por defecto.

### Fase 2 — Templates del nivel estándar
Implementar templates de nivel 2: `decisions/.gitkeep`, los dos workflows base, `prompts/system/base.md`, `state/progress.json`, `state/last_session.md`.

### Fase 3 — Templates del nivel completo
Implementar templates de nivel 3: `isa/architecture.isa.md`, los cuatro prompts de tareas, `knowledge/glossary.md`, `knowledge/constraints.md`.

### Fase 4 — Interpolación de variables
Implementar el motor de interpolación `{{variable}}` en el runner de workflows del CLI. Validación de inputs obligatorios antes de ejecutar.

---

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|-----------|
| Templates demasiado genéricos no dan valor inmediato | Alto | Diseñar templates con ejemplos concretos comentados, no placeholders vacíos |
| Developer llena `.dec/` con información sensible | Medio | `.dec/.gitignore` automático + advertencia en `dectl project init` |
| Schema de workflows cambia entre versiones del CLI | Alto | `schema_version` en `project.toml` + advertencia explícita del CLI |
| Modelos de 7B ignoran instrucciones en archivos `.dec/` | Medio | Mantener archivos bajo 2000 tokens, instrucciones al inicio del archivo |
