# Data Model — dectl CLI
> *Define los tipos Rust internos del CLI y el contrato exacto de todos los JSON outputs.*
> *La persistencia SQLite está en master/data-model.md.*
> *Los schemas de archivos .dec/ están en dot-dec/data-model.md.*
> *Last updated: 2026-06-02*

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
    Agent,
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

### Session (Phase 4)

```rust
// Resumen de sesión generado al cerrar
#[derive(Serialize)]
pub struct SessionSummary {
    pub date:       String,          // ISO 8601
    pub actions:    Vec<String>,     // acciones realizadas
    pub pending:    Vec<String>,     // items pendientes
    pub decisions:  Vec<String>,     // decisiones tomadas
    pub next_step:  String,          // próximo paso recomendado
}

// Cambios detectados en git
#[derive(Serialize)]
pub struct GitChanges {
    pub modified_files:    Vec<(String, String)>,  // (status, path)
    pub new_commits:       Vec<String>,            // mensajes de commit
    pub detected_features: Vec<String>,            // features de commits
}

// Decisión capturada
#[derive(Serialize)]
pub struct CapturedDecision {
    pub text:            String,
    pub tags:            Vec<String>,
    pub already_exists:  bool,
}

// Resultado de cada paso del session end
#[derive(Serialize)]
pub struct StepResult {
    pub name:     String,
    pub success:  bool,
    pub message:  String,
}

// Resultado completo del session end
#[derive(Serialize)]
pub struct SessionEndResult {
    pub steps:             Vec<StepResult>,
    pub decisions_saved:   usize,
}
```

---

## 3. JSON Output Shapes

Todos los outputs JSON con `--json` siguen el envelope:

```json
{"status": "ok", "data": { ... }}
{"status": "error", "message": "...", "hint": "..."}
```

Sin `--json` pero con `--format json`, el envelope NO se usa (solo el contenido de `data` se imprime directamente).

Los campos de error siempre incluyen `message` y opcionalmente `hint`.

---

### `dectl project init --json`

**Éxito**:
```json
{
  "status": "ok",
  "data": {
    "level": 1,
    "files_created": [
      ".dec/.gitignore",
      ".dec/config/project.toml",
      ".dec/isa/project.isa.md"
    ],
    "next_step": "Edit .dec/config/project.toml and .dec/isa/project.isa.md with your project details"
  }
}
```

**Éxito con auto-fill** (proyecto no vacío):
```json
{
  "status": "ok",
  "data": {
    "level": 2,
    "files_created": [".dec/config/project.toml", "..."],
    "auto_fill": {
      "detected_stack": { "languages": ["Rust"] },
      "filled_files": ["project.toml", "project.isa.md"]
    },
    "next_step": "Run 'dectl project info' to verify the setup"
  }
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
  "data": {
    "name": "my-api",
    "project_type": "api",
    "description": "REST API for inventory management",
    "stack": {
      "languages": ["rust"],
      "frameworks": [],
      "databases": [],
      "tools": []
    },
    "conventions": [],
    "warnings": [],
    "isa": {
      "vision": "A REST API to manage inventory...",
      "objective": null,
      "path": ".dec/isa/project.isa.md"
    }
  }
}
```

---

### `dectl project context --format json` (sin `--json`)

```json
{
  "project": "my-project",
  "tokens_used": 328,
  "tokens_limit": 4000,
  "files": [
    {
      "path": "isa/project.isa.md",
      "content": "# ISA: [Project Name]\n...",
      "tokens": 195
    },
    {
      "path": "config/project.toml",
      "content": "[project]\nname = \"...\"",
      "tokens": 35
    }
  ]
}
```

### `dectl project context --format json --json` (con envelope)

```json
{
  "status": "ok",
  "data": {
    "project": "my-project",
    "tokens_used": 328,
    "tokens_limit": 4000,
    "files": [
      { "path": "isa/project.isa.md", "content": "...", "tokens": 195 }
    ]
  }
}
```

### `dectl project context --format compact --json`

```json
{
  "status": "ok",
  "data": {
    "project": "my-project",
    "stack": "Rust, Axum",
    "last_session": "Implementada auth JWT. Pendiente refresh token.",
    "progress": "5/10 features complete",
    "decisions": "001-db-choice, 002-api-design",
    "memory_hits": 42
  }
}
```

