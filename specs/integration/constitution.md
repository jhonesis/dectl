# Constitution — Integration Layer
> *Principios que gobiernan cómo se comunican los tres actores del sistema.*
> *Hereda y extiende: specs/master/constitution.md*
> *Última actualización: 2026-05-13*

---

## 1. Identidad

La capa de integración no es código. Es un contrato de comportamiento: define cómo el modelo consume `.dec/`, cómo invoca `dectl`, cómo el CLI responde, y cómo el ciclo se cierra. Es el pegamento invisible que hace que los tres actores funcionen como un sistema coherente sin acoplamiento directo entre ellos.

Este contrato es lo que permite que dectl sea model-agnostic: cualquier entorno que respete este contrato funciona con dectl sin modificación.

---

## 2. Principios Fundamentales

**1. El contrato es el único punto de acoplamiento**
Los tres actores (modelo, CLI, `.dec/`) se conocen únicamente a través de este contrato. El modelo no sabe cómo está implementado el CLI. El CLI no sabe qué modelo lo invoca. `.dec/` no sabe quién lo lee. Cualquier cambio que rompa el contrato es un breaking change.

**2. El modelo siempre lidera, el CLI siempre sigue**
El modelo decide qué hacer. El CLI ejecuta lo que el modelo decide. Nunca al revés. El CLI no inicia acciones por sí solo salvo cuando el developer lo invoca directamente. Esta asimetría es fundamental — eliminarla introduce comportamiento impredecible.

**3. Todo output del CLI es consumible por el modelo**
Cada línea que `dectl` escribe a stdout o stderr debe ser legible y accionable por un modelo de 7B+. Mensajes de error en lenguaje natural. JSON disponible siempre. Sin códigos crípticos, sin stacktraces expuestos al modelo.

**4. El ciclo de sesión es explícito**
Una sesión tiene inicio, cuerpo y cierre. El modelo sabe cuándo empieza (lee contexto), qué puede hacer (sigue workflows), y cómo terminar (actualiza estado). Este ciclo no es implícito ni asumido — está documentado y el modelo puede seguirlo sin instrucciones verbales del developer.

**5. La integración no requiere instalación extra**
Cualquier entorno de codificación con IA que pueda ejecutar comandos del sistema puede integrarse con dectl. No hay SDK, no hay plugin obligatorio, no hay configuración especial. El contrato se respeta con capacidades universales: leer archivos + ejecutar comandos.

**6. Los errores de integración son recuperables**
Si un comando `dectl` falla durante una sesión, el modelo debe poder leer el error, entender qué pasó y decidir si continuar, corregir o escalar al developer. Los errores no deben dejar el sistema en un estado irrecuperable.

---

## 3. Reglas del Contrato

### Lectura de contexto
- El modelo DEBE leer `.dec/config/project.toml` e `.dec/isa/project.isa.md` antes de actuar en un proyecto nuevo
- El modelo DEBE consultar `.dec/decisions/` antes de proponer cambios arquitectónicos
- El modelo DEBE leer `.dec/state/last_session.md` al inicio de cada sesión para retomar contexto
- El modelo NO debe asumir que recuerda decisiones de sesiones anteriores — siempre releer `.dec/`

### Invocación del CLI
- El modelo invoca `dectl` generando texto con el comando exacto; el entorno lo ejecuta
- El modelo DEBE usar `--json` cuando necesita parsear el resultado de un comando
- El modelo NO debe invocar comandos que no entiende completamente — debe describir el comando antes de ejecutarlo
- El modelo DEBE verificar el exit code o el campo `status` en JSON antes de continuar

### Actualización de estado
- Al finalizar una tarea importante, el modelo DEBE actualizar `.dec/state/last_session.md`
- Al tomar una decisión arquitectónica, el modelo DEBE registrarla en `.dec/decisions/`
- Al completar una feature, el modelo DEBE actualizar `.dec/state/progress.json`
- Al almacenar contexto importante, el modelo DEBE ejecutar `dectl memory add`

### Manejo de errores
- Si un comando `dectl` retorna `status: "error"`, el modelo DEBE leer `message` y `hint` antes de reintentar
- Si un error persiste tras dos intentos, el modelo DEBE escalar al developer con descripción clara
- El modelo NUNCA debe ignorar un exit code distinto de 0

---

## 4. Lo que NO es parte del contrato

- El lenguaje de programación del entorno de codificación
- El tamaño o familia del modelo de lenguaje
- El sistema operativo del developer (dentro de lo soportado por el CLI)
- La forma en que el entorno muestra el output del CLI al developer
- Cómo el developer interactúa con el entorno (chat, comandos, voz)

---

## 5. Definition of Done para cambios en la integración

Un cambio al contrato de integración está completo cuando:
- [ ] El comportamiento nuevo está documentado en `specs/integration/spec.md`
- [ ] El cambio es backwards compatible O está marcado como breaking con versión mayor
- [ ] El `CLAUDE.md` del proyecto refleja el cambio
- [ ] Existe al menos un ejemplo concreto del nuevo comportamiento en `plan.md`
- [ ] Los tres actores pueden cumplir su parte del contrato sin modificación del otro
