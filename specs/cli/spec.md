# Specification — dectl CLI
> *Technology-agnostic. Describe QUÉ hace el CLI, no cómo se implementa.*
> *Extiende specs/master/spec.md con el contrato exacto de cada comando.*
> *Version: 1.0 | Status: Draft | Last updated: 2026-05-13*

---

## Overview

El CLI `dectl` es el ejecutor del sistema. Expone una interfaz de línea de comandos que permite al developer y al modelo gestionar memoria persistente, inicializar y explorar proyectos, ejecutar workflows y correr listas de comandos desde archivo. Cada comando tiene un contrato explícito de input, output, exit codes y comportamiento ante errores.

---

## Usuarios

- **Developer**: invoca comandos directamente en la terminal
- **Modelo de lenguaje**: genera texto que contiene comandos `dectl`; el entorno de codificación los ejecuta
- **Scripts y automatización**: invocan `dectl` con `--json` y `--non-interactive` para integración en pipelines

---

## Requisitos Funcionales

---

### REQ-C-001: Comando `dectl project init`

**User Story**:
> Como developer, quiero inicializar `.dec/` en mi proyecto con un solo comando para tener contexto estructurado disponible de inmediato sin configuración manual.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl project init` en un directorio sin `.dec/` THEN SHALL crear la estructura nivel 1: `.dec/.gitignore`, `.dec/config/project.toml`, `.dec/isa/project.isa.md`
- WHEN se ejecuta con `--standard` THEN SHALL crear nivel 1 + nivel 2: `decisions/`, `workflows/implement_feature.yaml`, `workflows/design_architecture.yaml`, `prompts/system/base.md`, `state/progress.json`, `state/last_session.md`
- WHEN se ejecuta con `--full` THEN SHALL crear nivel 2 + nivel 3: `isa/architecture.isa.md`, `prompts/tasks/`, `knowledge/`
- WHEN ya existe `.dec/` en el directorio THEN SHALL abortar con mensaje claro y exit code 1, sin modificar nada
- WHEN completa exitosamente THEN SHALL mostrar lista de archivos creados y el próximo paso recomendado
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", level: 1|2|3, files_created: [...], next_step: "..."}`

---

### REQ-C-002: Comando `dectl project info`

**User Story**:
> Como developer o modelo, quiero ver un resumen del contexto del proyecto actual para orientarme rápidamente sin leer cada archivo de `.dec/` individualmente.

**Acceptance Criteria**:
- WHEN se ejecuta en un directorio con `.dec/config/project.toml` válido THEN SHALL mostrar: nombre, tipo, stack, schema_version y descripción del proyecto
- WHEN existe `.dec/isa/project.isa.md` THEN SHALL incluir la sección "Visión" en el resumen
- WHEN existe `.dec/state/progress.json` THEN SHALL mostrar conteo de features por estado (done/in_progress/pending/blocked)
- WHEN algún archivo esperado está ausente THEN SHALL mostrar el resumen parcial con advertencia por cada archivo faltante — no abortar
- WHEN `schema_version` del proyecto es incompatible con el CLI THEN SHALL mostrar advertencia prominente antes del resumen
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", project: {...}, isa_vision: "...", progress: {...}, warnings: [...]}`

---

### REQ-C-003: Comando `dectl project scan`

**User Story**:
> Como developer o modelo, quiero ver el árbol de archivos del proyecto para entender su estructura sin navegar manualmente.

**Acceptance Criteria**:
- WHEN se ejecuta en cualquier directorio THEN SHALL mostrar el árbol de archivos respetando las reglas de `.gitignore` del proyecto
- WHEN se usa `--depth <n>` THEN SHALL limitar la profundidad del árbol a `n` niveles
- WHEN no existe `.gitignore` THEN SHALL mostrar todos los archivos excluyendo `.git/` y `target/`
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", tree: [...]}` donde cada nodo tiene `path`, `type: "file"|"dir"` y `children`

---

### REQ-C-004: Comando `dectl memory add`

**User Story**:
> Como developer o modelo, quiero almacenar una nota o decisión en memoria persistente para recuperarla en sesiones futuras sin importar qué herramienta uso.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl memory add "<contenido>"` THEN SHALL insertar la entrada en SQLite con timestamp ISO 8601 y retornar el ID asignado
- WHEN se usa `--tags t1,t2` THEN SHALL almacenar los tags asociados a la entrada
- WHEN se usa `--project <nombre>` THEN SHALL asociar la entrada al proyecto indicado
- WHEN no se usa `--project` y existe `.dec/config/project.toml` en el directorio actual THEN SHALL auto-detectar el nombre del proyecto y asociarlo
- WHEN no se provee argumento de contenido y stdin no es TTY THEN SHALL abortar con mensaje claro y exit code 1
- WHEN no se provee argumento y stdin es un pipe THEN SHALL leer el contenido completo desde stdin y almacenarlo
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", id: 42, preview: "primeros 80 chars..."}`

