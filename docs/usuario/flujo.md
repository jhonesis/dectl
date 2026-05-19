# Guía de Usuario — dectl

> *Cómo usar dectl en tu proyecto de desarrollo.*

---

## ¿Qué es dectl?

dectl es un **Developer Life OS** que te da a ti (y a cualquier IA que uses) memoria persistente, workflows ejecutables y contexto estructurado de proyecto.

**Tres conceptos clave:**
- `.dec/` — carpeta oculta donde vive todo el contexto de tu proyecto
- `dectl` — el binario que ejecuta comandos
- Memoria global — conocimiento que comparte entre proyectos

---

## Guía Rápida para Nuevo Proyecto

### Paso 1: Instalar dectl

```bash
# Opción A: Descargar binario pre-compilado
# (mover a tu PATH, ej: ~/bin/dectl o /usr/local/bin/dectl)

# Opción B: Compilar desde código
git clone https://github.com/tu/repo-dectl.git
cd repo-dectl/dectl
cargo build --release
sudo cp target/release/dectl /usr/local/bin/
```

Verifica la instalación:
```bash
dectl --version
# → dectl 0.1.0
```

---

### Paso 2: Crear `.dec/` en tu proyecto

```bash
cd ~/proyectos/mi-proyecto

dectl project init --standard
```

**¿Qué hace este comando?**
- Crea la carpeta `.dec/` (oculta, no interfiere con tu código)
- Crea archivos de configuración, visión, workflows, etc.
- No modifica ningún archivo de tu proyecto

**Niveles de inicialización:**

| Nivel | Comando | Cuándo usarlo |
|-------|---------|--------------|
| Minimal | `dectl project init` | Prototipos rápidos |
| Estándar | `dectl project init --standard` | **Recomendado** — proyectos reales |
| Completo | `dectl project init --full` | Proyectos grandes con arquitectura definida |

---

### Paso 3: Configurar tu proyecto

Edita los archivos creados en `.dec/`:

#### 3.1 Configurar `project.toml`

```bash
nano .dec/config/project.toml
```

Ejemplo:
```toml
[dec]
schema_version = "1.0"

[project]
name = "mi-proyecto"
type = "web-app"
description = "Descripción breve de tu proyecto"

[stack]
languages = ["typescript"]
frameworks = ["nextjs"]
databases = ["postgresql"]
tools = ["docker", "git"]
```

#### 3.2 Definir visión en `project.isa.md`

```bash
nano .dec/isa/project.isa.md
```

Ejemplo:
```markdown
# Visión
Breve descripción de qué hace tu proyecto y para quién.

# Objetivo principal
El objetivo más importante que resuelve tu proyecto.

# Stack técnico
- Frontend: ...
- Backend: ...
- DB: ...

# Conventions
- Commits en español/inglés
- Convenciones de código que sigue el equipo
```

---

### Paso 4: Empezar a trabajar

#### Ver estado del proyecto
```bash
dectl project info
```
Muestra: nombre, stack, visión y advertencias si algo falta.

#### Ver estructura de archivos
```bash
dectl project scan --depth 3
```
Muestra el árbol de archivos respetando `.gitignore`.

#### Generar contexto para IA (entorno stateless)
```bash
dectl project context --max-tokens 3000 | pbcopy
```
Copia un resumen compacto para pegar en ChatGPT/Claude.

---

## Uso con IA (Claude Code, etc.)

### Flujo 1: IA tiene acceso a comandos

La IA puede ejecutar comandos directamente:
```bash
dectl workflow list                    # ver workflows disponibles
dectl workflow run implement_feature --var feature_name=user_auth
dectl memory add "Decisión: usar JWT para auth"
```

### Flujo 2: IA sin acceso a comandos (stateless)

```bash
# 1. Generar contexto
dectl project context > contexto.txt

# 2. Copiar y pegar en la IA
# "Aquí está mi proyecto: [pegar contenido]"

# 3. IA responde con sugerencias
# 4. Tú aplicas los cambios manualmente
```

---

## Comandos Principales

### Gestión de Proyecto
```bash
dectl project init [--standard|--full]  # Crear .dec/
dectl project info                       # Ver estado
dectl project scan [--depth N]           # Ver archivos
dectl project context [--max-tokens N] [--format text|json]  # Resumen para IA
```

