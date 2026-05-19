# Data Model — dectl CLI
> *Define los tipos Rust internos del CLI y el contrato exacto de todos los JSON outputs.*
> *La persistencia SQLite está en master/data-model.md.*
> *Los schemas de archivos .dec/ están en dot-dec/data-model.md.*
> *Last updated: 2026-05-13*

---

## Organización

Este documento cubre tres categorías:

1. **Structs de configuración** — lo que el CLI lee de TOML al arrancar
2. **Structs de dominio** — tipos internos que viajan entre capas
3. **JSON output shapes** — el contrato exacto de `--json` para cada comando

---

## 1. Structs de Configuración

### `GlobalConfig` (de `~/.dectl/config.toml`)

```rust
#[derive(Deserialize, Default)]
pub struct GlobalConfig {
    pub core:     CoreConfig,
    pub memory:   MemoryConfig,
    pub workflow: WorkflowConfig,
}

#[derive(Deserialize)]
pub struct CoreConfig {
    #[serde(default = "default_editor")]
    pub default_editor: String,  // default: "vim"
    #[serde(default = "default_true")]
    pub color: bool,             // default: true
}

#[derive(Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_db_path")]
    pub db_path: String,         // default: "~/.dectl/memory.db"
    #[serde(default = "default_max_results")]
    pub max_results: usize,      // default: 20
}

#[derive(Deserialize)]
pub struct WorkflowConfig {
    #[serde(default = "default_trust_path")]
    pub trust_path: String,      // default: "~/.dectl/trust.toml"
}
```

---

### `ProjectConfig` (de `.dec/config/project.toml`)

```rust
#[derive(Deserialize)]
pub struct ProjectConfig {
    pub dec:         DecMeta,
    pub project:     ProjectMeta,
    pub stack:       StackConfig,
    #[serde(default)]
    pub conventions: ConventionsConfig,
}

#[derive(Deserialize)]
pub struct DecMeta {
    pub schema_version: String,  // ej. "1.0"
}

#[derive(Deserialize)]
pub struct ProjectMeta {
    pub name:        String,
    pub r#type:      ProjectType,
    pub description: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Api, Cli, Microservice, Monolith, Library, Other,
}

#[derive(Deserialize)]
pub struct StackConfig {
    pub languages:  Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
    #[serde(default)]
    pub databases:  Vec<String>,
    #[serde(default)]
    pub tools:      Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct ConventionsConfig {
    #[serde(default)]
    pub rules: Vec<String>,
}
```

---

## 2. Structs de Dominio

### Memory

```rust
// Entrada de memoria — mapea la tabla SQLite
#[derive(Serialize)]
pub struct MemoryEntry {
    pub id:         i64,
    pub content:    String,
    pub tags:       Option<String>,
    pub project:    Option<String>,
    pub created_at: String,   // ISO 8601
    pub updated_at: Option<String>,
}

// Vista resumida para list/search
#[derive(Serialize)]
pub struct MemoryPreview {
    pub id:         i64,
    pub created_at: String,
    pub tags:       Option<String>,
    pub project:    Option<String>,
    pub preview:    String,   // primeros 100 chars del content
}
```

---

### Workflow

```rust
#[derive(Deserialize, Serialize)]
pub struct Workflow {
    pub name:        String,
    pub description: String,
    #[serde(default)]
    pub inputs:      Vec<InputDef>,
    pub steps:       Vec<Step>,
}

#[derive(Deserialize, Serialize)]
pub struct InputDef {
    pub name:        String,
    pub description: String,
    pub required:    bool,
    pub default:     Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Step {
    pub r#type:      StepType,
    pub description: String,
    pub content:     Option<String>,  // prompt, write
    pub cmd:         Option<Vec<String>>,  // action
    pub path:        Option<String>,  // write
    #[serde(default)]
    pub shell:       bool,            // action — default false
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    Prompt,
    Action,
    Write,
}

// Estado de un step durante ejecución
pub enum StepStatus {
    Skipped,
    Executed,
    Failed { stderr: String, exit_code: Option<i32> },
}
```

---

### Project

```rust
// Resultado del scan
pub struct FileNode {
    pub path:     String,
    pub is_dir:   bool,
    pub children: Vec<FileNode>,  // vacío si is_dir = false
}

// Resumen de features para project info
#[derive(Serialize)]
pub struct ProgressSummary {
    pub done:        usize,
    pub in_progress: usize,
    pub pending:     usize,
    pub blocked:     usize,
    pub total:       usize,
}
```

---

## 3. JSON Output Shapes

Todos los outputs JSON siguen el envelope:
```json
{"status": "ok" | "error", ...campos específicos}
```

Los campos de error siempre incluyen `message` y opcionalmente `hint`.

---

### `dectl project init --json`

**Éxito**:
```json
{
  "status": "ok",
  "level": 1,
  "files_created": [
    ".dec/.gitignore",
    ".dec/config/project.toml",
    ".dec/isa/project.isa.md"
  ],
  "next_step": "Edit .dec/config/project.toml and .dec/isa/project.isa.md with your project details"
}
```

**Error** (`.dec/` ya existe):
```json
{
  "status": "error",
  "message": ".dec/ already exists in this directory",
  "hint": "Delete .dec/ first or use an empty directory"
}
```

---

### `dectl project info --json`

