# Technical Research — Integration Layer
> *Investiga las decisiones técnicas sobre el protocolo de integración entre actores.*
> *Last updated: 2026-05-13*

---

## Contexto

La integración no tiene código propio — es un contrato de comportamiento. Las decisiones aquí son sobre protocolos, formatos y patrones que deben funcionar igualmente bien con Claude Code, Gemini CLI, Qwen CLI, Phi, Ollama o cualquier entorno futuro. Las preguntas de investigación son sobre comportamiento observable, no implementación.

---

## Research Questions

### RQ-I-001: Mínimo común denominador entre entornos de IA

**Context**: REQ-I-007 requiere que dectl funcione con cualquier entorno que soporte lectura de archivos y ejecución de comandos. La pregunta es qué capacidades son universales y cuáles son específicas de algunos entornos.

**Entornos analizados**:

| Entorno | Lee archivos | Ejecuta comandos | Mantiene contexto entre invocaciones |
|---------|-------------|-----------------|-------------------------------------|
| Claude Code | ✅ nativo | ✅ nativo | ✅ dentro de sesión |
| Gemini CLI | ✅ nativo | ✅ nativo | ✅ dentro de sesión |
| Qwen CLI / Ollama + shell | ✅ via cat/read | ✅ via shell | ❌ depende de implementación |
| Cursor / Copilot Chat | ✅ nativo | ⚠️ via terminal integrado | ✅ dentro de sesión |
| Modelo puro via API | ❌ solo lo que se le pasa | ❌ no puede ejecutar | ❌ stateless |

**Mínimo común denominador identificado**:
- Leer archivos del filesystem → universal si el entorno tiene acceso al proyecto
- Ejecutar comandos de terminal → universal en entornos CLI-based
- Parsear JSON en stdout → universal (es texto)
- Verificar exit codes → universal en entornos que ejecutan comandos

**Decision**: El contrato de integración se define sobre el mínimo común denominador: leer archivos + ejecutar comandos + parsear JSON. Los entornos que no ejecutan comandos obtienen valor parcial (solo contexto de `.dec/`) — documentado en REQ-I-007.

---

### RQ-I-002: Formato óptimo para que el modelo genere comandos dectl

**Context**: REQ-I-003 requiere que el modelo genere comandos `dectl` como texto plano ejecutable. La pregunta es cómo estructurar este output para que sea parseable por el entorno y comprensible para el developer.

**Options Evaluated**:

| Formato | Ejemplo | Pros | Contras |
|---------|---------|------|---------|
| Texto plano en línea | `dectl memory add "decisión tomada"` | Universal, cualquier entorno lo ejecuta | Sin delimitación — difícil para el entorno saber qué ejecutar |
| Bloque de código markdown | ` ```bash\ndectl memory add "..."\n``` ` | Visual, delimitado, estándar en LLMs | Requiere que el entorno parsee markdown para ejecutar |
| Protocolo custom con tags | `<execute>dectl memory add "..."</execute>` | Delimitado, parseable | Requiere soporte específico del entorno |
| Formato exec-from-file | Genera archivo con comandos, luego ejecuta `dectl exec-from-file` | Batch, auditable, un solo punto de ejecución | Un paso extra para comandos simples |

**Decision**: Bloque de código markdown para comandos individuales; exec-from-file para secuencias de 3 o más comandos.

**Rationale**: Los bloques de código markdown son el estándar de facto en todos los entornos de codificación con IA — Claude Code, Gemini CLI, Cursor y similares los detectan y ejecutan automáticamente. Para secuencias largas, exec-from-file agrupa la intención y hace auditable qué se ejecutó. El developer puede leer el archivo antes de aprobar.

---

### RQ-I-003: Estructura óptima de `integration.md` para modelos 7B+

**Context**: El archivo `prompts/system/integration.md` define el ciclo de sesión por proyecto. Debe ser efectivo con modelos de 7B parámetros que tienen ventanas de contexto limitadas y seguimiento de instrucciones menos robusto que modelos grandes.

**Principios investigados para prompts efectivos con modelos pequeños**:

1. **Instrucciones como listas numeradas, no párrafos** — los modelos pequeños siguen mejor pasos explícitos que razonar sobre prosa
2. **Máximo 5-7 instrucciones por sección** — más instrucciones producen olvido o confusión
3. **Instrucciones positivas sobre negativas** — "haz X" funciona mejor que "no hagas Y"
4. **Anclas explícitas** — "ANTES de actuar", "AL FINALIZAR" reducen la ambigüedad de cuándo aplicar cada instrucción
5. **Sin jerga filosófica** — "mantén coherencia arquitectónica" es vago; "lee .dec/decisions/ antes de proponer cambios de arquitectura" es accionable

**Decision**: Template de `integration.md` con 4 secciones fijas: `## Al iniciar sesión`, `## Antes de actuar`, `## Al completar una tarea`, `## Al finalizar sesión`. Máximo 5 ítems por sección. Todo en lenguaje imperativo directo.