Modo human (sin `--json`):
```
project: my-project
stack: Rust, Axum
last_session: Implementada auth JWT. Pendiente refresh token.
progress: 5/10 features complete
decisions: 001-db-choice, 002-api-design
memory_hits: 42
```

---

### `dectl project scan --json`

```json
{
  "status": "ok",
  "data": {
    "count": 5,
    "files": [".", "./src", "./src/main.rs", "./Cargo.toml", "./AGENTS.md"]
  }
}
```

---

### `dectl memory add --json`

```json
{
  "status": "ok",
  "data": {
    "id": 42,
    "content_preview": "Decisión: usar PostgreSQL por soporte de JSONB",
    "tags": [],
    "project": "my-api",
    "created_at": "2026-05-26T10:46:08.799091+00:00"
  }
}
```

---

### `dectl memory list --json`

```json
{
  "status": "ok",
  "data": {
    "entries": [
      {
        "id": 42,
        "content": "Decisión: usar PostgreSQL...",
        "tags": [],
        "project": "my-api",
        "created_at": "2026-05-26T10:46:08.799091+00:00",
        "updated_at": "2026-05-26T10:46:08.799091+00:00"
      }
    ],
    "count": 1
  }
}
```

---

### `dectl memory search --json`

```json
{
  "status": "ok",
  "data": {
    "query": "postgresql",
    "entries": [
      {
        "id": 42,
        "content": "Decisión: usar PostgreSQL...",
        "tags": [],
        "project": "my-api",
        "created_at": "2026-05-26T10:46:08.799091+00:00",
        "updated_at": "2026-05-26T10:46:08.799091+00:00"
      }
    ],
    "count": 1
  }
}
```

---

### `dectl memory show --json`

```json
{
  "status": "ok",
  "data": {
    "entry": {
      "id": 42,
      "content": "# Decisión: Base de datos\n\nUsar PostgreSQL por...",
      "tags": [],
      "project": "my-api",
      "created_at": "2026-05-26T10:46:08.799091+00:00",
      "updated_at": "2026-05-26T10:46:08.799091+00:00"
    }
  }
}
```

---

### `dectl workflow list --json`

```json
{
  "status": "ok",
  "data": {
    "workflows": [
      {
        "name": "implement_feature",
        "description": "Implement a complete feature with tests and documentation",
        "inputs": [
          { "name": "feature_name", "required": true, "default": null },
          { "name": "module", "required": true, "default": null },
          { "name": "include_tests", "required": false, "default": "true" }
        ]
      }
    ]
  }
}
```

---

### `dectl workflow describe --json`

```json
{
  "status": "ok",
  "data": {
    "workflow": {
      "name": "implement_feature",
      "description": "Implement a complete feature...",
      "inputs": [
        { "name": "feature_name", "description": "...", "required": true, "default": null }
      ],
      "steps": [
        { "index": 1, "type": "prompt", "description": "...", "content": "..." },
        { "index": 2, "type": "action", "description": "...", "cmd": ["dectl", "memory", "search", "{{feature_name}}"] }
      ]
    }
  }
}
```

---

### `dectl workflow run --json`

**Éxito**:
```json
{
  "status": "ok",
  "data": {
    "workflow": "implement_feature",
    "steps_executed": 5,
    "steps_skipped": 0,
    "failed_step": null
  }
}
```

**Error en un paso**:
```json
{
  "status": "error",
  "message": "Step 3 failed",
  "hint": "Fix the error above and resume with --from-step 3",
  "data": {
    "workflow": "implement_feature",
    "steps_executed": 2,
    "steps_skipped": 0,
    "failed_step": {
      "index": 3,
      "type": "action",
      "description": "Search relevant decisions",
      "cmd": ["dectl", "memory", "search", "user_auth"],
      "exit_code": 1,
      "stderr": "Not found"
    }
  }
}
```

---

### `dectl exec-from-file --json`

**Éxito**:
```json
{
  "status": "ok",
  "data": {
    "total": 3,
    "succeeded": 3,
    "failed": 0,
    "error_line": null,
    "error_message": null
  }
}
```

**Error**:
```json
{
  "status": "error",
  "data": {
    "total": 3,
    "succeeded": 1,
    "failed": 1,
    "error_line": 2,
    "error_message": "Command failed at line 2"
  }
}
```