```json
{
  "status": "ok",
  "project": {
    "name": "my-api",
    "type": "api",
    "description": "REST API for inventory management",
    "schema_version": "1.0"
  },
  "stack": {
    "languages": ["python"],
    "frameworks": ["fastapi"],
    "databases": ["postgresql"],
    "tools": ["docker"]
  },
  "isa_vision": "A REST API to manage inventory for small stores.",
  "progress": {
    "done": 2,
    "in_progress": 1,
    "pending": 3,
    "blocked": 0,
    "total": 6
  },
  "warnings": []
}
```

---

### `dectl project scan --json`

```json
{
  "status": "ok",
  "tree": [
    {
      "path": "src",
      "type": "dir",
      "children": [
        { "path": "src/main.rs", "type": "file", "children": [] },
        { "path": "src/lib.rs",  "type": "file", "children": [] }
      ]
    },
    { "path": "Cargo.toml", "type": "file", "children": [] }
  ]
}
```

---

### `dectl memory add --json`

**Éxito**:
```json
{
  "status": "ok",
  "id": 42,
  "project": "my-api",
  "preview": "Decisión: usar PostgreSQL por soporte de JSONB y familiaridad del equipo"
}
```

---

### `dectl memory list --json`

```json
{
  "status": "ok",
  "entries": [
    {
      "id": 42,
      "created_at": "2026-05-13T14:32:00Z",
      "tags": "architecture,database",
      "project": "my-api",
      "preview": "Decisión: usar PostgreSQL por soporte de JSONB..."
    }
  ],
  "total": 1
}
```

---

### `dectl memory search --json`

```json
{
  "status": "ok",
  "query": "postgresql",
  "entries": [
    {
      "id": 42,
      "created_at": "2026-05-13T14:32:00Z",
      "tags": "architecture,database",
      "project": "my-api",
      "preview": "Decisión: usar **PostgreSQL** por soporte de JSONB..."
    }
  ],
  "total": 1
}
```

---

### `dectl memory show --json`

```json
{
  "status": "ok",
  "entry": {
    "id": 42,
    "content": "# Decisión: Base de datos\n\nUsar PostgreSQL por...",
    "tags": "architecture,database",
    "project": "my-api",
    "created_at": "2026-05-13T14:32:00Z",
    "updated_at": null
  }
}
```

---

### `dectl workflow list --json`

```json
{
  "status": "ok",
  "workflows": [
    {
      "name": "implement_feature",
      "description": "Implementa una nueva feature completa con tests y documentación",
      "required_inputs": ["feature_name", "module"]
    },
    {
      "name": "design_architecture",
      "description": "Guía al modelo para diseñar o revisar la arquitectura del proyecto",
      "required_inputs": []
    }
  ]
}
```

---

### `dectl workflow describe --json`

```json
{
  "status": "ok",
  "workflow": {
    "name": "implement_feature",
    "description": "Implementa una nueva feature completa con tests y documentación",
    "inputs": [
      {
        "name": "feature_name",
        "description": "Nombre de la feature (ej. 'user_authentication')",
        "required": true,
        "default": null
      },
      {
        "name": "include_tests",
        "description": "Generar tests automáticamente (true/false)",
        "required": false,
        "default": "true"
      }
    ],
    "steps": [
      {
        "index": 1,
        "type": "prompt",
        "description": "Cargar contexto del proyecto",
        "content": "Lee .dec/isa/project.isa.md y .dec/config/project.toml..."
      },
      {
        "index": 2,
        "type": "action",
        "description": "Buscar decisiones relevantes en memoria",
        "cmd": ["dectl", "memory", "search", "{{feature_name}}"]
      }
    ]
  }
}
```

---

### `dectl workflow run --json`

**Éxito**:
```json
{
  "status": "ok",
  "workflow": "implement_feature",
  "steps_executed": 5,
  "steps_skipped": 0,
  "failed_step": null
}
```

**Error en un paso**:
```json
{
  "status": "error",
  "workflow": "implement_feature",
  "steps_executed": 2,
  "steps_skipped": 0,
  "failed_step": {
    "index": 3,
    "type": "action",
    "description": "Buscar decisiones relevantes",
    "cmd": ["dectl", "memory", "search", "user_auth"],
    "exit_code": 1,
    "stderr": "No se encontró .dec/ en el directorio actual"
  },
  "message": "Step 3 failed",
  "hint": "Fix the error above and resume with --from-step 3"
}
```

---

### `dectl exec-from-file --json`

**Éxito**:
```json
{
  "status": "ok",
  "executed": 5,
  "failed_line": null,
  "failed_cmd": null
}
```

**Error**:
```json
{
  "status": "error",
  "executed": 2,
  "failed_line": 3,
  "failed_cmd": "dectl memory add",
  "message": "Command failed at line 3",
  "hint": "Fix the command and re-run from line 3"
}
```

---

## Relaciones con Otros Data Models

```
CLI Structs
    │
    ├── lee ──────────────────► GlobalConfig (~/.dectl/config.toml)
    │                            WorkflowConfig → trust_path
    │                            MemoryConfig → db_path
    │
    ├── lee ──────────────────► ProjectConfig (.dec/config/project.toml)
    │                            [definido en dot-dec/data-model.md]
    │
    ├── lee/escribe ──────────► SQLite memories + embeddings
    │                            [definido en master/data-model.md]
    │
    ├── lee ──────────────────► Workflow YAML (.dec/workflows/*.yaml)
    │                            [definido en dot-dec/data-model.md]
    │
    └── lee/escribe ──────────► progress.json + last_session.md
                                 [definido en dot-dec/data-model.md]
```
