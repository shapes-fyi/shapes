#!/usr/bin/env bash
# Shapes Specification — CLI Binding Conformance Runner
#
# Runs all test vectors against a CLI implementation of the Shapes spec.
# This runner is specific to the CLI binding (spec/bindings/cli.md).
# Other bindings (MCP, HTTP) need their own harness that reads the same
# test vector fixtures.
#
# Uses `--format json` to get structured output with invariant IDs, then
# verifies that the expected invariants from expected.json appear in the
# validation output.
#
# Usage:
#   ./runners/cli.sh                     # uses 'shapes' from PATH
#   ./runners/cli.sh /path/to/shapes     # uses specific binary
#
# Exit code:
#   0 — all tests passed
#   1 — one or more tests failed

set -euo pipefail

SHAPES="${1:-shapes}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VECTORS="$SCRIPT_DIR/../test-vectors"

passed=0
failed=0
errors=()

# ── Valid vectors: validate MUST exit 0 with empty array ──────────────────

for fixture in "$VECTORS"/valid/*/; do
  name="$(basename "$fixture")"
  exit_code=$(cd "$fixture" && "$SHAPES" validate --format json > /dev/null 2>&1; echo $?)

  if [ "$exit_code" -eq 0 ]; then
    printf "  PASS  valid/%s\n" "$name"
    passed=$((passed + 1))
  else
    printf "  FAIL  valid/%s (expected exit 0, got %s)\n" "$name" "$exit_code"
    errors+=("valid/$name")
    failed=$((failed + 1))
  fi
done

# ── Invalid vectors: validate MUST exit 2 with expected invariants ────────

for fixture in "$VECTORS"/invalid/*/; do
  name="$(basename "$fixture")"

  if [ ! -f "$fixture/expected.json" ]; then
    printf "  SKIP  invalid/%s (no expected.json)\n" "$name"
    continue
  fi

  # Run validate and capture JSON output
  json_output=$(cd "$fixture" && "$SHAPES" validate --format json 2>/dev/null) || true
  exit_code=$(cd "$fixture" && "$SHAPES" validate --format json > /dev/null 2>&1; echo $?)

  if [ "$exit_code" -ne 2 ]; then
    printf "  FAIL  invalid/%s (expected exit 2, got %s)\n" "$name" "$exit_code"
    errors+=("invalid/$name: wrong exit code")
    failed=$((failed + 1))
    continue
  fi

  # Check that each expected invariant appears in the output
  # Extract expected invariants from expected.json
  expected_invs=$(grep -o '"INV-[0-9]*"' "$fixture/expected.json" | tr -d '"')
  all_found=true

  for inv in $expected_invs; do
    if echo "$json_output" | grep -q "\"$inv\""; then
      : # found
    else
      printf "  FAIL  invalid/%s (missing invariant %s in output)\n" "$name" "$inv"
      errors+=("invalid/$name: missing $inv")
      all_found=false
      failed=$((failed + 1))
    fi
  done

  if [ "$all_found" = true ]; then
    printf "  PASS  invalid/%s [%s]\n" "$name" "$expected_invs"
    passed=$((passed + 1))
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
