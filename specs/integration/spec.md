# Specification — Integration Layer
> *Technology-agnostic. Define QUÉ debe ocurrir en la interacción entre los tres actores.*
> *Version: 1.0 | Status: Draft | Last updated: 2026-05-13*

---

## Overview

La capa de integración define el ciclo completo de una sesión de trabajo con dectl: cómo el modelo carga contexto, cómo decide e invoca comandos `dectl`, cómo procesa los resultados, y cómo cierra la sesión preservando estado. Es el contrato que hace al sistema model-agnostic — cualquier entorno que lo cumpla funciona con dectl sin modificación.

---

## Usuarios

- **Modelo de lenguaje**: cualquier IA que lee archivos y ejecuta comandos — Claude Code, Gemini CLI, Qwen CLI, Phi, o cualquier entorno futuro
- **Developer**: abre el proyecto, inicia la sesión, supervisa el trabajo del modelo
- **Entorno de codificación**: el shell, editor o herramienta que media entre el modelo y el sistema operativo

---

## Requisitos Funcionales

---

### REQ-I-001: Carga de contexto al inicio de sesión

**User Story**:
> Como modelo de lenguaje, quiero un protocolo claro de qué leer al iniciar una sesión para tener todo el contexto necesario sin que el developer tenga que repetirme información de sesiones anteriores.

**Acceptance Criteria**:
- WHEN el modelo inicia una sesión en un proyecto con `.dec/` THEN SHALL leer en este orden: `config/project.toml` → `isa/project.isa.md` → `state/last_session.md`
- WHEN existe `.dec/state/last_session.md` con contenido THEN el modelo SHALL retomar desde donde dice "Próximo paso recomendado" sin pedir confirmación al developer
- WHEN el modelo detecta que `.dec/` no existe en el proyecto THEN SHALL informar al developer y sugerir `dectl project init` antes de continuar
- WHEN el modelo ha leído el contexto base THEN SHALL confirmar al developer qué entendió del proyecto en no más de 3 líneas antes de preguntar qué hacer
- WHEN existe `.dec/prompts/system/integration.md` THEN el modelo SHALL seguir las instrucciones de ciclo de sesión ahí definidas con prioridad sobre el comportamiento por defecto
- WHEN el modelo ejecuta `dectl project info --json` al inicio THEN SHALL verificar el campo `warnings` y escalar al developer si contiene advertencias de schema_version

---

### REQ-I-002: Consulta de contexto específico

**User Story**:
> Como modelo de lenguaje, quiero saber exactamente cuándo y qué consultar en `.dec/` para no tomar decisiones basadas en suposiciones.

**Acceptance Criteria**:
- WHEN el developer pide un cambio arquitectónico THEN el modelo SHALL leer `.dec/decisions/` antes de responder
- WHEN el developer pide implementar una feature THEN el modelo SHALL leer el workflow relevante en `.dec/workflows/` si existe
- WHEN el modelo encuentra un término de dominio desconocido THEN SHALL consultar `.dec/knowledge/glossary.md` si existe, antes de asumir su significado
- WHEN el modelo va a escribir código THEN SHALL verificar `.dec/config/project.toml` para respetar convenciones del stack
- WHEN el modelo no encuentra el contexto relevante en `.dec/` THEN SHALL preguntar al developer en lugar de asumir

---

### REQ-I-003: Invocación de comandos dectl

**User Story**:
> Como entorno de codificación, quiero un patrón estándar para que el modelo genere e invoque comandos `dectl` para poder ejecutarlos de forma confiable sin interpretación adicional.

**Acceptance Criteria**:
- WHEN el modelo necesita persistir información THEN SHALL generar el comando `dectl` exacto como texto plano ejecutable
- WHEN el modelo genera un comando `dectl` THEN SHALL describirlo en una línea antes de ejecutarlo para que el developer entienda qué va a ocurrir
- WHEN el resultado de un comando es necesario para continuar THEN el modelo SHALL usar `--json` y parsear el campo `status` antes de actuar sobre los datos
- WHEN un comando retorna `status: "error"` THEN el modelo SHALL leer `message` y `hint` y decidir entre: corregir el comando, escalar al developer, o abortar la tarea
- WHEN el modelo invoca múltiples comandos en secuencia THEN SHALL verificar el resultado de cada uno antes de ejecutar el siguiente

---

### REQ-I-004: Ciclo de sesión completo

**User Story**:
> Como developer, quiero que el modelo siga un ciclo predecible de inicio, trabajo y cierre para que cualquier sesión pueda ser retomada desde donde quedó, independientemente del entorno usado.

**Acceptance Criteria**:
- WHEN inicia una sesión THEN el modelo SHALL ejecutar el protocolo de carga de contexto (REQ-I-001) antes de cualquier acción
- WHEN completa una tarea significativa THEN el modelo SHALL actualizar `.dec/state/progress.json` si afecta el estado de una feature
- WHEN finaliza una sesión THEN el modelo SHALL escribir `.dec/state/last_session.md` con: qué se hizo, qué quedó pendiente, decisiones tomadas y próximo paso recomendado
- WHEN toma una decisión arquitectónica THEN el modelo SHALL crear un archivo en `.dec/decisions/` con el formato ADR definido en dot-dec/research.md
- WHEN completa una tarea que vale la pena recordar THEN el modelo SHALL ejecutar `dectl memory add` con un resumen conciso

