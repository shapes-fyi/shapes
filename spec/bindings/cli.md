# Shapes Specification: CLI Transport Binding

**Version:** 0.1.0
**Status:** Draft

## Introduction

This document maps the abstract operations defined in
[operations.md](../operations.md) to the `shapes` command-line tool. The CLI
binding is the reference implementation of the Shapes Specification.

For each operation, this document specifies the command syntax, input format,
output format, exit codes, and stream behavior. Implementations of the CLI
binding MUST conform to the interface described here.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

---

## Conventions

### Output Streams

- **stdout** -- Data output. All node content, lists, trees, and validation
  results are written to stdout. This stream is machine-parseable when used
  with `--format json`.
- **stderr** -- Diagnostic output. Error messages, warnings, and progress
  information are written to stderr. Diagnostic output is not
  machine-parseable and MUST NOT be relied upon for programmatic
  consumption.

### Output Format

The default output format is YAML. The `--format` global flag controls
serialization:

- `--format yaml` (default) -- YAML output.
- `--format json` -- JSON output.

Implementations MUST support both formats for all operations that produce
data output.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0    | Success. The operation completed without errors. |
| 1    | Error. The operation failed (missing node, invalid input, I/O error, etc.). |
| 2    | Validation failures. Used exclusively by `shapes validate` when invariant violations are found. |

### Discovery

The CLI binding uses filesystem discovery as defined in
[discovery.md](../discovery.md), Mechanism 1. The `shapes` tool locates the
`.shapes/` directory by walking up from the current working directory. If no
`.shapes/` directory is found, commands that require a graph MUST exit with
code 1 and print a diagnostic to stderr.

The `shapes list` command with no arguments serves as the implicit discovery
operation: if it succeeds, a shapes graph exists and is readable.

---

## Global Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--format` | `yaml \| json` | `yaml` | Output serialization format. |

Global flags MUST appear before the subcommand or after all subcommand
arguments. Implementations SHOULD accept both positions.

---

## Operations

### discover

**Abstract operation:** `discover`

**Command:**

```
shapes list
```

Discovery is implicit in the CLI binding. The presence of a `.shapes/`
directory is the discovery signal. Running `shapes list` with no arguments
confirms the graph exists and returns all nodes. See
[discovery.md](../discovery.md) for the filesystem discovery algorithm.

**Exit codes:** 0 on success, 1 if no graph is found.

---

### get

**Abstract operation:** `get`

**Command:**

```
shapes get <node_type> <id> [--archived]
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `node_type` | positional | yes | One of: `shape`, `constraint`, `amendment`, `profile`. |
| `id` | positional | yes | The node ID (integer or string). |

**Flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--archived` | boolean | Include archived amendments in the rendered `amendment_log`. By default they are filtered out. When set, each archived entry is annotated (rendered as `{id, archived: true, archived_reason: "..."}` instead of a bare ID) so readers can distinguish archived from unarchived entries and see why they were archived. Has no effect when `node_type` is `amendment` — direct amendment fetch always returns the full record. |

**Output (stdout):** The full node object serialized in the selected format.
All fields defined in the node's JSON Schema are included, with
`amendment_log` filtered or annotated per the `--archived` flag.

**Exit codes:** 0 on success, 1 if the node does not exist.

**Example:**

```
$ shapes get shape 5
id: 5
name: Specification
description: >-
  Formal specification of the Shapes Specification...
status: promoted
intent:
  kind: documentation
  summary: Spec defining schemas, operations, invariants, and bindings
  source: human
...
```

---

### list

**Abstract operation:** `list`

**Command:**

```
shapes list [node_type] [--status <status>] [--kind <kind>] [--archived]
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `node_type` | positional | no | Filter to a single type: `shape`, `constraint`, `amendment`, `profile`. Omit to list all types. |

**Flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--status` | string | Filter by status name (e.g., `proposed`, `canonical`). |
| `--kind` | string | Filter by kind (matched against `intent.kind` for shapes, amendments, and profiles; `kind` for constraints). |
| `--archived` | boolean | Include archived amendments in the listing. By default, amendments with an `archived` field are hidden so decayed audit entries do not clutter agent context. Non-amendment node types are unaffected. |

**Output (stdout):** A list of summary entries. Each entry contains:

```yaml
- node_type: shape
  id: 5
  name: Specification
  status: promoted
  kind: documentation
```

