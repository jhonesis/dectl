# Technical Research — dectl CLI
> *Investiga incógnitas técnicas específicas del CLI no cubiertas en master/research.md.*
> *Last updated: 2026-05-13*

---

## Contexto

Las decisiones de stack principales (clap, rusqlite, serde_yaml, anyhow, etc.) están resueltas en `specs/master/research.md`. Este documento investiga las incógnitas específicas del CLI: comportamiento de stdin, renderizado de Markdown en terminal, detección de TTY, manejo de señales y estado de ejecución para `--from-step`.

---

## Research Questions

### RQ-C-001: Lectura de stdin en Rust

**Context**: REQ-C-004 requiere que `dectl memory add` lea contenido desde stdin cuando no se provee argumento y stdin es un pipe. Rust permite leer stdin de múltiples formas con distintas implicaciones de rendimiento y comportamiento.

**Options Evaluated**:

| Opción | Ejemplo | Pros | Contras |
|--------|---------|------|---------|
| `std::io::stdin().read_to_string()` | Lee todo stdin hasta EOF | Simple, stdlib pura, sin deps | Lee todo en memoria — aceptable para notas y decisiones |
| `BufReader` línea por línea | Itera líneas | Eficiente para archivos grandes | Innecesariamente complejo para el caso de uso |
| Crate `atty` + stdin | Detecta TTY, luego lee | Separación clara de responsabilidades | `atty` está sin mantenimiento activo |
| Crate `is-terminal` | Detecta TTY, luego lee | Mantenida activamente, recomendada por la comunidad | Dependencia adicional |

**Decision**: `is-terminal` para detección de TTY + `std::io::stdin().read_to_string()` para lectura.

**Rationale**: `is-terminal` es el reemplazo mantenido de `atty` y es lo que `clap` usa internamente — añadirla no agrega peso real. `read_to_string()` es suficiente para el caso de uso: los contenidos de memoria son notas y decisiones, no archivos de gigabytes. La combinación es idiomática en Rust moderno.

**Implementación patrón**:
```rust
use is_terminal::IsTerminal;
use std::io::Read;

fn get_content(arg: Option<String>) -> anyhow::Result<String> {
    match arg {
        Some(content) => Ok(content),
        None if !std::io::stdin().is_terminal() => {
            let mut content = String::new();
            std::io::stdin().read_to_string(&mut content)?;
            Ok(content.trim().to_string())
        }
        None => anyhow::bail!("Provide content as argument or via stdin pipe"),
    }
}
```

---

### RQ-C-002: Detección de TTY para colores y comportamiento interactivo

**Context**: La constitution requiere desactivar colores cuando stdout no es TTY, y el spec requiere que `--non-interactive` evite prompts. Además, `dectl workflow run` necesita detectar si puede mostrar un prompt Y/n o debe asumir no-interactivo.

**Options Evaluated**:

| Opción | Pros | Contras |
|--------|------|---------|
| `is-terminal` (ya elegido en RQ-C-001) | Una sola dep para stdin y stdout TTY check | — |
| `atty` | Conocida | Sin mantenimiento activo |
| `clap` built-in color detection | Zero dep adicional para colores | No cubre detección para prompts Y/n |

**Decision**: `is-terminal` para toda detección de TTY (stdout, stdin, stderr). `colored` o `termcolor` para output con color.

**Rationale**: Reutilizar `is-terminal` ya presente por RQ-C-001 para consistencia. Para colores, `colored` es la opción más simple — permite escribir `"texto".green()` sin configuración. Se desactiva automáticamente si se detecta no-TTY o si la variable de entorno `NO_COLOR` está definida (estándar de la comunidad).

**Lógica de interactividad**:
```rust
fn is_interactive() -> bool {
    std::io::stdout().is_terminal() 
    && std::env::var("DECTL_NON_INTERACTIVE").is_err()
    // --non-interactive flag también lo desactiva via config global del run
}
```

---

### RQ-C-003: Renderizado de Markdown en terminal

**Context**: REQ-C-007 requiere que `dectl memory show` renderice Markdown como texto plano legible en terminal. El contenido de memoria soporta Markdown completo. La pregunta es cuánto renderizar.

**Options Evaluated**:

| Opción | Pros | Contras |
|--------|------|---------|
| Output raw sin procesar | Cero deps, siempre funciona | `**bold**`, `##` headers son ruido visual |
| `termimad` | Renderiza Markdown con colores y formato en terminal | Dep adicional, puede fallar en terminales no-estándar |
| `pulldown-cmark` + ANSI manual | Control total | Mucho código para poco beneficio |
| Procesamiento mínimo propio | Sin deps, elimina la sintaxis más molesta | Puede ser incompleto o inconsistente |

**Decision**: Procesamiento mínimo propio para `memory show`; raw para todos los demás outputs.

