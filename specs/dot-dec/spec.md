# Specification — .dec/ System
> *Technology-agnostic. Describe QUÉ es .dec/ y qué debe lograr, no cómo se implementa.*
> *Version: 1.0 | Status: Draft | Last updated: 2026-05-13*

---

## Overview

`.dec/` es una estructura de carpetas y archivos que vive dentro de cada proyecto de software. Define el contexto completo del proyecto de forma que cualquier modelo de lenguaje o developer pueda entenderlo, trabajar con él y mantenerlo actualizado — sin depender de ninguna herramienta específica, sin conexión a internet, y sin configuración adicional.

`.dec/` no ejecuta nada. Es información estructurada. Su valor está en la calidad y organización del contexto que provee.

---

## Usuarios

- **Developer**: crea, mantiene y evoluciona `.dec/` a lo largo de la vida del proyecto
- **Modelo de lenguaje**: consume `.dec/` para obtener contexto antes de actuar; actualiza archivos al completar tareas
- **CLI dectl**: lee y escribe `.dec/` para ejecutar workflows, registrar decisiones y actualizar estado
- **Colaborador nuevo**: llega a un proyecto con `.dec/` y entiende el contexto sin intervención humana

---

## Requisitos Funcionales

### REQ-D-001: Configuración del proyecto

**User Story**:
> Como developer, quiero un archivo de configuración del proyecto dentro de `.dec/` para que cualquier modelo o herramienta entienda qué se está construyendo sin que yo tenga que explicarlo en cada sesión.

**Acceptance Criteria**:
- WHEN existe `.dec/config/project.toml` THEN cualquier consumidor SHALL poder leer el nombre, tipo, stack tecnológico y versión del schema del proyecto
- WHEN el campo `schema_version` está ausente THEN el CLI SHALL advertir al developer y sugerir ejecutar una migración
- WHEN el proyecto tiene convenciones especiales THEN el archivo SHALL proveer un campo para documentarlas de forma que el modelo las respete
- WHEN un developer abre el archivo por primera vez THEN SHALL entender qué campo representa qué sin necesidad de documentación externa

---

### REQ-D-002: Artefactos de Estado Ideal (ISA)

**User Story**:
> Como modelo de lenguaje, quiero leer documentos que describan el estado ideal del proyecto para tomar decisiones coherentes con la visión del developer sin necesitar que me la explique cada vez.

**Acceptance Criteria**:
- WHEN existe `.dec/isa/project.isa.md` THEN el modelo SHALL poder leer visión, objetivos, alcance, no-objetivos y riesgos del proyecto
- WHEN existe `.dec/isa/architecture.isa.md` THEN el modelo SHALL poder leer la arquitectura ideal, módulos, flujos principales y decisiones de diseño clave
- WHEN el modelo va a tomar una decisión que afecta la arquitectura THEN los ISA SHALL ser el primer documento de referencia
- WHEN un ISA está desactualizado respecto al estado real THEN el developer o modelo SHALL poder actualizarlo directamente editando el archivo
- WHEN un ISA es creado por primera vez THEN SHALL incluir secciones guía que indiquen al modelo qué escribir en cada parte

---

### REQ-D-003: Registro de Decisiones

**User Story**:
> Como developer, quiero registrar las decisiones técnicas importantes junto con su contexto y alternativas consideradas para que el modelo nunca proponga cambios que contradigan decisiones ya tomadas.

**Acceptance Criteria**:
- WHEN existe un archivo en `.dec/decisions/` THEN SHALL seguir un formato que incluya contexto, decisión tomada, alternativas consideradas, justificación y consecuencias
- WHEN el modelo va a proponer un cambio arquitectónico THEN SHALL consultar `.dec/decisions/` primero
- WHEN se registra una nueva decisión THEN SHALL asignársele un identificador secuencial único (ej. `0001-nombre.md`)
- WHEN hay múltiples decisiones THEN el developer o modelo SHALL poder entender cada una de forma independiente sin leer las demás
- WHEN una decisión queda obsoleta THEN SHALL poder marcarse como tal sin eliminarse, preservando la historia