---

### REQ-C-005: Comando `dectl memory list`

**User Story**:
> Como developer o modelo, quiero listar las entradas de memoria más recientes para retomar contexto de sesiones anteriores.

**Acceptance Criteria**:
- WHEN se ejecuta THEN SHALL mostrar entradas en orden cronológico inverso con: ID, fecha, tags y primeros 100 caracteres del contenido
- WHEN se usa `--project <nombre>` THEN SHALL filtrar por proyecto; mostrar solo entradas de ese proyecto y entradas globales (sin proyecto)
- WHEN se usa `--limit <n>` THEN SHALL mostrar máximo `n` entradas
- WHEN no hay entradas THEN SHALL mostrar mensaje informativo, no un error
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", entries: [{id, created_at, tags, project, preview}], total: n}`

---

### REQ-C-006: Comando `dectl memory search`

**User Story**:
> Como developer o modelo, quiero buscar en memoria por palabras clave para encontrar decisiones o notas específicas sin recordar cuándo las almacené.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl memory search "<query>"` THEN SHALL buscar en `content` y `tags` usando coincidencia de substring case-insensitive
- WHEN se usa `--project <nombre>` THEN SHALL limitar la búsqueda a entradas de ese proyecto y entradas globales
- WHEN hay resultados THEN SHALL mostrarlos con ID, fecha, tags y fragmento del contenido que contiene la coincidencia
- WHEN no hay resultados THEN SHALL mostrar mensaje informativo con la query usada, no un error
- WHEN la query está vacía THEN SHALL abortar con mensaje claro y exit code 1
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", query: "...", entries: [...], total: n}`

---

### REQ-C-007: Comando `dectl memory show`

**User Story**:
> Como developer o modelo, quiero ver el contenido completo de una entrada de memoria por su ID para leer decisiones o notas sin truncamiento.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl memory show <id>` con un ID existente THEN SHALL mostrar el contenido completo, fecha, tags y proyecto de la entrada
- WHEN el ID no existe THEN SHALL mostrar mensaje claro con el ID buscado y exit code 1
- WHEN el contenido tiene formato Markdown THEN SHALL renderizarlo como texto plano legible en terminal (no HTML)
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", entry: {id, content, tags, project, created_at, updated_at}}`

---

### REQ-C-008: Comando `dectl workflow list`

**User Story**:
> Como developer o modelo, quiero ver todos los workflows disponibles en el proyecto para saber qué automatizaciones existen sin abrir cada archivo YAML.

**Acceptance Criteria**:
- WHEN se ejecuta en un proyecto con `.dec/workflows/` THEN SHALL listar cada workflow con: nombre, descripción e inputs requeridos
- WHEN un archivo YAML en `.dec/workflows/` es inválido THEN SHALL mostrar advertencia con el nombre del archivo y continuar con los válidos
- WHEN no hay workflows THEN SHALL mostrar mensaje informativo, no un error
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", workflows: [{name, description, required_inputs: [...]}]}`

---

### REQ-C-009: Comando `dectl workflow describe`