When no nodes match the filters, the output is an empty list. This is not
an error.

**Exit codes:** 0 on success, 1 on error.

**Examples:**

```
$ shapes list shape --status promoted
$ shapes list constraint --kind testing
$ shapes list --kind system
```

---

### tree

**Abstract operation:** `tree`

**Command:**

```
shapes tree [node_type] [--root <id>] [--depth <n>]
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `node_type` | positional | no | Which DAG to render: `shape` (default) or `constraint`. |

**Flags:**

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--root` | NodeId | (all roots) | Show only the subtree rooted at this node. |
| `--depth` | integer | 10 | Maximum depth to render. |

**Output (stdout):** A textual tree rendering. Constraint references are
shown inline for shape trees. The output is human-readable formatted text,
not structured YAML/JSON (the `--format` flag does not apply to tree
output).

**Exit codes:** 0 on success, 1 if the specified root node does not exist.

**Example:**

```
$ shapes tree shape --root 1 --depth 2
[1] Shapes CLI (system, promoted) [C:6,7,8,9,11]
  [2] Core Graph Engine (component, promoted) [C:6,7]
  [3] Specification Website (component, promoted) [C:6]
  [4] Command Layer (component, promoted) [C:6]
  [5] Specification (documentation, promoted) [C:6]
  ...
```

---

### query.ancestors

**Abstract operation:** `query.ancestors`

**Command:**

```
shapes query ancestors <node_type> <id>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `node_type` | positional | yes | `shape` or `constraint`. Determines which DAG to traverse. |
| `id` | positional | yes | The starting node ID. |

**Output (stdout):** A list of ancestor NodeIds in BFS order, serialized in
the selected format. The starting node's own ID is not included.

```yaml
- 3
- 1
```

**Exit codes:** 0 on success, 1 if the node does not exist.

---

### query.descendants

**Abstract operation:** `query.descendants`

**Command:**

```
shapes query descendants <node_type> <id>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `node_type` | positional | yes | `shape` or `constraint`. Determines which DAG to traverse. |
| `id` | positional | yes | The starting node ID. |

**Output (stdout):** A list of descendant NodeIds in BFS order, serialized
in the selected format. The starting node's own ID is not included.

```yaml
- 12
- 13
- 14
```

**Exit codes:** 0 on success, 1 if the node does not exist.

---

### query.constraints

**Abstract operation:** `query.constraints`

**Command:**

```
shapes query constraints <shape_id>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `shape_id` | positional | yes | The shape ID to query constraints for. |

**Output (stdout):** A list of constraint entries with inheritance
information, serialized in the selected format.

```yaml
- constraint_id: 6
  constraint_name: YAML File Format
  source_shape_id: 1
  inherited: true
- constraint_id: 7
  constraint_name: DAG Integrity
  source_shape_id: 2
  inherited: false
```

**Exit codes:** 0 on success, 1 if the shape does not exist.

---

### validate

**Abstract operation:** `validate`

**Command:**

```
shapes validate
```

**Arguments:** None.

**Output (stdout):** A list of validation issues, serialized in the selected
format. Each issue contains `severity`, `node_type`, `node_id`, and
`message`. An empty list (no output) means the graph is valid.

```yaml
- severity: error
  node_type: shape
  node_id: "12"
  message: "Parent reference 99 does not resolve to an existing shape"
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0    | Graph is valid. No invariant violations found. |
| 2    | Validation failures. One or more invariant violations detected. Issues are printed to stdout. |
| 1    | Error. The validate operation itself failed (e.g., graph not found, I/O error). |

The validate command MUST check all invariants (INV-001 through INV-011 as
defined in [invariants.md](../invariants.md)) in a single pass. It MUST NOT
stop at the first violation.

---

### init

**Abstract operation:** `init`

**Command:**

```
shapes init
```

**Output (stdout):** Confirmation of the created graph, serialized in the
selected format.

```yaml
version: 0.1.0
path: .shapes
```

**Behavior:** Creates a `.shapes/` directory in the current working
directory containing `meta.yaml` and empty subdirectories (`shapes/`,
`constraints/`, `amendments/`, `profiles/`).

**Exit codes:** 0 on success, 1 if a graph already exists at the target
location.

---

### create

**Abstract operation:** `create`

**Command:**