---

### REQ-D-004: Workflows Declarativos

**User Story**:
> Como developer o modelo, quiero definir workflows como secuencias de pasos declarativos para que tareas complejas y recurrentes se ejecuten de forma consistente sin depender de memoria o instrucciones verbales.

**Acceptance Criteria**:
- WHEN existe un archivo en `.dec/workflows/` THEN SHALL ser un workflow válido con nombre, descripción y al menos un paso
- WHEN un workflow tiene pasos de tipo `prompt` THEN el contenido SHALL ser una instrucción clara y autosuficiente que el modelo pueda seguir sin contexto adicional
- WHEN un workflow tiene pasos de tipo `action` THEN SHALL especificar el comando exacto a ejecutar
- WHEN un workflow tiene pasos de tipo `write` THEN SHALL especificar la ruta del archivo destino y el contenido a escribir
- WHEN el developer lista los workflows disponibles THEN SHALL ver nombre y descripción de cada uno sin abrir los archivos
- WHEN un modelo lee un workflow THEN SHALL entender el objetivo completo del workflow solo con su `description` y sus pasos
- WHEN un workflow define variables de entrada THEN SHALL declararlas explícitamente con nombre, descripción y si son obligatorias u opcionales
- WHEN un workflow es ejecutado con variables THEN los valores SHALL poder interpolarse dentro de cualquier campo de cualquier paso usando la sintaxis `{{nombre_variable}}`
- WHEN se ejecuta un workflow con variables obligatorias sin proveer valores THEN el CLI SHALL abortar e informar qué variables faltan

---

### REQ-D-005: Prompts del Sistema

**User Story**:
> Como modelo de lenguaje, quiero leer prompts predefinidos del proyecto para mantener consistencia en estilo, calidad y convenciones a lo largo de todas las sesiones sin que el developer las repita.

**Acceptance Criteria**:
- WHEN existe `.dec/prompts/system/base.md` THEN SHALL contener las instrucciones base que definen el comportamiento del modelo dentro del proyecto
- WHEN existen archivos en `.dec/prompts/tasks/` THEN cada uno SHALL definir instrucciones específicas para una tarea concreta (implementar feature, revisar código, escribir tests, etc.)
- WHEN el developer quiere que el modelo siga una convención nueva THEN SHALL poder agregarla a los prompts sin reiniciar nada
- WHEN un prompt hace referencia a convenciones del proyecto THEN SHALL apuntar a los archivos de `.dec/` donde esas convenciones están definidas, no repetirlas

---

### REQ-D-006: Base de Conocimiento del Dominio

**User Story**:
> Como modelo de lenguaje, quiero acceder a conocimiento específico del dominio del proyecto para no inventar términos, reglas de negocio ni restricciones que el developer no ha definido explícitamente.

**Acceptance Criteria**:
- WHEN existe `.dec/knowledge/glossary.md` THEN SHALL contener definiciones de términos del dominio que el modelo debe usar de forma consistente
- WHEN existe `.dec/knowledge/constraints.md` THEN SHALL contener limitaciones técnicas, requisitos no funcionales y restricciones del sistema
- WHEN el modelo encuentra un término no definido en el glosario THEN SHALL preguntar al developer antes de asumir su significado
- WHEN el developer agrega una regla de negocio nueva THEN SHALL poder documentarla en knowledge sin impactar otros archivos

---

### REQ-D-007: Estado del Proyecto

**User Story**:
> Como developer o modelo, quiero leer y actualizar el estado de avance del proyecto para que cualquier sesión nueva pueda retomar el trabajo exactamente donde se dejó sin repetir contexto verbal.

**Acceptance Criteria**:
- WHEN existe `.dec/state/progress.json` THEN SHALL listar features con su estado (`pending`, `in_progress`, `done`, `blocked`) y notas opcionales
- WHEN el modelo completa una tarea THEN SHALL actualizar `progress.json` para reflejar el nuevo estado
- WHEN existe `.dec/state/last_session.md` THEN SHALL contener un resumen de la última sesión: qué se hizo, qué quedó pendiente y qué decisiones se tomaron
- WHEN el developer inicia una nueva sesión THEN leer `last_session.md` SHALL ser suficiente para retomar el contexto sin revisión adicional

