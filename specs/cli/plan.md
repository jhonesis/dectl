# Technical Implementation Plan — dectl CLI
> *Describe CÓMO implementar el CLI. Technology-specific.*
> *Version: 1.0 | Status: Draft | Last updated: 2026-05-13*

---

## Referencias

- Implementa: `specs/cli/spec.md`
- Constitution: `specs/cli/constitution.md`
- Research: `specs/cli/research.md`
- Extiende: `specs/master/plan.md`

---

## Stack Completo

Del maestro (`master/research.md`) más las adiciones de este SDD:

```toml
[dependencies]
# Del maestro
clap = { version = "4", features = ["derive"] }
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
anyhow = "1"
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
ignore = "0.4"

# Adiciones del CLI
is-terminal = "0.4"
colored = "2"
ctrlc = { version = "3", features = ["termination"] }
```

---

## Estructura de Comandos (clap derive API)

```rust
// main.rs
#[derive(Parser)]
#[command(name = "dectl", version, about = "Dev Environment Control")]
#[command(propagate_version = true)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Disable interactive prompts (for scripts)
    #[arg(long, global = true)]
    non_interactive: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Project(ProjectArgs),
    Memory(MemoryArgs),
    Workflow(WorkflowArgs),
    ExecFromFile(ExecFromFileArgs),
}

// --- Project ---
#[derive(Args)]
struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommands,
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// Initialize .dec/ in the current directory
    Init(ProjectInitArgs),
    /// Show project context summary
    Info,
    /// Show project file tree
    Scan(ProjectScanArgs),
}

#[derive(Args)]
struct ProjectInitArgs {
    /// Create standard structure (level 2)
    #[arg(long)]
    standard: bool,
    /// Create full structure (level 3)
    #[arg(long)]
    full: bool,
}

#[derive(Args)]
struct ProjectScanArgs {
    /// Maximum depth to display
    #[arg(long, short)]
    depth: Option<usize>,
}

// --- Memory ---
#[derive(Args)]
struct MemoryArgs {
    #[command(subcommand)]
    command: MemoryCommands,
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Add a memory entry (reads from stdin if no content provided)
    Add(MemoryAddArgs),
    /// List memory entries
    List(MemoryListArgs),
    /// Search memory entries
    Search(MemorySearchArgs),
    /// Show a memory entry by ID
    Show(MemoryShowArgs),
}

#[derive(Args)]
struct MemoryAddArgs {
    /// Content to store (reads from stdin if omitted)
    content: Option<String>,
    /// Comma-separated tags
    #[arg(long)]
    tags: Option<String>,
    /// Project to associate with (auto-detected if in a .dec/ project)
    #[arg(long)]
    project: Option<String>,
}

#[derive(Args)]
struct MemoryListArgs {
    #[arg(long)]
    project: Option<String>,
    #[arg(long, default_value = "20")]
    limit: usize,
}

#[derive(Args)]
struct MemorySearchArgs {
    query: String,
    #[arg(long)]
    project: Option<String>,
}

#[derive(Args)]
struct MemoryShowArgs {
    id: i64,
}

// --- Workflow ---
#[derive(Args)]
struct WorkflowArgs {
    #[command(subcommand)]
    command: WorkflowCommands,
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// List available workflows
    List,
    /// Describe a workflow's steps and inputs
    Describe(WorkflowDescribeArgs),
    /// Run a workflow
    Run(WorkflowRunArgs),
}

#[derive(Args)]
struct WorkflowDescribeArgs {
    name: String,
}

#[derive(Args)]
struct WorkflowRunArgs {
    name: String,
    /// Input variable in name=value format (repeatable)
    #[arg(long = "var", value_name = "NAME=VALUE")]
    vars: Vec<String>,
    /// Show steps without executing actions
    #[arg(long)]
    dry_run: bool,
    /// Resume from step N (1-indexed)
    #[arg(long, value_name = "N")]
    from_step: Option<usize>,
}

// --- Protocol ---
#[derive(Args)]
struct ExecFromFileArgs {
    path: String,
}
```

---

## Output System

