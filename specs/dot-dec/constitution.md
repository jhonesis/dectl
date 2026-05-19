# Constitution — .dec/ System
> *Principios supremos que gobiernan el diseño, evolución y compatibilidad de la estructura .dec/.*
> *Hereda y extiende: specs/master/constitution.md*
> *Este documento es la ley suprema para cualquier decisión sobre .dec/. Todos los demás documentos de este SDD deben respetarlo.*

---

## 1. Identidad

`.dec/` es el corazón del sistema dectl. Es el producto principal — no el CLI.

Es un sistema de contexto estructurado que vive dentro de cada proyecto y define:
- qué es el proyecto y hacia dónde va
- qué decisiones se han tomado y por qué
- cómo debe comportarse el modelo dentro del proyecto
- qué workflows guían las tareas complejas
- cuál es el estado actual del proyecto

`.dec/` debe ser útil **sin el CLI instalado**. Cualquier modelo que pueda leer archivos puede consumirlo directamente.

---

## 2. Principios Fundamentales

**1. Schema público e inmutable una vez publicado**
El schema de `.dec/` es un contrato con la comunidad. Una vez que un campo o archivo es parte del schema oficial, no puede eliminarse ni renombrarse en versiones mayores sin un proceso de deprecación explícito. Los cambios breaking destruyen la confianza.

**2. Mínimo viable por defecto**
`dectl project init` crea solo los archivos estrictamente necesarios para ser útil. La complejidad es opt-in. Un developer no debe sentirse abrumado al abrir un proyecto con `.dec/` por primera vez.

**3. Legible por humanos y modelos por igual**
Cada archivo debe ser comprensible sin herramientas adicionales. Un developer debe poder leerlo con `cat`. Un modelo debe poder parsearlo sin instrucciones especiales. Nunca formatos binarios, nunca estructuras que requieran un intérprete para tener sentido.

**4. Portable y reproducible**
Un proyecto con `.dec/` debe funcionar idénticamente en cualquier máquina con dectl instalado. Nada en `.dec/` debe depender del entorno local. Los datos personales y la memoria van en `~/.dectl/`, nunca en `.dec/`.

**5. Extensible sin breaking changes**
Nuevos campos opcionales pueden agregarse en cualquier momento. Campos obligatorios solo pueden agregarse en versiones mayores del schema con herramienta de migración incluida.

**6. El modelo es ciudadano de primera clase**
Cada archivo de `.dec/` debe incluir comentarios o estructura que guíe al modelo sobre cómo usarlo. No se asume que el modelo conoce el sistema — los archivos se explican a sí mismos.

---

## 3. Reglas de Formato

| Tipo de archivo | Formato | Razón |
|----------------|---------|-------|
| Documentos de contexto (ISA, decisiones, prompts, knowledge) | Markdown | Legible por humanos y modelos sin tooling |
| Configuración del proyecto | TOML | Parseable, tipado, familiar en ecosistema Rust |
| Workflows | YAML | Declarativo, legible, estándar para definición de pasos |
| Estado del proyecto | JSON | Fácil de parsear por el CLI y por modelos |

**Prohibido en `.dec/`:**
- Archivos binarios de cualquier tipo
- Formatos propietarios (`.docx`, `.xlsx`, etc.)
- Symlinks
- Archivos generados automáticamente sin indicación explícita de que son generados
- Secretos, tokens, contraseñas o cualquier credencial

---

## 4. Versionado del Schema

`.dec/` usa versionado semántico en `config/project.toml`:

```toml
[dec]
schema_version = "1.0"
```

**Reglas de versionado:**
- `PATCH` (1.0.x): correcciones de documentación del schema, sin cambios estructurales
- `MINOR` (1.x.0): nuevos archivos o campos opcionales — siempre backwards compatible
- `MAJOR` (x.0.0): cambios breaking — requieren herramienta de migración `dectl migrate`

`dectl` siempre lee el `schema_version` y advierte si la versión del proyecto es mayor a la que soporta el binario instalado.

---

## 5. Estructura de Niveles

`.dec/` tiene tres niveles de adopción. Cada nivel es un superset del anterior:

**Nivel 1 — Mínimo** (creado por `dectl project init` por defecto):
```
.dec/
├── config/project.toml
└── isa/project.isa.md
```
Con esto, cualquier modelo tiene suficiente contexto para entender qué es el proyecto.

**Nivel 2 — Estándar** (creado por `dectl project init --standard`):
Agrega decisions/, workflows/, prompts/system/, state/progress.json

**Nivel 3 — Completo** (creado por `dectl project init --full`):
Agrega knowledge/, prompts/tasks/, isa/architecture.isa.md, workflows avanzados

---

## 6. Compatibilidad con Modelos

`.dec/` debe funcionar correctamente con modelos de 7B parámetros o superiores. Esto implica:

- Ningún archivo individual debe exceder 2000 tokens en su estado inicial (los archivos crecen con el uso, pero los templates deben ser concisos)
- Cada archivo debe comenzar con un comentario o encabezado que explique su propósito y cómo el modelo debe usarlo
- Los workflows deben tener pasos atómicos y claros — sin ambigüedad sobre qué hacer en cada paso
- Las instrucciones en prompts deben ser directas, sin jerga filosófica

---

## 7. Separación Personal / Compartido

| Dato | Dónde vive | Razón |
|------|-----------|-------|
| Visión del proyecto, ISA, arquitectura | `.dec/` (git) | Compartido con el equipo |
| Decisiones técnicas | `.dec/` (git) | Compartido con el equipo |
| Workflows del proyecto | `.dec/` (git) | Compartido con el equipo |
| Preferencias personales del developer | `~/.dectl/` | Personal, no se versiona |
| Memoria global | `~/.dectl/memory.db` | Personal, no se versiona |
| Trust registry | `~/.dectl/trust.toml` | Personal, no se versiona |

`.dec/` debe ser commitable a git sin riesgo. El `.gitignore` del proyecto decide si incluirlo o no — dectl no fuerza ninguna decisión.

---

## 8. Definition of Done para cambios en .dec/

Un cambio al schema de `.dec/` está completo cuando:
- [ ] El campo/archivo está documentado en `specs/dot-dec/spec.md`
- [ ] El template correspondiente está actualizado en el código del CLI
- [ ] El `schema_version` está bumpeado correctamente
- [ ] Si es breaking: herramienta de migración implementada y testeada
- [ ] Si es nuevo archivo: el archivo se explica a sí mismo (incluye comentario o encabezado de propósito)
- [ ] `CLAUDE.md` del proyecto dectl actualizado si hay nuevos archivos referenciables