**Template resultante**:
```markdown
# Instrucciones de Sesión — [Nombre del Proyecto]
> El modelo debe leer y seguir estas instrucciones en cada sesión.

## Al iniciar sesión
1. Lee `.dec/config/project.toml` y `.dec/isa/project.isa.md`
2. Lee `.dec/state/last_session.md` y retoma desde "Próximo paso recomendado"
3. Ejecuta `dectl project info --json` y verifica que no haya warnings
4. Confirma al developer en 2-3 líneas qué entendiste del proyecto

## Antes de actuar
1. Para cambios arquitectónicos: lee `.dec/decisions/` primero
2. Para implementar features: busca el workflow en `.dec/workflows/`
3. Para términos de dominio: consulta `.dec/knowledge/glossary.md`
4. Describe lo que vas a hacer antes de hacerlo

## Al completar una tarea
1. Actualiza `.dec/state/progress.json` si completaste o avanzaste una feature
2. Registra decisiones importantes: `dectl memory add "..."`
3. Si tomaste una decisión arquitectónica: crea `.dec/decisions/XXXX-nombre.md`

## Al finalizar sesión
1. Escribe `.dec/state/last_session.md` con: qué hiciste, qué falta, próximo paso
2. Ejecuta `dectl memory add` con un resumen de la sesión
```

---

### RQ-I-004: Protocolo de verificación de comandos

**Context**: REQ-I-003 requiere que el modelo verifique el resultado de cada comando antes de continuar. La pregunta es cómo estructurar este comportamiento para que sea robusto con modelos pequeños que pueden olvidar verificar.

**Options Evaluated**:

| Approach | Pros | Contras |
|----------|------|---------|
| El modelo verifica manualmente después de cada comando | Explícito, trazable | Modelos pequeños lo olvidan frecuentemente |
| Siempre usar `--json` y el modelo parsea `status` | Fuerza verificación | Más verbose en comandos simples |
| Wrapper script que verifica y reporta | Automático | Introduce dependencia extra, rompe model-agnostic |
| Instrucción en integration.md de verificar siempre | Sin overhead técnico | Depende de que el modelo la siga |

**Decision**: Instrucción explícita en `integration.md` de usar `--json` en comandos cuyo resultado afecta el siguiente paso + verificar `status`. Para comandos de solo escritura (memory add, workflow run sin output necesario), el exit code es suficiente.

**Rationale**: La combinación de `--json` obligatorio para comandos de lectura + instrucción en `integration.md` cubre el 95% de los casos sin overhead técnico. El 5% restante (modelo que ignora la instrucción) se resuelve con la regla de dos reintentos antes de escalar.

---

### RQ-I-005: Manejo del estado entre sesiones en entornos stateless

**Context**: Algunos entornos (API pura, algunos CLIs de modelo) son completamente stateless — no retienen contexto entre invocaciones. El sistema debe seguir siendo útil en este escenario.

**Options Evaluated**:

| Approach | Pros | Contras |
|----------|------|---------|
| Documentar que el modo stateless no está soportado | Simple | Pierde una audiencia válida |
| Proveer un "context bundle" — un solo archivo que concatena todo `.dec/` | Un solo archivo para pasar al modelo | Puede ser muy largo para modelos con contexto limitado |
| Instrucciones en `integration.md` de qué incluir en el prompt inicial | Flexible, sin artefacto extra | El developer debe hacer el trabajo manual |
| Comando `dectl project context` que genera un resumen compacto | Automatizado, controlado en tamaño | Requiere implementación adicional en el CLI |

**Decision**: Agregar `dectl project context` como comando Phase 2 que genera un resumen compacto del proyecto para pasar como contexto a entornos stateless.

**Rationale**: El comando puede controlar el tamaño del output (máx. 4000 tokens por defecto, configurable), priorizar los archivos más importantes (ISA > decisions > state), y producir un formato que cualquier modelo puede consumir. Esto hace dectl útil incluso sin ejecución de comandos — un developer puede correr `dectl project context | pbcopy` y pegar el contexto en cualquier chat con IA.

---

## Resumen de Decisiones

| ID | Decisión | Resultado |
|----|---------|-----------|
| RQ-I-001 | Mínimo común denominador | Leer archivos + ejecutar comandos + parsear JSON |
| RQ-I-002 | Formato de comandos generados | Markdown code blocks; exec-from-file para 3+ comandos |
| RQ-I-003 | Estructura de integration.md | 4 secciones fijas, max 5 ítems, imperativo directo |
| RQ-I-004 | Verificación de comandos | `--json` obligatorio para comandos de lectura + integration.md |
| RQ-I-005 | Entornos stateless | `dectl project context` en Phase 2 |

---

## Impacto en Otros SDDs

| Cambio | SDD afectado | Acción requerida |
|--------|-------------|-----------------|
| Template de `integration.md` definido | dot-dec/plan.md | Agregar template al nivel 2 (D015b) |
| `dectl project context` nuevo comando | cli/spec.md, cli/tasks.md | Agregar REQ-C-013 y tarea C043 en Phase 2 |
| Formato exec-from-file para 3+ comandos | cli/spec.md REQ-C-011 | Ya cubierto — no requiere cambio |