Toda la salida del CLI pasa por el módulo `core/output.rs`. Nunca se usa `println!` directamente fuera de `output.rs`.

```rust
// core/output.rs

pub struct Output {
    pub json_mode: bool,
    pub color: bool,  // false si no-TTY o NO_COLOR set
}

impl Output {
    pub fn new(json_mode: bool) -> Self {
        Self {
            json_mode,
            color: std::io::stdout().is_terminal()
                && std::env::var("NO_COLOR").is_err(),
        }
    }

    /// Print success data — human or JSON
    pub fn success<T: Serialize + HumanDisplay>(&self, data: T) { ... }

    /// Print error — human or JSON, always to stderr
    pub fn error(&self, message: &str, hint: Option<&str>) { ... }

    /// Print informational line to stderr (never in JSON output)
    pub fn info(&self, message: &str) { ... }

    /// Print step execution progress (workflow runner)
    pub fn step(&self, index: usize, total: usize, description: &str, step_type: &str) { ... }
}

// JSON envelope — todos los comandos producen esto
#[derive(Serialize)]
struct JsonEnvelope<T: Serialize> {
    status: &'static str,  // "ok" | "error"
    #[serde(flatten)]
    data: T,
}

// Ejemplo de output JSON exitoso:
// {"status":"ok","id":42,"preview":"Decisión: usar PostgreSQL por..."}

// Ejemplo de output JSON de error:
// {"status":"error","message":"File not found: .dec/config/project.toml",
//  "hint":"Run 'dectl project init' to create the project structure"}
```

---

## Patrón de Implementación por Comando

Cada comando sigue el mismo patrón de 4 capas:

```
1. CLI layer (main.rs / comando args)
   └── valida presencia de argumentos requeridos
   └── construye contexto (Output, Config)

2. Command handler (ej. memory/add.rs)
   └── lógica de negocio
   └── llama a módulos de infraestructura (db, fs)
   └── construye struct de resultado

3. Infrastructure layer (db.rs, config.rs, etc.)
   └── sin conocimiento de Output ni CLI args
   └── retorna Result<T, anyhow::Error>

4. Output layer (core/output.rs)
   └── recibe el struct de resultado
   └── formatea como human-readable o JSON
```

**Ejemplo — `dectl memory add`**:
```
main.rs
  → memory::add::run(args, output, config)
      → core::config::detect_project()        // auto-detect project name
      → core::stdin::read_content(args.content) // arg or stdin
      → memory::db::insert(content, tags, project)
      → output.success(MemoryAddResult { id, preview })
```

---

## Módulo `core/stdin.rs`

```rust
// core/stdin.rs
use is_terminal::IsTerminal;
use std::io::Read;

pub fn read_content(arg: Option<String>) -> anyhow::Result<String> {
    match arg {
        Some(content) if !content.is_empty() => Ok(content),
        Some(_) => anyhow::bail!(
            "Content cannot be empty.\nHint: provide content as argument or pipe via stdin"
        ),
        None if !std::io::stdin().is_terminal() => {
            let mut content = String::new();
            std::io::stdin().read_to_string(&mut content)?;
            let content = content.trim().to_string();
            if content.is_empty() {
                anyhow::bail!("No content received from stdin")
            }
            Ok(content)
        }
        None => anyhow::bail!(
            "No content provided.\nUsage: dectl memory add \"<content>\"\n       cat file.md | dectl memory add"
        ),
    }
}
```

---

## Módulo `workflow/runner.rs` — Flujo Completo

