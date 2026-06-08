#!/usr/bin/env bash
set -e

# ============================================
# dectl — Interactive Walkthrough Demo
# Press Enter between each step to proceed.
# ============================================

pause() {
  echo ""
  read -p "Press Enter to continue..."
  echo ""
}

step_header() {
  echo ""
  echo "╔══════════════════════════════════════════════════╗"
  echo "║  $1"
  echo "╚══════════════════════════════════════════════════╝"
}

# ── Check dectl is installed ──
if ! command -v dectl &>/dev/null; then
  echo "📦 dectl not found. Installing..."
  curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/scripts/install.sh | bash
fi

VERSION=$(dectl --version 2>/dev/null || echo "installed")
echo "✅ dectl $VERSION"
pause

# ── Step 0: Create a dummy Node.js project ──
step_header "Step 0: Create a sample Node.js project"
TMPDIR=$(mktemp -d)
cd "$TMPDIR"
echo "  Creating a minimal Node.js project in $TMPDIR"
echo ""
echo '{"name":"demo-app","version":"1.0.0","description":"A simple Node.js API"}' > package.json
cat > index.js << 'EOF'
const http = require('http');
const server = http.createServer((req, res) => {
  res.end('Hello from dectl demo');
});
server.listen(3000);
EOF
echo "  Files created:"
echo "    - package.json"
echo "    - index.js"
pause

# ── Step 1: init --standard ──
step_header "Step 1: dectl project init --standard"
echo "  Creates .dec/ structure and auto-detects project stack."
echo "  Since this is a non-empty project, dectl will detect Node.js."
echo ""
echo "  > dectl project init --standard"
pause
dectl project init --standard
echo ""
echo "  ✅ .dec/ created. The AI detects: Node.js"
pause

# ── Step 2: project info ──
step_header "Step 2: dectl project info"
echo "  Shows the project context in human-readable format."
echo ""
echo "  > dectl project info"
pause
dectl project info
pause

# ── Step 3: project info --json ──
step_header "Step 3: dectl project info --json"
echo "  This is what the AI model reads automatically when"
echo "  you open the project in any AI-capable terminal."
echo ""
echo "  > dectl project info --json | python3 -m json.tool"
pause
dectl project info --json | python3 -m json.tool 2>/dev/null || dectl project info --json
echo ""
echo "  💡 The AI sees languages, frameworks, description, and decisions."
pause

# ── Step 4: memory add ──
step_header "Step 4: dectl memory add"
echo "  Stores persistent context that survives across sessions."
echo ""
echo "  > dectl memory add \"This demo uses Node.js for the API layer\" --tags demo,architecture"
echo "  > dectl memory add \"Chose CommonJS for simplicity\" --tags decisions"
pause
dectl memory add "This demo uses Node.js for the API layer" --tags demo,architecture
dectl memory add "Chose CommonJS for simplicity" --tags decisions
echo ""
echo "  ✅ 2 memories added."
pause

# ── Step 5: memory list ──
step_header "Step 5: dectl memory list"
echo "  Lists all stored memories for the current project."
echo ""
echo "  > dectl memory list --limit 5"
pause
dectl memory list --limit 5
pause

# ── Step 6: memory search ──
step_header "Step 6: dectl memory search"
echo "  Full-text search across all memories."
echo ""
echo "  > dectl memory search \"architecture\""
pause
dectl memory search "architecture"
pause

# ── Step 7: workflow list ──
step_header "Step 7: dectl workflow list"
echo "  Lists available workflow pipelines in .dec/workflows/"
echo ""
echo "  > dectl workflow list"
pause
dectl workflow list
pause

# ── Step 8: agent list ──
step_header "Step 8: dectl agent list"
echo "  Lists built-in and custom agents available."
echo ""
echo "  > dectl agent list"
pause
dectl agent list
pause

# ── Step 9: agent describe ──
step_header "Step 9: dectl agent describe researcher"
echo "  Shows agent definition: role, steps, and inputs."
echo ""
echo "  > dectl agent describe researcher"
pause
dectl agent describe researcher
pause

# ── Step 10: project context --compact ──
step_header "Step 10: dectl project context --compact"
echo "  A compressed summary for stateless AI environments."
echo ""
echo "  > dectl project context --format compact"
pause
dectl project context --format compact
pause

# ── Step 11: opencode ──
step_header "Step 11: Test the AI-First Flow"
echo "  The magic of dectl: open this project inside an AI tool."
echo ""
echo "  Run this command in a SEPARATE terminal:"
echo ""
echo "    opencode $TMPDIR"
echo ""
echo "  Inside opencode, type:"
echo ""
echo "    dectl project info --json"
echo ""
echo "  The AI will already know this is a Node.js project with"
echo "  the description and decisions you stored. No manual setup needed."
echo ""
echo "  (Close this walkthrough or leave it open as reference.)"
echo ""
pause

# ── Step 12: cleanup ──
step_header "Step 12: Cleanup"
echo "  Remove the temporary project."
echo ""
echo "  > rm -rf $TMPDIR"
echo ""
read -p "Delete temp project? (Y/n): " choice
if [[ "$choice" =~ ^[Nn]$ ]]; then
  echo "  Leaving project at $TMPDIR"
  echo "  cd $TMPDIR"
else
  cd /
  rm -rf "$TMPDIR"
  echo "  ✅ Cleaned up."
fi
echo ""

# ── Done ──
echo "╔══════════════════════════════════════════════════╗"
echo "║  ✅ dectl walkthrough complete!                  ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "📖 Full guide:  https://github.com/jhonesis/dectl/REVIEWER.md"
echo "📧 Contact:     jhonesis@proton.me"
