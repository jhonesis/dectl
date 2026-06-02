# Specification — dectl CLI
> *Technology-agnostic. Describe QUÉ hace el CLI, no cómo se implementa.*
> *Extiende specs/master/spec.md con el contrato exacto de cada comando.*
> *Version: 1.0 | Status: Updated | Last updated: 2026-06-02*

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
- WHEN se ejecuta con `--standard` THEN SHALL crear nivel 1 + nivel 2: `decisions/`, `workflows/implement_feature.yaml`, `workflows/design_architecture.yaml`, `prompts/system/base.md`, `prompts/system/integration.md`, `state/progress.json`, `state/last_session.md`
- WHEN se ejecuta con `--full` THEN SHALL crear nivel 2 + nivel 3: `isa/architecture.isa.md`, `prompts/tasks/`, `knowledge/`
- WHEN ya existe `.dec/` en el directorio THEN SHALL abortar con mensaje claro y exit code 1, sin modificar nada
- WHEN completa exitosamente THEN SHALL mostrar lista de archivos creados y el próximo paso recomendado
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", level: 1|2|3, files_created: [...], next_step: "..."}`
- WHEN el proyecto tiene código existente THEN SHALL auto-detectar el stack (lenguajes, frameworks, herramientas) y auto-llenar los archivos `.dec/` con el contexto detectado
- WHEN el proyecto está vacío THEN SHALL ofrecer prompts interactivos para nombre, tipo, lenguajes, descripción y visión (si TTY disponible)
- WHEN se usa `--type api|cli|microservice|other` THEN SHALL crear workflows y prompts específicos del tipo
- WHEN init completa en un proyecto no vacío THEN SHALL crear `AGENTS.md` en la raíz del proyecto

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
- WHEN se usa `--global` THEN SHALL NO asociar la entrada a ningún proyecto (memoria global)
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
- WHEN se usa `--global` THEN SHALL mostrar todas las entradas sin filtrar por proyecto
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
- WHEN se usa `--global` THEN SHALL buscar en todas las entradas sin filtrar por proyecto
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

### REQ-C-013: Comando `dectl project context`

**User Story**:
> Como modelo de lenguaje en un entorno stateless, quiero obtener un resumen compacto del proyecto para tener contexto sin leer cada archivo individualmente.

**Acceptance Criteria**:
- WHEN se ejecuta en un directorio con `.dec/` THEN SHALL leer los archivos prioritarios (project.toml, project.isa.md, last_session.md, progress.json, integration.md, decisions recientes) y concatenarlos en un resumen legible
- WHEN se usa `--max-tokens <n>` THEN SHALL asignar un budget proporcional a cada sección según su peso (last_session 25%, isa 20%, config 15%, progress 10%, integration 5%, decisions 25%), truncar cada archivo individualmente, y redistribuir el sobrante iterativamente
- WHEN se usa `--format json` THEN SHALL retornar el contexto como objeto JSON estructurado con campos: project, vision, last_session, decisions, progress, token_count
- WHEN se usa `--format compact` THEN SHALL retornar solo 6 líneas clave (project, stack, last_session, progress, decisions, memory_hits) ignorando `--max-tokens`
- WHEN el proyecto tiene `last_session.md` con `**Fecha**: YYYY-MM-DD` THEN SHALL comparar fechas de modificación de cada archivo contra esa fecha; archivos más recientes que la última sesión reciben peso ×2, archivos sin cambios reciben peso ×0.5 (antes de normalizar)
- WHEN no existe `.dec/` THEN SHALL abortar con mensaje claro y exit code 1

---

### REQ-C-014: Comando `dectl session end`

**User Story**:
> Como developer o modelo, quiero ejecutar un solo comando al finalizar la sesión para que todo el contexto se capture automáticamente y esté disponible en la próxima sesión.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl session end` THEN SHALL realizar cinco acciones en secuencia:
  1. Actualizar `.dec/state/last_session.md` con resumen estructurado (fecha, acciones, pendientes, decisiones, próximo paso)
  2. Sincronizar cambios de git a `.dec/state/progress.json` (marcar features como done, detectar nuevas features)
  3. Capturar decisiones no guardadas y almacenarlas en memoria (usando patrones regex sobre commits y archivos de sesión)
  4. Sincronizar cambios del stack detectados en el filesystem con `.dec/config/project.toml` (config_sync)
  5. Registrar actividad de agentes desde el último cierre de sesión en el log (agent_sync)
- WHEN un paso falla THEN los pasos restantes SHALL continuar independientemente (un fallo no detiene a los demás)
- WHEN se usa `--dry-run` THEN SHALL previsualizar todos los cambios sin escribir ningún archivo
- WHEN se usa `--skip-git` THEN SHALL omitir el paso de sincronización con git sin error
- WHEN no existe un repositorio git THEN SHALL omitir el paso de git gracefulmente (no es un error)
- WHEN se usa `--json` THEN SHALL retornar un resultado estructurado con estado por paso, conteo de decisiones guardadas, cambios de configuración y sesiones de agentes: `{status: "ok", data: {steps: [{name, success, message}], decisions_saved: n, config_changes: {stack_changed, project_toml_updated, isa_updated}, agent_sessions: n}}`
- WHEN todos los pasos fallan THEN SHALL exit con código no cero
- WHEN al menos un paso tiene éxito THEN SHALL exit con código 0

---

### REQ-C-015: Comando `dectl generate-completions`

**User Story**:
> Como developer, quiero generar scripts de autocompletado para mi shell para tener sugerencias de comandos y flags al presionar Tab.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl generate-completions bash|zsh|fish|powershell` THEN SHALL generar el script de autocompletado correspondiente en stdout
- WHEN el shell no es soportado THEN SHALL abortar con mensaje claro listando los shells soportados y exit code 1
- WHEN se usa con pipe o redirección THEN SHALL escribir el script sin colores ni texto adicional