**Rationale**: El renderizado completo de Markdown en terminal es un problema complejo — distintos emuladores soportan distintos códigos ANSI. Para `memory show` basta con eliminar la sintaxis más ruidosa visualmente: `**bold**` → `bold`, `## Header` → `Header` con línea separadora, `` `code` `` → `code`. Esto es ~30 líneas de Rust sin dependencias externas. Para prompts de workflows, el Markdown se muestra raw — los modelos lo leen bien y los developers lo reconocen.

---

### RQ-C-004: Estado de ejecución para `--from-step`

**Context**: REQ-C-010 requiere que `dectl workflow run --from-step <n>` permita reanudar desde un paso específico. Esto implica que el CLI debe tener alguna forma de saber desde dónde continuar. La pregunta es si el estado se guarda entre invocaciones o si `--from-step` es simplemente un skip de los primeros `n-1` pasos.

**Options Evaluated**:

| Opción | Pros | Contras |
|--------|------|---------|
| Skip simple: ejecutar desde el paso N ignorando los anteriores | Sin estado persistente, simple de implementar | No verifica que los pasos previos hayan corrido; developer es responsable |
| Estado persistente en `~/.dectl/workflow_state.toml` | Sabe qué pasos corrieron exitosamente | Complejidad de gestionar estado obsoleto entre versiones del workflow |
| Estado en `.dec/state/` del proyecto | Visible al modelo y al developer | Poluciona `.dec/` con archivos de estado efímero |

**Decision**: Skip simple — `--from-step <n>` salta los primeros `n-1` pasos sin verificar estado previo.

**Rationale**: El caso de uso es recuperación de un fallo obvio: el developer sabe que los pasos 1-3 corrieron bien y el paso 4 falló. No necesita que el CLI lo verifique. La complejidad del estado persistente es desproporcionada para este beneficio. Un mensaje claro en la ejecución (`Skipping steps 1-3 (--from-step 4)`) documenta el comportamiento y pone la responsabilidad donde corresponde: en el developer.

---

### RQ-C-005: Manejo de SIGINT (Ctrl+C)

**Context**: La constitution requiere que el CLI responda correctamente a SIGINT con exit code 130 y limpie estado parcial si aplica. En Rust, el comportamiento por defecto de SIGINT termina el proceso sin cleanup. Durante `workflow run`, puede haber archivos parcialmente escritos o comandos externos en ejecución.

**Options Evaluated**:

| Opción | Pros | Contras |
|--------|------|---------|
| Comportamiento default de Rust (termina en SIGINT) | Cero código | No garantiza cleanup; exit code puede no ser 130 |
| `ctrlc` crate | Handler simple, cross-platform | Dep adicional; complejo en el contexto de procesos hijos |
| `signal-hook` crate | Más completo, manejo de múltiples señales | Más complejo de lo necesario para este caso |

**Decision**: `ctrlc` crate para handler de SIGINT con cleanup mínimo.

**Rationale**: Durante `workflow run`, si hay un proceso hijo corriendo (paso `action`), el SIGINT debe propagarse al hijo antes de terminar. `ctrlc` provee esto de forma cross-platform con ~10 líneas de código. El cleanup es mínimo: no hay estado que necesite rollback — los archivos escritos por pasos `write` ya completados se conservan (el developer puede reanudar con `--from-step`). El exit code 130 se garantiza explícitamente.

---

## Dependencias Adicionales al Maestro

Las siguientes dependencias se agregan al `Cargo.toml` como resultado de este research. Complementan las ya definidas en `master/research.md`.

| Dependencia | Versión | Licencia | Justificación |
|------------|---------|---------|--------------|
| `is-terminal` | 0.4+ | MIT | TTY detection para stdin/stdout/stderr (RQ-C-001, RQ-C-002) |
| `colored` | 2.x | MIT | Colores en output de terminal, auto-desactiva si no-TTY (RQ-C-002) |
| `ctrlc` | 3.x | MIT/Apache-2.0 | Handler SIGINT con propagación a procesos hijos (RQ-C-005) |

**Nota**: No se agrega dependencia para Markdown — se implementa procesamiento mínimo propio (~30 líneas).

---

## Resumen de Decisiones

| ID | Decisión | Resultado |
|----|---------|-----------|
| RQ-C-001 | Lectura stdin | `is-terminal` + `read_to_string()` |
| RQ-C-002 | Detección TTY y colores | `is-terminal` + `colored`; respeta `NO_COLOR` |
| RQ-C-003 | Renderizado Markdown | Procesamiento mínimo propio en `memory show` |
| RQ-C-004 | Estado `--from-step` | Skip simple sin estado persistente |
| RQ-C-005 | SIGINT | `ctrlc` crate, exit code 130, cleanup mínimo |
