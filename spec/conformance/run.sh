#!/usr/bin/env bash
# Shapes Context Protocol — CLI Binding Conformance Runner
#
# Runs all test vectors against a CLI implementation of the Shapes protocol.
# This runner is specific to the CLI binding (spec/bindings/cli.md).
# Other bindings (MCP, HTTP) need their own harness that reads the same
# test vector fixtures.
#
# The test vectors (`.shapes/` directories) are transport-agnostic.
# This script only checks the CLI binding's contract:
#   - valid vectors: `shapes validate` exits 0
#   - invalid vectors: `shapes validate` exits 2
#
# Usage:
#   ./run.sh                     # uses 'shapes' from PATH
#   ./run.sh /path/to/shapes     # uses specific binary
#
# Exit code:
#   0 — all tests passed
#   1 — one or more tests failed

set -euo pipefail

SHAPES="${1:-shapes}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VECTORS="$SCRIPT_DIR/test-vectors"

passed=0
failed=0
errors=()

# ── Valid vectors: validate MUST exit 0 ───────────────────────────────────

for fixture in "$VECTORS"/valid/*/; do
  name="$(basename "$fixture")"
  exit_code=$(cd "$fixture" && "$SHAPES" validate > /dev/null 2>&1; echo $?)

  if [ "$exit_code" -eq 0 ]; then
    printf "  PASS  valid/%s\n" "$name"
    passed=$((passed + 1))
  else
    printf "  FAIL  valid/%s (expected exit 0, got %s)\n" "$name" "$exit_code"
    errors+=("valid/$name")
    failed=$((failed + 1))
  fi
done

# ── Invalid vectors: validate MUST exit 2 ─────────────────────────────────

for fixture in "$VECTORS"/invalid/*/; do
  name="$(basename "$fixture")"

  if [ ! -f "$fixture/expected.json" ]; then
    printf "  SKIP  invalid/%s (no expected.json)\n" "$name"
    continue
  fi

  exit_code=$(cd "$fixture" && "$SHAPES" validate > /dev/null 2>&1; echo $?)

  if [ "$exit_code" -eq 2 ]; then
    printf "  PASS  invalid/%s\n" "$name"
    passed=$((passed + 1))
  else
    printf "  FAIL  invalid/%s (expected exit 2, got %s)\n" "$name" "$exit_code"
    errors+=("invalid/$name")
    failed=$((failed + 1))
  fi
done

# ── Summary ───────────────────────────────────────────────────────────────

echo
printf "%d passed, %d failed\n" "$passed" "$failed"

if [ "$failed" -gt 0 ]; then
  echo
  echo "Failures:"
  for e in "${errors[@]}"; do
    printf "  - %s\n" "$e"
  done
  exit 1
fi