**User Story**:
> Como developer o modelo, quiero ver el detalle completo de un workflow para entender qué hace cada paso antes de ejecutarlo.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl workflow describe <nombre>` THEN SHALL mostrar: nombre, descripción, todos los inputs con required/default, y todos los steps con tipo, descripción y contenido/comando
- WHEN el workflow no existe THEN SHALL mostrar mensaje claro con el nombre buscado y exit code 1
- WHEN el workflow tiene variables THEN SHALL mostrar los inputs en sección separada antes de los steps
- WHEN se usa `--json` THEN SHALL retornar el workflow completo como objeto JSON estructurado

---

### REQ-C-010: Comando `dectl workflow run`

**User Story**:
> Como developer o modelo, quiero ejecutar un workflow para guiar una tarea compleja de forma consistente sin recordar cada paso manualmente.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl workflow run <nombre>` THEN SHALL cargar, validar e iniciar la ejecución del workflow
- WHEN el workflow tiene inputs requeridos sin valor provisto THEN SHALL abortar antes del primer paso listando los inputs faltantes con exit code 1
- WHEN se usa `--var nombre=valor` THEN SHALL usar ese valor para la variable indicada; el flag es repetible
- WHEN el workflow contiene pasos `action` y no ha sido confiado antes THEN SHALL mostrar resumen de los comandos que ejecutará y pedir confirmación Y/n antes del primer paso `action`
- WHEN el usuario confirma o el workflow ya es confiado THEN SHALL ejecutar los pasos en orden
- WHEN un paso `prompt` se ejecuta THEN SHALL imprimir el contenido interpolado, indicar que el modelo debe actuar y pausar esperando ENTER del developer
- WHEN un paso `action` se ejecuta THEN SHALL mostrar el comando, ejecutarlo y mostrar su output en tiempo real
- WHEN un paso `write` se ejecuta THEN SHALL crear o sobreescribir el archivo indicado e informar la ruta
- WHEN un paso falla THEN SHALL mostrar qué paso falló (índice y descripción), el error, y el estado de los pasos anteriores — exit code 2
- WHEN se usa `--from-step <n>` THEN SHALL saltar los pasos anteriores a `n` e iniciar la ejecución desde el paso indicado; los pasos saltados se marcan como "skipped" en el output
- WHEN `--from-step <n>` referencia un paso que no existe THEN SHALL abortar con mensaje claro indicando el rango válido y exit code 1
- WHEN se usa `--dry-run` THEN SHALL mostrar todos los pasos con sus comandos/contenidos interpolados sin ejecutar ninguna acción
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok"|"error", steps_executed: n, failed_step: {...} | null}`

---

### REQ-C-011: Comando `dectl exec-from-file`

**User Story**:
> Como modelo o script, quiero ejecutar una lista de comandos `dectl` desde un archivo para automatizar secuencias sin repetir invocaciones manuales.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl exec-from-file <ruta>` THEN SHALL leer el archivo línea por línea, ejecutar cada línea no vacía como subcomando `dectl` en orden
- WHEN una línea comienza con `#` THEN SHALL tratarla como comentario e ignorarla
- WHEN un comando falla THEN SHALL reportar el número de línea, el comando fallido y el error — y detener la ejecución con exit code del comando fallido
- WHEN el archivo no existe THEN SHALL abortar con mensaje claro y exit code 1
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok"|"error", executed: n, failed_line: n | null, failed_cmd: "..." | null}`

---

### REQ-C-012: Flags globales

**User Story**:
> Como script o modelo, quiero flags consistentes en todos los comandos para integrar `dectl` en pipelines sin tratar cada comando de forma especial.

**Acceptance Criteria**:
- WHEN se usa `--json` en cualquier comando THEN SHALL producir JSON válido con envelope `{status, ...}` en stdout
- WHEN se usa `--help` en cualquier comando o subcomando THEN SHALL mostrar descripción, uso, flags disponibles y al menos un ejemplo
- WHEN se usa `--version` en el comando raíz THEN SHALL mostrar versión del binario y versión de schema de `.dec/` soportada
- WHEN stdout no es TTY (pipe o redirección) THEN SHALL desactivar colores automáticamente sin flag adicional
- WHEN se usa `--non-interactive` THEN SHALL abortar en lugar de mostrar prompts interactivos — útil para scripts

---

## Requisitos No Funcionales

- **Tiempo de respuesta**: todos los comandos de lectura completan en menos de 200ms en hardware estándar
- **Tiempo de respuesta — escritura**: comandos de escritura (memory add, project init) completan en menos de 500ms
- **Tamaño del binario**: el binario release con `strip` no excede 20MB
- **Portabilidad**: funciona en Linux x86_64 y macOS arm64/x86_64 sin dependencias de runtime
- **Compatibilidad de shell**: funciona en bash, zsh y fish sin configuración especial
- **Señales**: responde correctamente a SIGINT (Ctrl+C) — limpia estado parcial si aplica y sale con exit code 130

---

## Fuera de Alcance

- Ejecución de modelos de lenguaje
- Sincronización remota o acceso a memoria desde otra máquina
- GUI o interfaz web
- Comandos de edición interactiva en terminal (TUI)
- Gestión de múltiples bases de datos de memoria simultáneas

---

## Preguntas Abiertas

- [x] ¿`dectl memory add` desde stdin? → **Sí. Si no se provee argumento, lee desde stdin.**
- [x] ¿`dectl workflow run --from-step <n>`? → **Sí. Permite reanudar desde un paso específico.**
