#!/bin/bash
set -euo pipefail

TMP=$(mktemp -d)
cd "$TMP"

# Init project
dectl project init --standard

# Project info
dectl project info --json | grep -q '"status": "ok"'

# Memory add / list
dectl memory add "test memory" --tags test
dectl memory list --json | grep -q test

# Memory show (by id 1)
dectl memory show 1 --json | grep -q '"status": "ok"'

# Memory search
dectl memory search "test" --json | grep -q '"status": "ok"'

# Workflow list
dectl workflow list --json | grep -q '"status": "ok"'

# Doctor
dectl doctor --json | grep -q '"status": "ok"'

# Agent list
dectl agent list --json | grep -q '"status": "ok"'

# Version
dectl version | grep -q "dectl v"

# Completions
dectl generate-completions bash | grep -q "complete"

# Memory query
dectl memory query "type:note" --json | grep -q '"status": "ok"'

# Cleanup
rm -rf "$TMP"

echo "ALL SMOKE TESTS PASSED"
