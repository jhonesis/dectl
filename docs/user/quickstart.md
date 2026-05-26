# Quickstart — dectl

## Scenario 1: Legacy project (the anchor moment)

```bash
git clone https://github.com/your/legacy-project.git
cd legacy-project
dectl project init --standard
# Open your favorite AI (Claude Code, Gemini CLI, etc.)
# Ask: "Explain the architecture of this project"
# The AI reads .dec/ and responds with precise context.
```

## Scenario 2: New project

```bash
mkdir my-new-project && cd my-new-project
dectl project init --standard
# Answer the interactive questions about stack and purpose
# Start coding. The AI already knows what you're building.
```

## Scenario 3: Team collaboration

```bash
# The .dec/ is shared in git (like .editorconfig)
git add .dec/
git commit -m "Add .dec/ project context"

# Each member has their own personal memory
# ~/.dectl/memory.db — never compete over context
```