---

### `dectl agent list --json`

```json
{
  "status": "ok",
  "data": {
    "agents": [
      {
        "name": "coder",
        "role": "Feature implementer following project conventions",
        "description": "Implements code respecting the stack and conventions from .dec/",
        "source": "builtin"
      }
    ]
  }
}
```

---

### `dectl agent describe --json`

```json
{
  "status": "ok",
  "data": {
    "name": "coder",
    "role": "Feature implementer...",
    "description": "...",
    "source": "builtin",
    "inputs": [],
    "steps": [
      { "type": "prompt", "content": "...", "description": "..." },
      { "type": "action", "cmd": ["dectl", "memory", "search", "{{task}}"], "description": "..." }
    ]
  }
}
```

---

### `dectl agent trust --json`

**Éxito**:
```json
{
  "status": "ok",
  "data": {
    "agent": "coder",
    "project": "/home/user/projects/myapp",
    "already_trusted": false
  }
}
```

**Ya confiado**:
```json
{
  "status": "ok",
  "data": {
    "agent": "coder",
    "project": "/home/user/projects/myapp",
    "already_trusted": true
  }
}
```

**Error** (agente no existe):
```json
{
  "status": "error",
  "message": "Agent 'nonexistent' not found",
  "hint": "Run 'dectl agent list' to see available agents"
}
```

---

### `dectl agent run --json`

```json
{
  "status": "ok",
  "data": {
    "agents": ["coder"],
    "status": "completed",
    "duration_secs": 12,
    "steps_executed": 3,
    "error": null
  }
}
```

---

### `dectl generate-completions` (no tiene `--json`)

El comando genera script de shell en stdout; no tiene output JSON.

---

### `dectl spec init --json`

**Éxito**:
```json
{
  "status": "ok",
  "data": {
    "message": ".dec/sdd/ ready",
    "bridge": {
      "project_toml": true,
      "project_isa": true
    },
    "next": "Interview the user and create specs/ with SDD documents"
  }
}
```

**Error** (no .dec/):
```json
{
  "status": "error",
  "message": ".dec/ not found. Run `dectl project init` first."
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

---

### `dectl session end --json`

**Éxito** (al menos un paso):
```json
{
  "status": "ok",
  "data": {
    "steps": [
      { "name": "last_session.md", "success": true, "message": "updated" },
      { "name": "progress.json", "success": true, "message": "synced with git" },
      { "name": "memory", "success": true, "message": "2 decisions saved" },
      { "name": "config_sync", "success": true, "message": "no changes detected" },
      { "name": "agent_sync", "success": true, "message": "0 agent sessions" }
    ],
    "decisions_saved": 2,
    "config_changes": {
      "stack_changed": false,
      "project_toml_updated": false,
      "isa_updated": false
    },
    "agent_sessions": 0
  }
}
```

**Parcial** (un paso falló):
```json
{
  "status": "ok",
  "data": {
    "steps": [
      { "name": "last_session.md", "success": true, "message": "updated" },
      { "name": "progress.json", "success": true, "message": "no git repo found (skipped)" },
      { "name": "memory", "success": false, "message": "database not found" },
      { "name": "config_sync", "success": true, "message": "project.toml updated" },
      { "name": "agent_sync", "success": true, "message": "3 agent sessions" }
    ],
    "decisions_saved": 0,
    "config_changes": {
      "stack_changed": true,
      "project_toml_updated": true,
      "isa_updated": false
    },
    "agent_sessions": 3
  }
}
```

**Error** (todos los pasos fallaron):
```json
{
  "status": "error",
  "message": "All session end steps failed",
  "data": {
    "steps": [
      { "name": "last_session.md", "success": false, "message": "failed to generate summary: ..." },
      { "name": "progress.json", "success": false, "message": "..." },
      { "name": "memory", "success": false, "message": "..." },
      { "name": "config_sync", "success": false, "message": "..." },
      { "name": "agent_sync", "success": false, "message": "..." }
    ],
    "decisions_saved": 0,
    "config_changes": {
      "stack_changed": false,
      "project_toml_updated": false,
      "isa_updated": false
    },
    "agent_sessions": 0
  }
}
```