---

### REQ-I-005: Ejecución de workflows

**User Story**:
> Como developer, quiero que el modelo use los workflows definidos en `.dec/workflows/` como guía de trabajo para que las tareas complejas se ejecuten de forma consistente sin importar qué modelo uso.

**Acceptance Criteria**:
- WHEN existe un workflow para la tarea solicitada THEN el modelo SHALL seguirlo en lugar de improvisar su propio proceso
- WHEN el modelo encuentra un paso `prompt` en un workflow THEN SHALL ejecutar exactamente las instrucciones de ese paso
- WHEN el modelo encuentra un paso `action` en un workflow THEN SHALL ejecutar el comando indicado y verificar el resultado
- WHEN el modelo encuentra un paso `write` en un workflow THEN SHALL crear el archivo indicado con el contenido especificado
- WHEN un workflow tiene variables de entrada THEN el modelo SHALL obtener los valores del developer si no están disponibles en el contexto antes de iniciar

---

### REQ-I-006: Manejo de errores en la integración

**User Story**:
> Como developer, quiero que el modelo maneje los errores de forma transparente para que yo pueda intervenir cuando sea necesario sin perder el contexto del trabajo en curso.

**Acceptance Criteria**:
- WHEN un comando `dectl` falla con exit code 1 (error de usuario) THEN el modelo SHALL corregir el comando y reintentar una vez antes de escalar
- WHEN un comando `dectl` falla con exit code 2 (error de sistema) THEN el modelo SHALL escalar inmediatamente al developer con descripción del problema y contexto de qué se estaba haciendo
- WHEN un comando falla dos veces consecutivas THEN el modelo SHALL detenerse y reportar al developer con: comando ejecutado, error recibido, y sugerencia de acción
- WHEN un workflow falla en un paso THEN el modelo SHALL informar al developer el paso fallido, el error, y que puede reanudar con `dectl workflow run <nombre> --from-step <n>`
- WHEN el modelo no puede completar una tarea por falta de contexto en `.dec/` THEN SHALL documentar qué información falta antes de detenerse

---

### REQ-I-007: Compatibilidad con entornos de codificación

**User Story**:
> Como developer que usa distintas herramientas de IA, quiero que dectl funcione igual de bien con cualquier entorno que soporte lectura de archivos y ejecución de comandos.

**Acceptance Criteria**:
- WHEN un entorno puede leer archivos del sistema THEN SHALL poder consumir `.dec/` sin configuración adicional
- WHEN un entorno puede ejecutar comandos del sistema THEN SHALL poder invocar `dectl` sin configuración adicional
- WHEN un entorno no soporta ejecución de comandos THEN SHALL poder usar `.dec/` en modo lectura para obtener contexto parcial — dectl no debe requerir el CLI para proveer valor básico
- WHEN el developer cambia de entorno entre sesiones THEN el sistema SHALL funcionar correctamente porque el estado está en `.dec/` y `~/.dectl/`, no en el entorno

---

### REQ-I-008: Protocolo exec-from-file

**User Story**:
> Como modelo o script de automatización, quiero poder generar una lista de comandos `dectl` en un archivo y ejecutarlos en batch para coordinar secuencias complejas sin interacción manual paso a paso.

**Acceptance Criteria**:
- WHEN el modelo genera una secuencia de comandos `dectl` THEN SHALL poder escribirlos en un archivo y ejecutarlos con `dectl exec-from-file <ruta>`
- WHEN un comando en el archivo falla THEN la ejecución SHALL detenerse en ese punto con el número de línea y el error
- WHEN el modelo usa exec-from-file THEN SHALL verificar el resultado final antes de reportar éxito al developer
- WHEN el archivo contiene comentarios con `#` THEN SHALL ser ignorados, permitiendo documentar la secuencia inline

---

## Requisitos No Funcionales

- **Latencia de integración**: el overhead de invocar `dectl` (sin contar ejecución de comandos externos) no debe exceder 200ms — el modelo no debe esperar por el CLI
- **Resiliencia**: un fallo del CLI no debe corromper `.dec/` ni la memoria SQLite — el modelo puede reintentar sin pérdida de estado
- **Observabilidad**: el developer debe poder entender qué está haciendo el modelo en todo momento — cada invocación de `dectl` debe ser visible en el entorno
- **Idempotencia de lectura**: leer cualquier archivo de `.dec/` múltiples veces no produce side effects — el modelo puede releer libremente

---

## Fuera de Alcance

- Comunicación bidireccional en tiempo real entre CLI y modelo (el CLI no llama al modelo)
- Autenticación o autorización entre actores (el sistema es local y de un solo usuario)
- Sincronización de estado entre múltiples modelos trabajando en paralelo
- Plugin específico para ningún entorno (la integración es via filesystem + shell)

---

## Preguntas Abiertas

- [x] ¿Debe existir `.dec/prompts/system/integration.md`? → **Sí. Va en nivel 2. Permite personalizar el ciclo de sesión por proyecto.**
- [x] ¿Quién verifica schema_version? → **El CLI vía `dectl project info --json`. El modelo lee el campo `warnings` y escala si hay advertencia.**
