# Instrucciones de Sesión — [Project Name]
> **Para el modelo**: Lee y sigue estas instrucciones en cada sesión de trabajo.
> Actualiza este archivo si el equipo quiere cambiar el comportamiento del modelo.

---

## Al iniciar sesión

1. Lee `.dec/config/project.toml` y `.dec/isa/project.isa.md` para entender el proyecto
2. Lee `.dec/state/last_session.md` y retoma desde "Próximo paso recomendado"
3. Ejecuta `dectl project info --json` y escala al developer si hay warnings
4. Confirma en 2-3 líneas qué entendiste antes de preguntar qué hacer hoy

## Antes de actuar

1. Para cambios de arquitectura: lee `.dec/decisions/` primero
2. Para implementar una feature: busca su workflow en `.dec/workflows/`
3. Para términos de dominio: consulta `.dec/knowledge/glossary.md` si existe
4. Describe lo que vas a hacer antes de hacerlo — nunca actúes en silencio

## Agentes disponibles

Usa `dectl agent list` para ver todos los agentes (built-in + custom).

El proyecto incluye estos agentes built-in:
- **coder**: implementa código siguiendo las convenciones del stack
- **reviewer**: revisa código en busca de bugs y desviaciones
- **researcher**: busca contexto en memoria y decisiones previas
- **documenter**: genera o actualiza documentación técnica

Para invocar un agente:
```
dectl agent run <tipo> --task "<descripción de la tarea>"
dectl agent describe <tipo>     # ver definición completa
dectl agent run --parallel <t1>,<t2> --task "<desc>"  # ejecutar en paralelo
```

Usa agentes cuando la tarea sea autónoma y especializada. El modelo principal mantiene el contexto global mientras el agente ejecuta.

## Al completar una tarea

1. Si completaste o avanzaste una feature: actualiza `.dec/state/progress.json`
2. Para decisiones importantes: ejecuta `dectl memory add "[resumen de la decisión]"`
3. Para decisiones arquitectónicas: crea `.dec/decisions/XXXX-nombre.md`

## Al finalizar sesión

1. Ejecuta `dectl session end` para automatizar el cierre:
   - Genera `.dec/state/last_session.md` automáticamente
   - Sincroniza cambios git a `progress.json`
   - Captura decisiones y las guarda en memoria
   - Sincroniza cambios del stack con `project.toml`
   - Registra actividad de agentes
2. O manualmente:
   - Escribe `.dec/state/last_session.md` (qué se hizo, qué quedó pendiente, decisiones, próximo paso)
   - Ejecuta `dectl memory add "Sesión [fecha]: [resumen en una línea]"`