```
run(name, vars, dry_run, from_step, non_interactive, output)
  │
  ├── 1. loader::load(name)           → Workflow struct
  ├── 2. validate_inputs(workflow, vars) → HashMap<String, String> o error
  ├── 3. trust::check(project_path, name)
  │       si no trusted y hay action steps y no dry_run:
  │         si non_interactive → error (no puede pedir confirmación)
  │         si interactive → mostrar comandos → pedir Y/n → guardar trust
  │
  ├── 4. iterar steps (con índice 1-based)
  │       si índice < from_step → output.info("Skipping step N...") → continuar
  │       │
  │       ├── type: prompt
  │       │     interpolate(content, vars)
  │       │     output.step(...)
  │       │     si dry_run → mostrar contenido → continuar
  │       │     si no dry_run → imprimir contenido → esperar ENTER (si interactive)
  │       │
  │       ├── type: action
  │       │     interpolate(cmd[], vars)
  │       │     output.step(...)
  │       │     si dry_run → mostrar comando → continuar
  │       │     si no dry_run → ejecutar via Command o sh -c
  │       │       éxito → continuar
  │       │       fallo → output.error(step_info + stderr) → exit 2
  │       │
  │       └── type: write
  │             interpolate(path + content, vars)
  │             output.step(...)
  │             si dry_run → mostrar path + preview → continuar
  │             si no dry_run → crear dirs si necesario → escribir archivo
  │               éxito → continuar
  │               fallo → output.error(...) → exit 2
  │
  └── 5. output.success(WorkflowRunResult { steps_executed, skipped })
```

---

## Manejo de SIGINT

```rust
// main.rs — setup antes de ejecutar cualquier comando
fn setup_ctrlc() {
    ctrlc::set_handler(move || {
        // Limpiar línea de progreso si existe
        eprint!("\r");
        eprintln!("Interrupted.");
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");
}
```

Nota: los procesos hijos lanzados por pasos `action` reciben el SIGINT automáticamente por herencia del grupo de procesos. No se necesita propagación manual.

---

## Renderizado Markdown Mínimo (`memory show`)

```rust
// core/markdown.rs
pub fn to_plain(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        // Headers: ## Title → Title + separador
        if let Some(title) = line.strip_prefix("# ").or(line.strip_prefix("## "))
            .or(line.strip_prefix("### ")) {
            out.push_str(title);
            out.push('\n');
            out.push_str(&"─".repeat(title.len().min(60)));
        }
        // Bold/italic: **text** → text, *text* → text
        else {
            let cleaned = line
                .replace("**", "")
                .replace('*', "")
                .replace("__", "")
                .replace('_', "");
            // Inline code: `code` → code (mantener sin backticks)
            let cleaned = cleaned.replace('`', "");
            out.push_str(&cleaned);
        }
        out.push('\n');
    }
    out
}
```

---

## Exit Codes — Implementación

```rust
// core/error.rs
pub enum ExitCode {
    Success = 0,
    UserError = 1,     // args inválidos, archivo no encontrado, input vacío
    SystemError = 2,   // permisos, disco lleno, comando externo falló
    StateError = 3,    // .dec/ corrupto, schema incompatible, trust denegado
    Interrupted = 130, // SIGINT
}

// Uso en main.rs:
fn main() {
    setup_ctrlc();
    let cli = Cli::parse();
    let output = Output::new(cli.json);

    let result = run_command(cli.command, &output);

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            output.error(&e.to_string(), e.downcast_ref::<Hint>().map(|h| h.0.as_str()));
            let code = classify_error(&e);
            std::process::exit(code as i32);
        }
    }
}
```

---

## Fases de Implementación

### Fase 1 — Comandos core y memoria
`project init/info/scan` + todos los comandos `memory` + `exec-from-file` + sistema de output + stdin.

Depende de: D001–D015 (templates de `.dec/`) completados primero.

### Fase 2 — Workflows
`workflow list/describe/run` con trust, interpolación de variables, `--dry-run` y `--from-step`.

Depende de: D016–D025 (motor de interpolación) completados primero.

### Fase 3 — Polish
Shell completions, `--non-interactive` validado en todos los caminos, colores refinados, mensajes de error revisados con developers reales.

---

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|-----------|
| `--from-step` usado con workflow modificado desde la última ejecución | Medio | Mostrar advertencia si el workflow fue modificado (comparar mtime) |
| Proceso hijo de `action` step no termina ante SIGINT | Alto | Usar `Command::kill_on_drop` para garantizar cleanup |
| stdin en pipe vacío cuelga el proceso | Medio | Timeout de lectura de stdin de 30 segundos |
| Colores en terminal Windows cmd.exe | Bajo | `colored` soporta Windows 10+ con ANSI nativo; documentar limitación |
