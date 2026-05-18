use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum InitLevel {
    Level1,
    Level2,
    Level3,
}

pub struct Templates;

impl Templates {
    pub fn level1() -> Vec<(&'static str, &'static str)> {
        vec![
            (".dec/.gitignore", Self::GITIGNORE_L1),
            (".dec/config/project.toml", Self::PROJECT_TOML_L1),
            (".dec/isa/project.isa.md", Self::PROJECT_ISA),
        ]
    }

    pub fn level2() -> Vec<(&'static str, &'static str)> {
        let mut files = Self::level1();
        files.extend([
            (".dec/decisions/.gitkeep", ""),
            (
                ".dec/workflows/implement_feature.yaml",
                Self::WORKFLOW_IMPLEMENT_FEATURE,
            ),
            (
                ".dec/workflows/design_architecture.yaml",
                Self::WORKFLOW_DESIGN_ARCHITECTURE,
            ),
            (".dec/prompts/system/base.md", Self::SYSTEM_BASE),
            (".dec/state/progress.json", Self::PROGRESS_JSON),
            (".dec/state/last_session.md", Self::LAST_SESSION),
        ]);
        files
    }

    pub fn level3() -> Vec<(&'static str, &'static str)> {
        let mut files = Self::level2();
        files.extend([
            (".dec/isa/architecture.isa.md", Self::ARCHITECTURE_ISA),
            (
                ".dec/prompts/tasks/implement_feature.md",
                Self::TASK_IMPLEMENT_FEATURE,
            ),
            (".dec/prompts/tasks/write_tests.md", Self::TASK_WRITE_TESTS),
            (".dec/prompts/tasks/review_code.md", Self::TASK_REVIEW_CODE),
            (
                ".dec/prompts/tasks/document_module.md",
                Self::TASK_DOCUMENT_MODULE,
            ),
            (".dec/knowledge/glossary.md", Self::KNOWLEDGE_GLOSSARY),
            (".dec/knowledge/constraints.md", Self::KNOWLEDGE_CONSTRAINTS),
        ]);
        files
    }

    pub fn files_for_level(level: InitLevel) -> Vec<(&'static str, &'static str)> {
        match level {
            InitLevel::Level1 => Self::level1(),
            InitLevel::Level2 => Self::level2(),
            InitLevel::Level3 => Self::level3(),
        }
    }

    const GITIGNORE_L1: &str = r#"# dectl — archivos que no deben versionarse

# Estado local personal (no compartir con el equipo)
state/local_*.json

# Por si acaso — estos archivos NUNCA deben estar en .dec/
*.env
*.key
*.pem
*.secret
secrets.*
.env.*
"#;

    const PROJECT_TOML_L1: &str = r#"# dectl project configuration
# Este archivo define el proyecto para el modelo y las herramientas.
# El modelo debe leerlo al inicio de cada sesión.

[dec]
schema_version = "1.0"

[project]
name = "nombre-del-proyecto"
# Tipo de proyecto: api | cli | microservice | monolith | library | other
type = "other"
description = "Descripción breve del proyecto en una frase."

[stack]
# Lista de tecnologías principales del proyecto
languages = []
frameworks = []
databases = []
tools = []

[conventions]
# Convenciones especiales que el modelo debe seguir en este proyecto.
rules = []
"#;

    const PROJECT_ISA: &str = r#"# ISA: [Nombre del Proyecto]
> **Para el modelo**: Lee este documento antes de tomar cualquier decisión importante.
> Actualiza este archivo cuando la visión o el alcance cambien significativamente.
> Si algo aquí contradice lo que ves en el código, pregunta al developer antes de asumir.

---

## Visión
<!-- Una frase: qué es el proyecto y para quién. -->


## Objetivo Principal
<!-- Qué problema concreto resuelve y cómo se mide el éxito. -->


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


## Riesgos Principales
<!-- Los 2-3 riesgos más importantes. Breve. -->
1.
2.
"#;

    const WORKFLOW_IMPLEMENT_FEATURE: &str = r#"name: implement_feature
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
"#;

    const WORKFLOW_DESIGN_ARCHITECTURE: &str = r#"name: design_architecture
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
"#;

    const SYSTEM_BASE: &str = r#"# System Prompt Base — [Nombre del Proyecto]
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
"#;

    const PROGRESS_JSON: &str = r#"{
  "_comment": "Estado de features del proyecto. Actualizar al completar tareas.",
  "schema_version": "1.0",
  "updated_at": "",
  "features": []
}
"#;

    const LAST_SESSION: &str = r#"# Última Sesión
> Actualiza este archivo al finalizar cada sesión de trabajo.
> El modelo debe leerlo al inicio de una sesión nueva para retomar contexto.