---

### REQ-D-008: Auto-documentación

**User Story**:
> Como developer que llega a un proyecto con `.dec/` por primera vez, quiero que cada archivo me indique su propósito y cómo usarlo para no necesitar documentación externa ni explicaciones del autor original.

**Acceptance Criteria**:
- WHEN cualquier archivo de `.dec/` es abierto THEN SHALL contener en su encabezado una descripción de su propósito y cómo debe ser usado por el modelo o el developer
- WHEN un archivo tiene campos o secciones opcionales THEN SHALL indicar explícitamente que son opcionales y cuándo usarlos
- WHEN un workflow tiene pasos complejos THEN cada paso SHALL tener un campo `description` que explique su intención
- WHEN `.dec/` es inicializado THEN cada archivo creado SHALL ser útil inmediatamente, no un placeholder vacío

---

### REQ-D-009: Versionado del Schema

**User Story**:
> Como developer que usa dectl en múltiples proyectos a lo largo del tiempo, quiero que `.dec/` indique explícitamente qué versión del schema usa para que el CLI pueda advertirme si hay incompatibilidades.

**Acceptance Criteria**:
- WHEN existe `.dec/config/project.toml` THEN SHALL contener un campo `schema_version` con la versión del schema de `.dec/` en uso
- WHEN la versión del schema del proyecto es mayor a la que soporta el CLI THEN el CLI SHALL advertir al developer con un mensaje claro antes de ejecutar cualquier operación
- WHEN hay una nueva versión del schema disponible THEN el CLI SHALL informar al developer y ofrecer una ruta de migración

---

### REQ-D-010: Niveles de Adopción

**User Story**:
> Como developer que empieza a usar dectl, quiero poder adoptar `.dec/` de forma incremental para no sentirme abrumado con una estructura compleja desde el primer día.

**Acceptance Criteria**:
- WHEN el developer inicializa un proyecto sin flags THEN SHALL crearse solo la estructura mínima (nivel 1): `config/project.toml` y `isa/project.isa.md`
- WHEN el developer inicializa con flag estándar THEN SHALL crearse la estructura nivel 2, que agrega decisiones, workflows base, prompt del sistema y estado
- WHEN el developer inicializa con flag completo THEN SHALL crearse la estructura nivel 3 con todos los archivos y carpetas
- WHEN un developer trabaja con nivel 1 THEN el sistema SHALL ser completamente funcional — ninguna feature core de dectl SHALL requerir nivel 2 o 3 para operar

---

## Requisitos No Funcionales

- **Legibilidad**: cualquier archivo de `.dec/` debe ser comprensible por un developer sin experiencia previa con dectl en menos de 2 minutos de lectura
- **Tamaño inicial**: ningún archivo template debe exceder 2000 tokens en su estado inicial
- **Compatibilidad de modelos**: `.dec/` debe proveer valor real con modelos de 7B parámetros o superiores
- **Portabilidad**: `.dec/` debe funcionar idénticamente copiado a cualquier máquina con dectl instalado
- **Compatibilidad con git**: `.dec/` completo debe ser commitable sin riesgo; no debe contener datos personales ni secretos

---

## Fuera de Alcance

- Ejecución de código o comandos (responsabilidad del CLI)
- Almacenamiento de memoria persistente entre proyectos (responsabilidad de `~/.dectl/memory.db`)
- Sincronización remota o colaboración en tiempo real
- Interfaz gráfica o dashboard (fase posterior)
- Soporte para lenguajes de markup distintos a Markdown para documentos de contexto

---

## Preguntas Abiertas

- [x] ¿Debe `.dec/` incluir un archivo `README.md` propio? → **No. El `CLAUDE.md` del proyecto es suficiente.**
- [x] ¿Los workflows soportan variables/parámetros de entrada en v1? → **Sí. Los workflows deben soportar variables de entrada desde v1.**
