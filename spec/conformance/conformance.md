# Shapes Context Protocol: Conformance Test Vectors

**Version:** 0.1.0
**Status:** Draft

## Introduction

This document defines the conformance requirements for implementations of the
Shapes Context Protocol. A conforming implementation MUST pass all valid test
vectors and MUST reject all invalid test vectors, reporting the expected
invariant violations.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

## What Conformance Means

An implementation is conforming if and only if:

1. **Valid vectors pass:** For every test vector under `test-vectors/valid/`,
   the implementation MUST load the `.shapes/` directory without error and
   MUST report zero invariant violations when running the `validate` operation.

2. **Invalid vectors fail with the correct invariant:** For every test vector
   under `test-vectors/invalid/`, the implementation MUST detect at least the
   invariants listed in the fixture's `expected.json` file. The implementation
   MAY report additional invariants beyond those listed (e.g., a cycle may also
   trigger a reciprocity violation), but MUST report all expected ones.

An implementation that passes all valid vectors but misses an expected
invariant on any invalid vector is non-conforming.

## Test Vectors vs. Test Runners

The test vectors (`.shapes/` directories) are **transport-agnostic**. They are
plain data that any implementation can load regardless of binding (CLI, MCP,
HTTP, library). The vectors are the protocol-level conformance artifact.

Test runners are **binding-specific**. Each transport binding needs its own
harness that loads the test vectors and exercises the implementation through
its native interface. The vectors are shared; the runners are not.

| Binding | How the runner invokes `validate` | How it checks the result |
|---------|----------------------------------|--------------------------|
| CLI     | `shapes validate` (cwd = fixture) | Exit code: 0 = pass, 2 = fail |
| MCP     | Call `shapes_validate` tool | Response array: empty = pass, non-empty = fail |
| HTTP    | `POST /validate` | Status 200 + empty array = pass, non-empty = fail |
| Library | `validate(graph)` function call | Return value: empty = pass, non-empty = fail |

A reference CLI runner is provided at `run.sh`.

## How to Run Conformance Tests

For each test vector directory:

1. Point the implementation at the `.shapes/` directory within the fixture.
2. Load all node files (shapes, constraints, amendments, profiles) and
   `meta.yaml`.
3. Run the full `validate` operation.
4. For **valid** vectors: assert zero validation issues.
5. For **invalid** vectors: assert at least one validation issue is reported.
   The `expected.json` at the fixture root documents which invariant(s) the
   fixture is designed to test — use it to verify that the *correct* invariant
   was detected, not just that *some* error occurred.

### Example (pseudocode)

```
for fixture in test-vectors/valid/*:
    graph = load(fixture/.shapes/)
    issues = validate(graph)
    assert issues == []

for fixture in test-vectors/invalid/*:
    graph = load(fixture/.shapes/)
    issues = validate(graph)
    assert len(issues) > 0
    # Optionally verify specific invariants:
    expected = json.parse(fixture/expected.json)
    for inv in expected.invariants:
        assert inv in issues.invariant_ids
```

### CLI Binding Runner

```bash
./run.sh                     # uses 'shapes' from PATH
./run.sh /path/to/shapes     # uses specific binary
```

The CLI runner checks only exit codes (0 vs. 2) since that is the CLI
binding's contract for the `validate` operation. It does not parse output
text.

## Test Vector Structure

### Valid Vectors

```
test-vectors/valid/<name>/
  .shapes/
    meta.yaml
    shapes/
      1.yaml
      ...
    constraints/
      1.yaml
      ...
    amendments/
      ...
    profiles/
      ...
```

Valid vectors contain only a `.shapes/` directory. No `expected.json` is
needed because the expected result is zero violations.

### Invalid Vectors

```
test-vectors/invalid/<name>/
  expected.json
  .shapes/
    meta.yaml
    shapes/
      ...
    constraints/
      ...
    amendments/
      ...
    profiles/
      ...
```

The `expected.json` file at the fixture root has the following structure:

```json
{
  "invariants": ["INV-001"],
  "description": "Human-readable explanation of what is wrong"
}
```

- `invariants` — Array of invariant IDs (from `spec/invariants.md`) that
  a conforming implementation MUST report for this fixture.
- `description` — Prose explanation of the violation for human readers.
  Implementations do not need to match this string.

## Test Vector Inventory

### Valid

| Name        | What it tests                                                        |
|-------------|----------------------------------------------------------------------|
| `minimal`   | Single shape, no constraints, no profile. Smallest valid graph.      |
| `hierarchy` | 3-level shape DAG with constraint inheritance and reciprocal links.  |
| `full`      | All node types: profile, shapes, constraint, amendment, evidence, provenance, realization. |

### Invalid

| Name                       | Expected Invariant | What it tests                                             |
|----------------------------|-------------------|-----------------------------------------------------------|
| `cycle-shape`              | INV-001           | Two shapes forming a parent-child cycle.                  |
| `cycle-constraint`         | INV-002           | Two constraints forming a parent-child cycle.             |
| `dangling-constraint-ref`  | INV-003           | Shape references a constraint ID that does not exist.     |
| `dangling-parent`          | INV-004           | Shape references a parent ID that does not exist.         |
| `dangling-child`           | INV-005           | Shape references a child ID that does not exist.          |
| `dangling-profile-ref`     | INV-006           | Shape references a profile ID that does not exist.        |
| `empty-amendment-targets`  | INV-007           | Amendment with empty targets object.                      |
| `dangling-amendment-target`| INV-008           | Amendment targets a shape ID that does not exist.         |
| `non-reciprocal-link`      | INV-009           | Shape lists child that does not list it as parent.        |
| `missing-profile-fields`   | INV-010           | Shape governed by profile is missing a required intent field. |
| `duplicate-id`             | INV-011           | Two shape files contain the same id field value.          |

## Invariant Reference

See [`spec/invariants.md`](../invariants.md) for the full definition of each
invariant ID referenced by the test vectors.