**Fecha**: (sin sesiones aún)
**Qué se hizo**: —
**Qué quedó pendiente**: —
**Decisiones tomadas**: —
**Próximo paso recomendado**: Completar la inicialización del proyecto en .dec/isa/project.isa.md
"#;

    const ARCHITECTURE_ISA: &str = r#"# ISA: Arquitectura — [Nombre del Proyecto]
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
"#;

    const TASK_IMPLEMENT_FEATURE: &str = r#"# Prompt: Implementar Feature

## Contexto
Estás implementando una nueva feature. Lee `.dec/isa/project.isa.md` y `.dec/config/project.toml` primero.

## Tu tarea
1. Lee `.dec/decisions/` para entender restricciones arquitectónicas
2. Diseña la implementación brevemente antes de escribir código
3. Implementa la feature siguiendo las convenciones del proyecto
4. Si `include_tests` es true, genera tests para la nueva funcionalidad
5. Confirma que el código compila y pasa lint

## Restricciones
- Sigue las convenciones en `config/project.toml` → `[conventions]`
- No modifiques archivos fuera del módulo asignado sin approval
- Consulta `.dec/decisions/` antes de tomar decisiones arquitectónicas

## Al terminar
- Ejecuta `dectl memory add` con un resumen de lo que hiciste
- Actualiza `.dec/state/progress.json` si la feature está completa
"#;

    const TASK_WRITE_TESTS: &str = r#"# Prompt: Escribir Tests

## Contexto
Debes escribir tests para una funcionalidad existente. Lee el módulo primero.

## Tu tarea
1. Identifica qué funcionalidades necesitan tests
2. Escribe tests que cubran casos normales y edge cases
3. Sigue el framework de testing del proyecto (ver `config/project.toml`)
4. Ejecuta los tests para confirmar que pasan

## Restricciones
- Tests deben ser independientes y poder ejecutarse en cualquier orden
- No hardcodear paths — usar variables de entorno o configuración
- Cobertura mínima: happy path + casos de error principales

## Al terminar
- Ejecuta todos los tests del módulo para confirmar que no rompiste nada
- Registra con `dectl memory add` qué tests añadiste
"#;

    const TASK_REVIEW_CODE: &str = r#"# Prompt: Code Review

## Contexto
Debes hacer review de código cambios o propuesto. Enfócate en calidad, no en estilo.

## Tu tarea
1. Lee el código propuesto o los cambios recentos
2. Identifica:
   - Bugs potenciales o casos de borde no manejados
   - Problemas de seguridad
   - Violaciones de las convenciones del proyecto
   - Mejores oportunidades de mejora
3. Proporciona feedback constructivo con ejemplos específicos

## Qué buscar
- Errores lógicos o de null handling
- Performance issues obvios
- Violaciones del architecture decisions/
- Falta de tests en código crítico

## Al terminar
- Registra con `dectl memory add` un resumen del review
- Si hay issues críticos, propón soluciones específicas
"#;

    const TASK_DOCUMENT_MODULE: &str = r#"# Prompt: Documentar Módulo

## Contexto
Debes documentar un módulo existente del proyecto.

## Tu tarea
1. Lee el código del módulo completo
2. Identifica:
   - La responsabilidad principal del módulo
   - Las funciones/métodos públicos y sus contratos
   - Dependencias y side effects
3. Escribe documentación clara:
   - README.md en la carpeta del módulo o sección en docs/
   - Comentarios doc (/// en Rust, docstring en Python)
   - Ejemplos de uso donde sea útil

## Restricciones
- Documentación debe ser útil para alguien que no escribió el código
- No documentar el qué (el código ya lo dice), sino el por qué y el cómo
- Mantener documentación cerca del código (comments, docstrings)

## Al terminar
- Registra con `dectl memory add` qué documentaste
"#;

    const KNOWLEDGE_GLOSSARY: &str = r#"# Glosario del Proyecto
> **Para el modelo**: Define aquí términos del dominio que son específicos del proyecto.
> Usa estos términos consistentemente. Si necesitas añadir uno, consulta al developer primero.

---

## Términos

### [término]
Definición breve. Una frase.

### [término]
Definición breve. Una frase.

---

## Acrónimos

| Acrónimo | Significado |
|----------|-------------|
|          |             |
"#;

    const KNOWLEDGE_CONSTRAINTS: &str = r#"# Restricciones del Proyecto
> **Para el modelo**: Estas restricciones deben respetarse en todo momento.
> Si una restricción cambia, actualiza este archivo y notifica al team.

---

## Restricciones Técnicas

### [título]
Descripción de la restricción y por qué existe.
- Límite: [ejemplo]
- Impacto: [qué limitaciones impone al código]

---

## Restricciones de Negocio

### [título]
Descripción de la restricción de negocio.
- Límite: [ejemplo]
- Impacto: [qué limitaciones impone]

---

## Convenciones Obligatorias

- [ ] Regla 1
- [ ] Regla 2
- [ ] Regla 3
"#;
}
