# Quick Start — dectl

> *Para empezar a usar dectl en 5 minutos.*

---

## Instalación (2 minutos)

```bash
# Descarga el binario de la última release
# O compila desde código:
git clone https://github.com/tu/repo-dectl.git
cd repo-dectl/dectl
cargo build --release
sudo cp target/release/dectl /usr/local/bin/
```

Verifica:
```bash
dectl --version
```

---

## Setup de Proyecto (3 minutos)

```bash
cd ~/proyectos/mi-proyecto

# Crear .dec/ (carpeta oculta con contexto)
dectl project init --standard

# Configurar
nano .dec/config/project.toml
nano .dec/isa/project.isa.md

# Verificar
dectl project info
```

¡Listo! Tu proyecto ahora tiene contexto.

---

## Comandos Esenciales

```bash
# Ver estado del proyecto
dectl project info

# Ver archivos (respeta .gitignore)
dectl project scan

# Generar contexto para IA
dectl project context > contexto.txt

# Agregar a memoria
dectl memory add "Decisión: usar PostgreSQL"

# Buscar en memoria
dectl memory search "PostgreSQL"

# Listar workflows
dectl workflow list
```

---

## Con IA (Ejemplo)

```bash
# 1. Copiar contexto
dectl project context | pbcopy

# 2. Pegar en Claude/ChatGPT:
# "Estoy trabajando en [pegar contexto]. 
#  Necesito implementar auth con JWT."

# 3. IA te dice qué hacer
# 4. Tú aplicas los cambios
```

---

## Estructura

```
mi-proyecto/
├── .dec/           ← contexto de dectl (oculto)
│   ├── config/
│   ├── isa/
│   └── workflows/
├── src/            ← tu código
└── ...             ← archivos del proyecto
```

---

## Tips

- `.dec/` no interfiere con tu código (carpeta oculta)
- La memoria es global — funciona en todos tus proyectos
- No necesitas IA para usar dectl — funciona desde terminal

---

*Para más detalles: [flujo.md](./flujo.md)*