# Documentación para Desarrolladores — dectl

> *Guía para contribuir al desarrollo de dectl.*

## Estructura del Proyecto

```
dectl/
├── src/
│   ├── main.rs           # CLI completa
│   ├── core/             # Configuración y output
│   ├── project/          # Comandos de proyecto
│   ├── memory/           # CRUD de memoria
│   ├── protocol/         # exec-from-file
│   └── workflow/         # Módulo de workflows (Phase 2)
├── tests/                # Tests de integración
├── docs/                 # Documentación
└── specs/                # SDDs del proyecto
    ├── master/           # Visión general
    ├── dot-dec/          # Sistema .dec/
    ├── cli/              # CLI
    └── integration/      # Integración entre actores
```

## Comandos de Desarrollo

```bash
# Compilar
cargo build
cargo build --release

# Tests
cargo test
cargo test --test project_commands
cargo test --test memory_commands
cargo test --test protocol_commands

# Linting
cargo fmt
cargo clippy -- -D warnings

# Binary size
du -sh target/release/dectl  # debe ser < 20MB
```

## Phase 1 — Completado

- [x] Setup del proyecto
- [x] Core module (config, output)
- [x] Project commands (init, info, scan)
- [x] Memory module (add, list, search, show)
- [x] Protocol (exec-from-file)
- [x] Tests (31 passing)
- [x] Binary (3.6MB)

## Phase 2 — Pendiente

- [ ] D016-D020: Templates nivel 3
- [ ] C024-C032: Workflow commands
- [ ] I004-I006: Integration layer

Ver: [specs/cli/tasks.md](../specs/cli/tasks.md)

## Para Empezar a Desarrollar

1. Lee `CLAUDE.md` en la raíz
2. Lee `last_session.md` para estado actual
3. Lee `specs/integration/plan.md` antes de Phase 2
4. Ejecuta `cargo test` para verificar

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   .dec/     │    │    dectl    │    │   Modelo    │
│  (archivos) │───▶│  (binario)  │◀───│  (IA/CLI)   │
└─────────────┘    └──────┬──────┘    └─────────────┘
                          │
                   ┌──────▼──────┐
                   │ ~/.dectl/   │
                   │  memory.db  │
                   └─────────────┘
```

## Contributing

1. Elige una tarea de `specs/*/tasks.md`
2. Lee el spec y plan correspondiente
3. Implementa siguiendo los checkpoints
4. Tests deben pasar
5. `cargo fmt && cargo clippy` deben estar limpios

Ver: [CONTRIBUTING.md](../CONTRIBUTING.md)