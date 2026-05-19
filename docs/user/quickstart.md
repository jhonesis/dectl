# Quick Start — dectl

> > *Get started with dectl in 5 minutes.*

---

## Installation (2 minutes)

```bash
# Download the latest release binary
# Or build from source:
git clone https://github.com/jhonesis/dectl.git
cd dectl
cargo build --release
sudo cp target/release/dectl /usr/local/bin/
```

Check:
```bash
dectl --version
```

---

## Project Setup (3 minutes)

```bash
cd ~/projects/my-project

# Create .dec/ (hidden folder with context)
dectl project init --standard

# Configure
nano .dec/config/project.toml
nano .dec/isa/project.isa.md

# Check
dectl project info
```

All set! Your project now has context.

---

## Essential Commands

```bash
# View project status
dectl project info

# View files (respects .gitignore)
dectl project scan

# Generate context for AI
dectl project context > context.txt

# Add to memory
dectl memory add "Decision: use PostgreSQL"

# Search in memory
dectl memory search "PostgreSQL"

# List workflows
dectl workflow list
```

---

## With AI (Example)

```bash
# 1. Copy context
dectl project context | pbcopy

# 2. Paste in Claude/ChatGPT:
# "I'm working on [paste context]. 
#  Need to implement auth with JWT."

# 3. AI tells you what to do
# 4. You apply the changes
```

---

## Structure

```
my-project/
├── .dec/           ← context of dectl (hidden)
│   ├── config/
│   ├── isa/
│   └── workflows/
├── src/            ← your code
└── ...             ← project files
```

---

## Tips

- `.dec/` does not interfere with your code (hidden folder)
- the memory is global — works in all your projects
- you don't need AI to use dectl — it works from terminal

---

*For more details: [flow.md](./flow.md)*