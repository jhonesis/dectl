# Strategic Notes — dectl
> *Decisiones de diseño, riesgos identificados y principios de producto.*
> *Este documento debe consultarse antes de tomar decisiones importantes sobre dirección, features o comunicación del proyecto.*
> *Last updated: 2026-06-02*

---

## Nombre del proyecto

**Decisión: `dectl`** (Dev Environment Control)

**Rationale**: `dec` es demasiado genérico — colisiona con un comando Unix estándar, con la abreviatura de diciembre, y con términos matemáticos. En búsquedas de GitHub o Google, `dec cli developer` no converge. `dectl` es corto, técnico, evoca `kubectl` y `systemctl` (herramientas que los developers ya asocian con control y orquestación), y es único y buscable.

**Implicaciones**:
- El binario se llama `dectl`
- El repositorio se llama `dectl`
- La carpeta del proyecto sigue siendo `.dec/` — es el nombre del sistema, no del binario
- Los comandos son `dectl project init`, `dectl memory add`, etc.

---

## Fortalezas reales del proyecto

### Separación de tres actores
Es la decisión arquitectónica más importante. Hace que dectl sea adoptable de forma incremental:
- Un developer puede usar solo `.dec/` sin instalar el CLI
- Otro puede usar el CLI sin implementar todos los workflows
- Un equipo puede compartir `.dec/` en git con memoria personal separada en `~/.dectl/`

Esta adoptabilidad incremental es lo que PAI nunca tuvo y es una ventaja competitiva concreta.

### Local-first como propuesta de valor real
No es solo filosofía. Hay casos de uso concretos que lo justifican:
- Proyectos con código propietario o datos sensibles
- Empresas con restricciones de seguridad sobre qué sale a la red
- Developers en regiones con conectividad limitada o costosa
- Cualquiera que no quiera depender de la disponibilidad de una API externa

### Model-agnostic como diferenciador
PAI tiene 13k stars pero está muerto para cualquiera que no use Claude Code en macOS. dectl funciona con cualquier modelo que pueda leer archivos y ejecutar comandos. Eso multiplica la audiencia potencial por un orden de magnitud.

---

## Riesgos identificados

### Riesgo 1 — Onboarding complicado mata la adopción
**Descripción**: Si un developer necesita leer documentación, configurar archivos manualmente o entender la arquitectura antes de ver valor, no adopta la herramienta. PAI cayó en esta trampa.

**Criterio de éxito**: `dectl project init` debe producir un `.dec/` con contexto útil en menos de 30 segundos, y el modelo debe poder trabajar mejor inmediatamente — sin configuración adicional.

**Implicación de diseño**: Los templates que genera `dectl project init` son el producto más importante del proyecto. Deben ser inteligentes, concretos y útiles desde el primer momento. No placeholders, no comentarios genéricos.

**Acción**: Diseñar el onboarding como criterio de aceptación desde Phase 1, no como feature de Phase 3.

---

### Riesgo 2 — Falta de caso de uso ancla
**Descripción**: Sin una historia concreta de principio a fin, los developers no entienden qué problema resuelve dectl en su vida diaria. Las features técnicas no venden — los momentos concretos sí.

**El momento ancla a diseñar**:
> "Abrí un proyecto legacy que no tocaba hace 6 meses, corrí `dectl project init`, le pedí al modelo que explicara la arquitectura, y en 2 minutos tenía contexto completo sin explicar nada."

**Implicación de diseño**: Este momento debe ser posible en Phase 1. Debe ser el primer ejemplo del README y el guión de cualquier demo o video.

**Acción**: Construir este escenario como test de aceptación end-to-end antes de declarar Phase 1 completo.

---

### Riesgo 3 — `.dec/` sin contrato público bloquea el ecosistema
**Descripción**: PAI nunca definió un contrato explícito para su estructura interna. Por eso nadie construyó herramientas compatibles, plugins ni integraciones. Todo el valor quedó atrapado dentro del proyecto original.

**Oportunidad**: Si el schema de `.dec/` está especificado públicamente desde el día uno — qué archivos existen, qué campos son obligatorios, cómo evoluciona — otros pueden construir sobre él. Plugins para editores, integraciones con herramientas de proyecto management, dashboards alternativos.

**Acción**: El SDD de `.dec/` debe ser público antes de tener código. Es el documento más importante del proyecto desde la perspectiva de ecosistema.

---

### Riesgo 4 — Complejidad creciente de `.dec/` aleja a usuarios nuevos
**Descripción**: A medida que el sistema crece, `.dec/` puede volverse intimidante. Un developer que abre un proyecto ajeno y ve 7 carpetas con 15 archivos Markdown puede sentir que es demasiado.

**Mitigación**: Diseñar `.dec/` con niveles. El nivel mínimo útil es solo `config/project.toml` y `isa/project.isa.md`. Todo lo demás es opcional y se agrega cuando se necesita. `dectl project init` crea solo el mínimo por defecto, con un flag `--full` para la estructura completa.

---

## Principios de producto

Estos principios deben guiar decisiones de diseño cuando haya trade-offs:

**1. Útil antes de completo**
Una feature que funciona en el 80% de los casos y se instala en 30 segundos vale más que una feature perfecta que requiere configuración. Siempre priorizar el camino feliz.

**2. El schema de `.dec/` es sagrado**
Una vez que el schema de `.dec/` es público, los cambios breaking destruyen la confianza de la comunidad. Versionar el schema desde el día uno. Nunca modificar, solo extender.

**3. El modelo es un ciudadano de primera clase, no un usuario secundario**
Cada comando, cada archivo de `.dec/`, cada mensaje de error debe ser legible y accionable tanto por un humano como por un modelo. Si un modelo no puede entender un mensaje de error, el mensaje de error está mal.

**4. Cero magia**
dectl no debe hacer nada que el developer no pueda entender leyendo el código o los archivos. Sin side effects ocultos, sin comportamiento sorpresivo. Lo que ves en `.dec/` es lo que pasa.

**5. Portable por defecto**
Un proyecto con `.dec/` debe funcionar en cualquier máquina con dectl instalado, sin configuración adicional. La memoria personal va en `~/.dectl/`, nunca en `.dec/`.

---

## Oportunidades no exploradas aún

### dectl como estándar abierto
Si dectl gana tracción, `.dec/` puede convertirse en un estándar de facto para context-aware AI development — similar a lo que `.editorconfig` hizo para estilos de código. Esto requiere documentación de referencia clara y un proceso de versionado explícito.

### Integración con herramientas existentes
- **Git hooks**: actualizar `.dec/state/progress.json` automáticamente en cada commit
- **CI/CD**: leer `.dec/` para generar contexto de revisión automática de PRs
- **Editores**: extensión para VS Code / Neovim que muestra el contexto de `.dec/` en un panel lateral

### Templates de comunidad
Un repositorio de templates `.dec/` por tipo de proyecto (API REST, CLI tool, microservicio, librería) que la comunidad pueda contribuir y reutilizar. Similar al ecosistema de `.gitignore` templates de GitHub.

---

## Decisiones resueltas

- [x] `dectl project init` crea estructura mínima por defecto, con `--standard` y `--full` flags para niveles superiores — resuelto en implementación
- [x] Schema versionado desde Phase 1 con `schema_version` en `project.toml` y `dectl migrate` tool — resuelto en Phase 4
- [x] Escenario ancla diseñado e implementado como test end-to-end (`tests/e2e_anchor.rs`) — completado en Phase 7
- [x] README principal combina quickstart ancla + enlaces a docs/ detalladas — resuelto en Phase 7
