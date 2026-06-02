# Technical Research — .dec/ System
> *Documenta decisiones técnicas sobre el schema, formatos y comportamientos de .dec/*
> *Last updated: 2026-06-02*

---

## Research Questions

### RQ-D-001: Sintaxis de interpolación de variables en workflows

**Context**: REQ-D-004 requiere que los workflows soporten variables de entrada con interpolación en cualquier campo de cualquier paso. La sintaxis de interpolación debe ser reconocible, fácil de escribir por un developer y por un modelo, y no colisionar con sintaxis de otros formatos (YAML, shell, Markdown).

**Options Evaluated**:

| Opción | Ejemplo | Pros | Contras |
|--------|---------|------|---------|
| `{{variable}}` (Handlebars/Jinja style) | `{{feature_name}}` | Universalmente reconocida, usada en Ansible, GitHub Actions, Jinja2, Handlebars | Puede colisionar con sintaxis de shell scripts dentro de pasos `action` |
| `${{variable}}` (GitHub Actions style) | `${{feature_name}}` | Diferenciada del shell, familiar para devs de CI/CD | Menos legible, el `$` puede confundir en contextos no-shell |
| `<variable>` (XML-like) | `<feature_name>` | Simple | Colisiona con HTML/XML en contenido de pasos `write`, ambigua |
| `%variable%` (Windows env style) | `%feature_name%` | Simple | Poco familiar en contexto Unix, no intuitiva para modelos |
| `${variable}` (shell style) | `${feature_name}` | Familiar para developers Unix | Colisión directa con variables de shell en pasos `action` |

**Decision**: `{{variable}}` (doble llave, sin signo).

**Rationale**: Es la sintaxis más reconocida en herramientas declarativas (Ansible, Jinja2, Handlebars, Liquid, Helm). Los modelos la generan correctamente sin instrucciones adicionales. La colisión potencial con shell se resuelve escapando — `\{{` produce un literal `{{` — lo cual es un comportamiento estándar y esperado. Es legible en Markdown y YAML sin ambigüedad visual.

---

### RQ-D-002: Declaración de variables en el schema YAML del workflow

**Context**: Los workflows deben declarar sus variables de entrada explícitamente (REQ-D-004) para que el CLI pueda validar que todas las obligatorias están presentes antes de ejecutar, y para que el developer o modelo entienda qué inputs requiere el workflow sin ejecutarlo.

**Options Evaluated**:

| Opción | Estructura | Pros | Contras |
|--------|-----------|------|---------|
| Sección `inputs` al nivel raíz | `inputs: [{name, description, required, default}]` | Explícita, validable, similar a GitHub Actions | Añade verbosidad al YAML |
| Variables inferidas del contenido | El CLI escanea `{{var}}` en los pasos | Sin declaración explícita necesaria | No permite documentar ni defaults; difícil distinguir obligatorias de opcionales |
| Sección `vars` simplificada | `vars: {nombre: "descripción"}` | Más compacta | Pierde información de obligatoriedad y defaults |

**Decision**: Sección `inputs` al nivel raíz con estructura completa.

**Rationale**: La declaración explícita es la única opción que satisface todos los criterios de aceptación de REQ-D-004 — especialmente la validación de obligatorias y la auto-documentación del workflow. Un modelo que lee el workflow entiende exactamente qué necesita para ejecutarlo. El CLI puede validar sin ejecutar. La verbosidad es un trade-off aceptable dado que los workflows son documentos de intención, no código de producción.

**Schema resultante**:
```yaml
name: implement_feature
description: Implementa una nueva feature completa con tests y documentación
inputs:
  - name: feature_name
    description: Nombre de la feature a implementar (ej. "user_authentication")
    required: true
  - name: module
    description: Módulo donde se implementará la feature
    required: true
  - name: include_tests
    description: Si se deben generar tests automáticamente
    required: false
    default: "true"
steps:
  - type: prompt
    content: |
      Implementa la feature "{{feature_name}}" en el módulo "{{module}}".
      Include tests: {{include_tests}}.
```

---

### RQ-D-003: Formato del archivo de decisiones (ADR)

**Context**: REQ-D-003 requiere un formato para registros de decisiones que incluya contexto, decisión, alternativas, justificación y consecuencias. Existe el estándar ADR (Architecture Decision Record) ampliamente adoptado. La pregunta es si usarlo tal cual, adaptarlo o diseñar uno propio.

**Options Evaluated**:

| Opción | Pros | Contras |
|--------|------|---------|
| ADR estándar completo (Nygard) | Conocido, herramientas existentes, compatible con adr-tools | Incluye secciones como "Status" con estados complejos (Proposed/Accepted/Deprecated/Superseded) que pueden confundir |
| ADR simplificado propio | Adaptado a modelos pequeños, más directo | No compatible con herramientas ADR existentes |
| MADR (Markdown ADR) | Más estructurado, popular en proyectos modernos | Más verboso, headers más complejos |

**Decision**: ADR simplificado propio, inspirado en MADR pero optimizado para modelos de 7B+.

**Rationale**: El ADR estándar tiene estados complejos (Superseded by, etc.) que requieren mantener referencias cruzadas — innecesario para proyectos individuales o equipos pequeños. El formato propio puede ser más directo y auto-explicativo. Se mantiene compatibilidad conceptual con ADR (mismas secciones core) para que developers familiarizados lo reconozcan.

**Schema resultante**:
```markdown
# [0001] Título de la decisión

**Fecha**: YYYY-MM-DD
**Estado**: activa | obsoleta | supersedida por [0002]

## Contexto
Qué situación o problema motivó esta decisión.

## Decisión
La decisión tomada, en una frase clara.

## Alternativas consideradas
- Opción A: descripción y por qué se descartó
- Opción B: descripción y por qué se descartó

## Justificación
Por qué esta decisión es mejor que las alternativas.

## Consecuencias
Qué cambia, qué se gana, qué se sacrifica.
```

---

### RQ-D-004: Formato del ISA (Ideal State Artifact)

**Context**: REQ-D-002 requiere documentos ISA que describan el estado ideal del proyecto. El ISA es el concepto más importante heredado del PAI original, pero necesita ser rediseñado para modelos pequeños — el ISA original de PAI asume modelos con ventanas de contexto grandes y capacidad de razonamiento en 7 fases.

**Options Evaluated**:

| Opción | Pros | Contras |
|--------|------|---------|
| ISA original de PAI (7 fases, filosófico) | Probado con Claude | Demasiado largo, dependiente de razonamiento avanzado, confuso para 7B |
| Documento libre sin estructura | Máxima flexibilidad | Sin consistencia entre proyectos, difícil de consumir por el modelo |
| ISA estructurado con secciones fijas y guiadas | Consistente, auto-documentado, funciona con 7B | Menos flexible |

**Decision**: ISA estructurado con secciones fijas, guiado, optimizado para 7B+.

**Rationale**: La estructura fija garantiza que cualquier modelo, independientemente de su capacidad, encuentre la información que busca en el lugar esperado. Las secciones guiadas (con instrucciones dentro del template) eliminan la curva de aprendizaje. El tamaño máximo de 2000 tokens iniciales (mandatado por constitution.md) fuerza concisión — que es precisamente lo que los modelos pequeños necesitan.

**Schema de `project.isa.md`**:
```markdown
# ISA: [Nombre del Proyecto]
> El modelo debe leer este documento antes de tomar cualquier decisión importante.
> Actualizar cuando la visión o el alcance cambien significativamente.

## Visión
Una frase que describe qué es el proyecto y para quién.

## Objetivo principal
Qué problema concreto resuelve y cómo mide el éxito.

## Alcance (qué SÍ incluye)
- Item 1
- Item 2

## No-objetivos (qué NO incluye)
- Item 1
- Item 2

## Stack tecnológico
Lenguajes, frameworks, bases de datos y herramientas principales.

## Restricciones conocidas
Limitaciones técnicas, de tiempo o de recursos que el modelo debe respetar.

## Riesgos principales
Los 2-3 riesgos más importantes del proyecto.
```

---

### RQ-D-005: Manejo de `.dec/` en git

**Context**: Constitution.md establece que `.dec/` debe ser commitable sin riesgo. Pero hay archivos dentro de `.dec/` que podrían contener información sensible si el developer no tiene cuidado (por ejemplo, un workflow con una API key hardcodeada). La pregunta es cómo proteger al developer sin forzar decisiones.

**Options Evaluated**:

| Opción | Pros | Contras |
|--------|------|---------|
| `.dec/` completamente en git por defecto | Simple, fomenta compartir contexto en equipo | Riesgo si el developer pone datos sensibles |
| `.dec/` en `.gitignore` por defecto | Seguro | Pierde el valor de compartir contexto en equipo |
| `.dec/` en git con `.dec/.gitignore` interno que excluye subcarpetas sensibles | Balance entre seguridad y utilidad | Más complejo de comunicar |

**Decision**: `.dec/` en git por defecto, con un `.dec/.gitignore` generado automáticamente que excluye patrones comunes de datos sensibles.

**Rationale**: El valor del proyecto se multiplica cuando `.dec/` se versiona — el equipo comparte contexto, la historia de decisiones se preserva, y el onboarding de nuevos miembros es instantáneo. El `.dec/.gitignore` automático protege contra errores comunes sin quitar flexibilidad. Constitution.md ya prohíbe explícitamente secretos en `.dec/` — el `.gitignore` es una red de seguridad adicional.

**Contenido del `.dec/.gitignore` generado**:
```
# Archivos de estado local (no compartir en equipo)
state/local_*.json

# Por si acaso — nunca deben estar aquí
*.env
*.key
*.pem
secrets.*
```

---

## Resumen de Decisiones

| ID | Decisión | Resultado |
|----|---------|-----------|
| RQ-D-001 | Sintaxis de variables | `{{variable}}` con escape `\{{` |
| RQ-D-002 | Declaración de inputs | Sección `inputs:` explícita en YAML |
| RQ-D-003 | Formato ADR | ADR simplificado propio, inspirado en MADR |
| RQ-D-004 | Formato ISA | Estructurado, secciones fijas, máx. 2000 tokens |
| RQ-D-005 | Git handling | En git por defecto + `.dec/.gitignore` automático |