```
shapes create <node_type> [flags]
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `node_type` | positional | yes | One of: `shape`, `constraint`, `amendment`, `profile`. |

**Common Flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--from` | string | Path to a YAML file containing the full node definition. Use `-` to read from stdin. When provided, all other field flags are ignored. |
| `--id-only` | boolean | If set, output only the assigned ID (as a bare integer) instead of the full node object. Intended for scripting. |

**Shape Flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--name` | string | Shape name (required unless `--from` is used). |
| `--kind` | string | Intent kind (e.g., `system`, `component`, `feature`). |
| `--summary` | string | Intent summary. |
| `--source` | string | Intent source (e.g., `human`, `agent`). |

**Constraint Flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--name` | string | Constraint name (required unless `--from` is used). |
| `--kind` | string | Constraint kind (e.g., `testing`, `architecture`, `style`). |
| `--rule` | string | The concrete rule statement. |
| `--enforcement` | string | How the constraint is enforced (e.g., `ci`, `manual-review`). |

**Amendment and Profile Flags:**

For amendment and profile creation, use the `--from` flag to provide a full
YAML definition. These node types have structures that are not well-suited
to individual flag decomposition.

**Output (stdout):** The full created node object, serialized in the
selected format. If `--id-only` is set, output is a bare integer followed
by a newline.

**Exit codes:** 0 on success, 1 on error (schema validation failure, graph
invariant violation, I/O error).

**Examples:**

```
$ shapes create shape --name "Auth Module" --kind component --summary "Handles authentication"
$ shapes create constraint --name "No Panics" --kind safety --rule "No unwrap() in production code" --enforcement ci
$ shapes create shape --from shape-definition.yaml
$ cat shape.yaml | shapes create shape --from -
$ shapes create shape --name "New Feature" --id-only
23
```

---

### amendment archive / amendment unarchive

**Abstract operation:** none (CLI-specific amendment maintenance)

**Command:**

```
shapes amendment archive <id> --reason <reason>
shapes amendment unarchive <id>
```

**Arguments:**

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | positional | yes | The numeric amendment ID to archive or unarchive. |

**Flags (archive only):**

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--reason` | string | yes | Explanation of why this amendment is being archived. |

**Description:**
Sets or clears the display-only `archived` field on an amendment.
Archiving hides an amendment from default `shapes list` output and
from the rendered `amendment_log` in `shapes get <parent>` unless
`--archived` is passed. It is not a delete — the amendment YAML stays
on disk, reciprocity still applies, and CI-002 continues to count the
amendment. A reason is required so future readers understand why the
entry was archived.

This is the sole permitted mutation of a canonical amendment. CI-003
(modified-amendment-immutability) explicitly allows diffs whose only
field delta is `archived`; every other field remains immutable.

**Output (stdout):** The full amendment object after the change, so
the caller can confirm the new state.

**Exit codes:** 0 on success, 1 if the amendment does not exist.

**Examples:**

```
$ shapes amendment archive 17 --reason "Changes integrated into target shapes"
$ shapes amendment unarchive 17
```

---

## Operation Summary

| Abstract Operation | CLI Command | Notes |
|---|---|---|
| discover | `shapes list` | Implicit. Presence of `.shapes/` is the discovery signal. |
| get | `shapes get <node_type> <id> [--archived]` | node_type: shape, constraint, amendment, profile. `--archived` surfaces archived amendments in `amendment_log`. |
| list | `shapes list [node_type] [--status X] [--kind Y] [--archived]` | No args = all types. `--archived` includes archived amendments. |
| tree | `shapes tree [node_type] [--root X] [--depth N]` | Defaults to shape. Constraints shown inline. |
| query.ancestors | `shapes query ancestors <node_type> <id>` | node_type: shape or constraint |
| query.descendants | `shapes query descendants <node_type> <id>` | node_type: shape or constraint |
| query.constraints | `shapes query constraints <shape_id>` | Returns constraints with inheritance info |
| validate | `shapes validate` | Exit 0 if clean, exit 2 if issues |
| init | `shapes init` | Creates `.shapes/` directory |
| create | `shapes create <node_type> [flags]` | `--from`, `--id-only`, per-type field flags |
| (cli-only) | `shapes amendment archive \| unarchive <id>` | Toggles display-only archived flag. Sole permitted mutation under CI-003. |