---

### REQ-C-016: Comando `dectl version`

**User Story**:
> Como developer o script, quiero verificar la versión instalada del CLI para confirmar compatibilidad.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl version` THEN SHALL mostrar la versión del binario
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", version: "0.1.0", schema: "1.0"}`

---

### REQ-C-017: Config Sync en `dectl session end`

**User Story**:
> Como developer, quiero que `dectl session end` detecte automáticamente cambios en el stack del proyecto y actualice `.dec/config/project.toml` para mantener la coherencia entre el código real y la configuración registrada.

**Acceptance Criteria**:
- WHEN `dectl session end` se ejecuta THEN SHALL ejecutar un paso de config_sync después de decision_capture
- WHEN el stack detectado en el filesystem difiere del registrado en `project.toml` THEN SHALL hacer merge de los items nuevos (languages, frameworks, tools) sin remover existentes
- WHEN el `project.type` ha cambiado (ej: de "npm" a "cargo") THEN SHALL actualizar el tipo en `project.toml`
- WHEN se usa `--dry-run` THEN SHALL mostrar cambios detectados sin aplicarlos
- WHEN `project.isa.md` no menciona lenguajes o frameworks detectados THEN SHALL generar warnings de coherencia (sin modificar el archivo)
- WHEN el paso de config_sync falla THEN SHALL reportar el error pero NO detener los demás pasos de session end
- WHEN se usa `--json` THEN SHALL incluir `config_changes` en el output con `stack_changed`, `project_toml_updated` e `isa_updated`

---

### REQ-C-018: Comandos de Agentes

**User Story**:
> Como developer o modelo, quiero invocar agentes especializados para tareas específicas de coding, review, research y documentation.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl agent list` THEN SHALL mostrar agentes built-in y custom con nombre, rol, descripción y source
- WHEN se ejecuta `dectl agent describe <type>` THEN SHALL mostrar definición completa del agente
- WHEN se ejecuta `dectl agent run <type> --task "<desc>"` THEN SHALL cargar agente, inyectar tarea, ejecutar steps, y registrar en agent_log
- WHEN se ejecuta `dectl agent run --parallel <type1>,<type2>` THEN SHALL lanzar agentes en threads separados
- WHEN un agente falla en paralelo THEN SHALL continuar con los demás y reportar status "partial"
- WHEN se usa `--json` en cualquier comando de agente THEN SHALL retornar envelope con status y datos específicos
- WHEN un agente custom tiene mismo nombre que built-in THEN SHALL usar el custom

**Referencia**: Ver `specs/agents/spec.md` para requisitos detallados (REQ-A-001 a REQ-A-009).

---

### REQ-C-020: Comando `dectl agent trust`

**User Story**:
> Como developer o modelo, quiero confiar un agente sin ejecutarlo para evitar prompts interactivos durante `dectl agent run`.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl agent trust <type>` THEN SHALL verificar que el agente existe (built-in o custom) y mostrar error si no
- WHEN se usa `--project <path>` THEN SHALL confiar el agente para el proyecto indicado (por defecto: directorio actual)
- WHEN el agente ya está confiado THEN SHALL informar que ya está confiado (idempotente)
- WHEN se completa THEN SHALL agregar entrada en `~/.dectl/trust.toml` con path canónico y timestamp
- WHEN se usa `--json` THEN SHALL retornar `{status:"ok", agent:"coder", project:"/path", already_trusted:false}`
- WHEN el agente no existe THEN SHALL abortar con mensaje claro y exit code 1
- WHEN `--non-interactive` y trust necesario en `agent run` THEN el mensaje de error SHALL sugerir `dectl agent trust <type> --project .`

---

### REQ-C-019: Comando `dectl spec init`

**User Story**:
> Como developer o modelo, quiero inicializar la metodología SDD en .dec/ para que el agente IA pueda seguir el proceso Build+Verify+Gate y crear specs/ con contenido real.

**Acceptance Criteria**:
- WHEN se ejecuta `dectl spec init` THEN SHALL asegurar que `.dec/sdd/` existe con SKILL.md, references/templates.md y references/examples.md
- WHEN `.dec/sdd/` ya existe THEN SHALL ser idempotente (no modificar nada)
- WHEN se ejecuta THEN SHALL actualizar `.dec/config/project.toml` con `[specs] dir = "specs"`
- WHEN se ejecuta THEN SHALL actualizar `.dec/isa/project.isa.md` con enlace a "See specs/ for SDD artifacts"
- WHEN no existe `.dec/` THEN SHALL abortar con mensaje claro y exit code 1
- WHEN no existe `.dec/config/project.toml` THEN SHALL abortar con mensaje claro
- WHEN se usa `--json` THEN SHALL retornar `{status: "ok", data: {message: ".dec/sdd/ ready", bridge: {project_toml: true, project_isa: true}, next: "Interview the user and create specs/"}}`

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