### Memoria
```bash
dectl memory add "contenido"                    # Agregar a memoria
dectl memory add "contenido" --tags tag1,tag2    # Con tags
dectl memory list                               # Ver entradas
dectl memory search "palabra"                   # Buscar
dectl memory show 1                             # Ver entrada específica
```

### Workflows
```bash
dectl workflow list                     # Listar workflows
dectl workflow describe implement_feature  # Ver detalle
dectl workflow run implement_feature --var nombre=valor  # Ejecutar
dectl workflow run nombre --dry-run     # Previsualizar sin ejecutar
```

### Protocolo
```bash
dectl exec-from-file workflow.txt  # Ejecutar comandos desde archivo
```

### Flags Globales
```bash
dectl --json              # Salida en JSON
dectl --non-interactive   # Sin prompts interactivos
dectl --help              # Ayuda
dectl --version           # Versión
```

---

## Estructura de `.dec/`

```
.dec/
├── config/
│   └── project.toml           # Configuración del proyecto
├── isa/
│   └── project.isa.md         # Visión y objetivo
├── workflows/
│   ├── implement_feature.yaml # Workflow para features
│   └── design_architecture.yaml # Workflow para arquitectura
├── prompts/
│   └── system/
│       ├── base.md            # Instrucciones base para IA
│       └── integration.md     # Protocolo de sesión
├── state/
│   ├── progress.json          # Estado de features
│   └── last_session.md        # Resumen de última sesión
├── decisions/                  # Decisiones arquitectónicas
├── knowledge/                  # (nivel 3) Glosario y constraints
├── .gitignore                 # No se sube a git por defecto
└── README.md                  # Explicación del contexto
```

---

## Ejemplo: Pepito y su proyecto "Tornillo"

### Día 1: Setup

```bash
# Pepito clona su repositorio
git clone https://github.com/pepito/tornillo.git
cd tornillo

# Crea .dec/
dectl project init --standard

# Configura
nano .dec/config/project.toml
nano .dec/isa/project.isa.md

# Verifica
dectl project info
```

### Día 2: Primera sesión con IA

```bash
# Genera contexto
dectl project context > contexto.txt

# Copia y pega en Claude:
# "Estoy trabajando en [pegar contexto]. Necesito implementar auth."

# Claude responde con plan
# Pepito aplica los cambios
```

### Día 3: Continuar trabajo

```bash
# Ver qué quedó pendiente
cat .dec/state/last_session.md

# Ejecutar workflow
dectl workflow run implement_feature --var feature_name=inventory_crud --var module=src/inventory

# Agregar decisión a memoria
dectl memory add "Decisión: usar PostgreSQL para inventario, no Redis"
```

---

## Comandos que funcionan sin `.dec/`

- `dectl memory add "..."` — memoria global
- `dectl --version` — información del CLI
- `dectl exec-from-file <path>` — ejecutar script

## Comandos que requieren `.dec/`

- `dectl project info` — necesita configuración
- `dectl project scan` — escanea el proyecto
- `dectl workflow list` — busca en `.dec/workflows/`

---

## Tips

1. **`.dec/` no se sube a git** — tiene `.gitignore` que lo excluye
   - Si quieres versionarlo, quadrágnalo del `.gitignore`

2. **La memoria es global** — `~/.dectl/memory.db`
   - Funciona en cualquier proyecto
   - Comparte conocimiento entre proyectos

3. **Workflows son opcionales** — no obligan a seguir un flujo

4. **El `.dec/` no interfiere con tu código** — está en carpeta oculta

---

## Troubleshooting

### ".dec/ already exists"
Ya tienes un contexto creado. Para re-crear: `rm -rf .dec/`

### "No such file or directory" en dectl
El binario no está en tu PATH. Verifica:
```bash
which dectl
export PATH=$PATH:~/bin  # o donde esté el binario
```

### Proyecto sin `.dec/` y quiero uno
```bash
dectl project init --standard
```

### Quiero ver mi contexto actual
```bash
dectl project info --json | jq
```

---

## Próximos Pasos

1. Configura tu `.dec/` con los datos de tu proyecto
2. Ejecuta `dectl project info` para verificar
3. Lee `.dec/prompts/system/integration.md` para entender el protocolo de sesión
4. Explora los workflows disponibles: `dectl workflow list`

---

*Versión: 0.1.0*
*Documentación: `/docs/usuario.md`*