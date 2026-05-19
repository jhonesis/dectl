# Data Model — .dec/ System
> *Define todos los schemas de archivos de .dec/, sus campos, tipos y validaciones.*
> *Last updated: 2026-05-13*

---

## Nota sobre el modelo de datos

`.dec/` no usa una base de datos. Sus entidades son archivos estructurados en tres formatos:

| Formato | Archivos | Parseado por |
|---------|---------|-------------|
| TOML | `config/project.toml` | CLI (serde + toml) |
| JSON | `state/progress.json` | CLI (serde_json) + modelo |
| YAML | `workflows/*.yaml` | CLI (serde_yaml) |
| Markdown | ISA, decisions, prompts, knowledge, state/last_session.md | Modelo (lectura directa) |

Los archivos Markdown no tienen schema estricto — su estructura es convencional y auto-documentada mediante comentarios internos. Este documento define los schemas de los archivos parseables (TOML, JSON, YAML).

---

## Entidades TOML

### `config/project.toml`

Schema completo con tipos y validaciones:

```
[dec]
schema_version    string   requerido   Versión del schema de .dec/ (ej. "1.0")

[project]
name              string   requerido   Nombre del proyecto, kebab-case recomendado
type              string   requerido   Enum: "api" | "cli" | "microservice" |
                                       "monolith" | "library" | "other"
description       string   requerido   Descripción en una frase, máx. 200 caracteres

[stack]
languages         array    requerido   Mínimo 1 elemento. Ej: ["rust", "python"]
frameworks        array    opcional    Puede ser vacío: []
databases         array    opcional    Puede ser vacío: []
tools             array    opcional    Ej: ["docker", "git"]

[conventions]
rules             array    opcional    Strings con instrucciones para el modelo.
                                       Cada elemento es una regla en lenguaje natural.
                                       Ej: ["Commits siempre en inglés",
                                            "Type hints obligatorios en Python"]
```

**Validaciones del CLI al leer `project.toml`**:
- `schema_version` presente → si no, advertencia y sugerencia de migración
- `project.name` no vacío
- `project.type` dentro del enum válido → si no, warning (no error — forward compatibility)
- `stack.languages` con al menos un elemento

---

## Entidades JSON

### `state/progress.json`

```
{
  "_comment"       string     opcional    Nota interna, ignorada por el CLI
  "schema_version" string     requerido   Versión del schema (ej. "1.0")
  "updated_at"     string     requerido   ISO 8601. Vacío ("") en estado inicial
  "features"       array      requerido   Array de Feature. Puede ser vacío []
}

Feature:
{
  "id"      string   requerido   Identificador único, kebab-case. Ej: "user-auth"
  "name"    string   requerido   Nombre legible. Ej: "Autenticación de usuarios"
  "status"  string   requerido   Enum: "pending" | "in_progress" | "done" | "blocked"
  "notes"   string   opcional    Contexto adicional. Vacío ("") si no hay notas
}
```

**Validaciones**:
- `id` único dentro del array — el CLI advierte si hay duplicados
- `status` dentro del enum válido — valores desconocidos se preservan (forward compatibility)
- `updated_at` en formato ISO 8601 cuando no está vacío

**Ejemplo válido**:
```json
{
  "_comment": "Estado de features del proyecto. Actualizar al completar tareas.",
  "schema_version": "1.0",
  "updated_at": "2026-05-13T14:00:00Z",
  "features": [
    {
      "id": "user-auth",
      "name": "Autenticación de usuarios",
      "status": "done",
      "notes": "JWT implementado. Refresh tokens en próxima iteración."
    },
    {
      "id": "payment-processing",
      "name": "Procesamiento de pagos",
      "status": "in_progress",
      "notes": "Stripe integrado, falta webhook handler."
    },
    {
      "id": "email-notifications",
      "name": "Notificaciones por email",
      "status": "pending",
      "notes": ""
    }
  ]
}
```

---

## Entidades YAML

### `workflows/*.yaml`

Schema completo:

```
name           string          requerido   Identificador del workflow.
                                           kebab-case. Ej: "implement_feature"
                                           Debe coincidir con el nombre del archivo
                                           (sin .yaml)

description    string          requerido   Mostrado en dectl workflow list.
                                           Máx. 200 caracteres.

inputs         array<Input>    opcional    Variables de entrada del workflow.
                                           Puede omitirse si el workflow no tiene
                                           variables.

steps          array<Step>     requerido   Mínimo 1 paso.
```

**Schema de `Input`**:

