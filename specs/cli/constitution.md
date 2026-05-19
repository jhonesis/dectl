# Constitution — dectl CLI
> *Principios que gobiernan el diseño, implementación y evolución del binario `dectl`.*
> *Hereda y extiende: specs/master/constitution.md*
> *Última actualización: 2026-05-13*

---

## 1. Identidad

El CLI `dectl` es el cuerpo del sistema. Ejecuta lo que el modelo decide y el developer solicita. No piensa, no razona, no contiene lógica de negocio que dependa de un modelo de lenguaje. Su responsabilidad es exclusivamente ejecutar acciones reales en el sistema operativo de forma predecible, segura y reportable.

El CLI es un binario estático único. Se instala con un comando. Funciona sin configuración previa. El primer comando que ejecuta un developer nuevo debe ser útil.

---

## 2. Principios Fundamentales

**1. Un comando hace una cosa**
Cada subcomando tiene una responsabilidad única y bien definida. No hay comandos que combinen lectura + escritura + ejecución en un solo paso sin que el developer lo sepa explícitamente.

**2. Predecible ante todo**
El mismo comando con los mismos argumentos produce el mismo resultado siempre. Sin side effects ocultos. Sin comportamiento que varíe según el estado del sistema de formas no documentadas.

**3. Fallar rápido y claramente**
Si algo va a fallar, debe fallar en el primer paso posible con un mensaje que diga exactamente qué falló, por qué, y qué debe hacer el developer para resolverlo. Nunca fallar silenciosamente.

**4. El modelo puede leer cualquier output**
Todo output del CLI — incluyendo errores — debe ser parseable por un modelo de 7B+. Mensajes de error en lenguaje natural claro. `--json` disponible en todos los comandos para output estructurado.

**5. Cero configuración para empezar**
`dectl project init` es el único comando que requiere estar en un directorio de proyecto. Todo lo demás tiene defaults razonables que funcionan sin configuración previa.

**6. Extensible sin romper**
Nuevos subcomandos se agregan sin modificar los existentes. Nuevos flags son siempre opcionales. El contrato de cada comando existente es inmutable una vez publicado en una versión estable.

---

## 3. Reglas de Implementación

### Comandos
- Toda subcomando tiene `--help` con descripción, uso y al menos un ejemplo
- Todo subcomando soporta `--json` con envelope `{status: "ok"|"error", ...datos}`
- Los comandos de lectura (list, show, search, info, scan) son idempotentes — nunca modifican estado
- Los comandos de escritura (add, init, run, write) reportan exactamente qué cambió
- Ningún comando interactivo sin un flag `--yes` o `--non-interactive` para uso en scripts

### Errores
- Usar `anyhow` para propagación de errores en toda la codebase
- Nunca `panic!` en paths de producción — solo en tests y en `main()` como último recurso
- Mensajes de error: qué pasó + por qué + qué hacer. Nunca solo un código de error
- Exit codes: `0` = éxito, `1` = error de usuario (args inválidos, archivo no encontrado), `2` = error de sistema (permisos, disco lleno), `3` = error de estado (`.dec/` corrupto, schema incompatible)

### Output
- Human-readable por defecto: colores en terminal, no colores si stdout no es TTY
- `--json` produce JSON válido, minificado, sin trailing newline extra
- Nunca mezclar output informativo con output de datos en stdout — los logs van a stderr
- Progreso de operaciones largas va a stderr, no stdout

### Seguridad
- Toda escritura de archivo está restringida al directorio del proyecto o `~/.dectl/`
- Comandos que ejecutan código externo (workflow `action` steps) requieren trust explícito
- Nunca ejecutar comandos construidos con interpolación directa de strings de usuario sin sanitización

### Testing
- Todo módulo tiene tests unitarios en `#[cfg(test)]`
- Todo comando tiene al menos un integration test que invoca el binario real
- Los tests no dependen de red, modelo externo ni estado del sistema del desarrollador
- Fixtures en `tests/fixtures/` para `.dec/` de prueba y workflows de prueba

---

## 4. Estructura de Módulos

Cada módulo tiene una responsabilidad única. Un módulo no importa de otro módulo del mismo nivel — solo de `core`. La dependencia va hacia arriba, nunca entre hermanos.

```
main.rs          ← registra comandos, sin lógica
core/            ← config, output, errors — importado por todos
project/         ← solo lee/escribe el sistema de archivos del proyecto
memory/          ← solo interactúa con SQLite
workflow/        ← solo parsea y ejecuta workflows YAML
protocol/        ← solo lee y ejecuta archivos de comandos
```

---

## 5. Contrato de Versionado

- El CLI usa semver: `MAJOR.MINOR.PATCH`
- `PATCH`: bugfixes sin cambios de comportamiento observable
- `MINOR`: nuevos comandos o flags opcionales — backwards compatible
- `MAJOR`: cambios en comportamiento de comandos existentes o flags — requiere anuncio y migration guide
- El comando `dectl --version` siempre muestra versión y schema de `.dec/` soportado

---

## 6. Definition of Done para el CLI

Un comando está completo cuando:
- [ ] Implementa el requisito del spec sin desviaciones no documentadas
- [ ] `--help` es preciso, completo e incluye un ejemplo real
- [ ] `--json` output es válido y documentado en el spec
- [ ] Tests unitarios del módulo pasan con cobertura ≥ 70%
- [ ] Integration test del comando pasa invocando el binario real
- [ ] Sin warnings de `clippy` introducidos
- [ ] `rustfmt` aplicado
- [ ] Exit codes correctos para todos los caminos de error