```
name           string    requerido   Nombre de la variable. snake_case.
                                     Ej: "feature_name"
                                     Usado como {{feature_name}} en los pasos.

description    string    requerido   Qué representa esta variable.
                                     Incluir ejemplos ayuda al modelo y al developer.

required       boolean   requerido   true = el CLI aborta si no se provee.
                                     false = usa default si no se provee.

default        string    condicional Requerido si required = false.
                                     Ignorado si required = true.
```

**Schema de `Step`**:

```
type           string    requerido   Enum: "prompt" | "action" | "write"

description    string    requerido   Explica la intención del paso.
                                     Mostrado en dectl workflow describe.

content        string    condicional Requerido si type = "prompt" o "write".
                                     Soporta interpolación {{variable}}.
                                     En "prompt": instrucción para el modelo.
                                     En "write": contenido a escribir en el archivo.

cmd            array     condicional Requerido si type = "action".
                                     Array de strings. Primer elemento = ejecutable.
                                     Soporta interpolación {{variable}} en
                                     cualquier elemento.
                                     Ej: ["dectl", "memory", "add", "{{feature_name}}"]

path           string    condicional Requerido si type = "write".
                                     Ruta relativa al directorio del proyecto.
                                     Soporta interpolación {{variable}}.

shell          boolean   opcional    Solo para type = "action".
                                     Default: false.
                                     true = ejecuta via sh -c (habilita pipes y
                                     redirects). Requiere confirmación de trust
                                     igual que cualquier action step.
```

---

## Diagrama de Relaciones entre Archivos

```
config/project.toml
    │
    ├── schema_version ──────────────────► versiona toda la estructura .dec/
    ├── project.name ────────────────────► referenciado en prompts y workflows
    └── stack.languages ─────────────────► guía al modelo al escribir código
           │
           ▼
isa/project.isa.md ──────────────────────► leído antes de cualquier decisión
    │
    └── referencia implícita a ──────────► decisions/*.md
                                           (el modelo consulta ambos)

workflows/*.yaml
    │
    ├── inputs[].name ───────────────────► interpolado como {{name}} en steps
    ├── steps[type=action].cmd ──────────► ejecutado por CLI via std::process
    ├── steps[type=write].path ──────────► escrito por CLI al sistema de archivos
    └── steps[type=prompt].content ──────► impreso por CLI para que el modelo actúe

state/progress.json
    │
    └── features[].status ───────────────► actualizado por modelo o CLI
                                           al completar tareas

state/last_session.md
    │
    └── leído al inicio de sesión ───────► escrito al final de sesión
        por el modelo                      por el modelo o workflow write step
```

---

## Reglas de Interpolación de Variables

La interpolación aplica a todos los campos de tipo `string` en un `Step` que soporten contenido dinámico: `content`, `path`, y elementos de `cmd`.

**Sintaxis**: `{{nombre_variable}}`
- `nombre_variable` debe coincidir exactamente con un `name` en `inputs`
- Case-sensitive: `{{Feature_Name}}` ≠ `{{feature_name}}`
- Espacios no permitidos dentro de las llaves: `{{ nombre }}` no es válido
- Escape: `\{{` produce el literal `{{` sin interpolación

**Proceso de resolución**:
1. El CLI extrae todas las ocurrencias de `{{...}}` en los steps
2. Verifica que cada variable referenciada esté declarada en `inputs`
3. Verifica que todas las variables con `required: true` tengan valor provisto
4. Para variables con `required: false` sin valor provisto, usa `default`
5. Sustituye todas las ocurrencias antes de ejecutar cualquier paso

**Errores**:
- Variable en step no declarada en `inputs` → error en carga del workflow, no en ejecución
- Variable `required: true` sin valor → error antes de ejecutar el primer paso
- Variable `required: false` sin valor y sin `default` → error en carga del workflow

---

## Versionado del Schema

El campo `schema_version` en `config/project.toml` y `state/progress.json` sigue semver:

| Versión | Qué cambió | Backwards compatible |
|---------|-----------|---------------------|
| `1.0` | Schema inicial | — |
| `1.x` | Nuevos campos opcionales | ✅ Sí |
| `2.0` | Cambios breaking | ❌ No — requiere `dectl migrate` |

El CLI lee `schema_version` al abrir cualquier proyecto y:
- Versión menor o igual a la soportada → opera normalmente
- Versión minor mayor → opera con advertencia ("campos nuevos podrían ignorarse")
- Versión major mayor → aborta con instrucciones para actualizar el CLI
