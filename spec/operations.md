# Shapes Specification: Abstract Operation Catalog

**Version:** 0.1.0
**Status:** Draft

## Introduction

This document defines the abstract operations of the Shapes Specification.
Each operation is specified independently of any transport or binding (CLI,
MCP, HTTP, in-process library). A binding specification maps these abstract
operations to a concrete interface; this document defines only the semantics,
parameters, return types, preconditions, postconditions, and error conditions.

An implementation that exposes a Shapes Specification interface MUST or
SHOULD implement each operation according to the requirement levels specified
below.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

## Requirement Levels

Operations are classified into two tiers:

- **MUST implement (Core Read):** These operations are required for any
  conforming implementation. Without them, an agent cannot discover, read, or
  navigate the shapes graph. An implementation that omits any MUST operation is
  non-conforming.

- **SHOULD implement (Write):** These operations enable graph creation and
  mutation. An implementation that provides read-only access (e.g., a static
  export or a query-only MCP server) MAY omit SHOULD operations while
  remaining useful, but a full-featured implementation SHOULD provide them.

## Common Types

The following types are referenced throughout the operation definitions.

### NodeType

```
enum NodeType {
    "shape"
    "constraint"
    "amendment"
    "profile"
}
```

Determines which node set an operation targets. For DAG traversal operations,
`node_type` implicitly selects the DAG: `"shape"` selects the shape
composition graph, `"constraint"` selects the constraint composition graph.
Amendment and Profile nodes do not form their own DAGs.

### NodeId

```
type NodeId = integer (>= 0) | string (non-empty)
```

An opaque node identifier. Implementations MAY restrict to integers or strings;
the specification permits either. Two IDs are equal if and only if they have the
same type and the same value. IDs are scoped to a node type namespace: a Shape
with `id: 5` and a Constraint with `id: 5` are distinct nodes.

### Status

```
enum Status {
    "proposed"
    "promoted"
    "canonical"
    "rejected"
    "superseded"
    "abandoned"
    "reverted"
}
```

Seven-state lifecycle. `proposed`, `promoted`, and `canonical` are progressive
states. `rejected`, `superseded`, `abandoned`, and `reverted` are terminal
states.

### ValidationIssue

```
type ValidationIssue = {
    severity:  "error" | "warning"
    node_type: string
    node_id:   string
    message:   string
}
```

Returned by the `validate` operation. Each issue identifies the node and
describes the invariant violation.

---

## Core Read Operations (MUST implement)

### discover

**Requirement level:** MUST

**Description:**
Probe whether a Shapes graph exists at the given path and return summary
metadata. This is the entry point for any agent or tool interacting with
shapes: call `discover` first to determine if shapes exist and what the graph
contains.

**Parameters:**

| Name | Type   | Required | Description                                           |
|------|--------|----------|-------------------------------------------------------|
| path | string | no       | Directory to probe. Defaults to current working directory. |

**Return type:**

```
{
    version:     string
    node_counts: {
        shape:      integer
        constraint: integer
        amendment:  integer
        profile:    integer
    }
}
```

**Preconditions:**
None. This operation is specifically designed to be called without any prior
knowledge of whether a shapes graph exists.

**Postconditions:**
None. This is a pure read operation.

**Error conditions:**

| Error    | Condition                                        |
|----------|--------------------------------------------------|
| NotFound | No shapes graph exists at the specified path.    |

**Agent question it answers:**
"Does this project use Shapes? What's in it?"

---

### get

**Requirement level:** MUST

**Description:**
Retrieve the full definition of a single node by type and ID. Returns the
complete node object matching the JSON Schema for that node type, including
all fields: intent, status, constraints, realization, evidence, provenance,
parents, children, metadata, and any type-specific fields.

**Parameters:**

| Name      | Type     | Required | Description                     |
|-----------|----------|----------|---------------------------------|
| node_type | NodeType | yes      | The type of node to retrieve.   |
| id        | NodeId   | yes      | The ID of the node to retrieve. |

**Return type:**
The full node object. The structure depends on `node_type`:
- `"shape"`: Shape object per the Shape JSON Schema
- `"constraint"`: Constraint object per the Constraint JSON Schema
- `"amendment"`: Amendment object per the Amendment JSON Schema
- `"profile"`: Profile object per the Profile JSON Schema

**Preconditions:**
A shapes graph MUST exist (i.e., `discover` would succeed).

**Postconditions:**
None. This is a pure read operation.

**Error conditions:**

| Error    | Condition                                                   |
|----------|-------------------------------------------------------------|
| NotFound | No node of the specified type with the specified ID exists. |

**Agent question it answers:**
"Tell me everything about this shape/constraint/amendment/profile."

---

### list

**Requirement level:** MUST

**Description:**
List nodes with optional filters. Returns a summary entry for each matching
node. Without any filters, returns all nodes across all types.

**Parameters:**

| Name      | Type     | Required | Description                                                 |
|-----------|----------|----------|-------------------------------------------------------------|
| node_type | NodeType | no       | Filter to a specific node type. Omit to list all types.     |
| status    | string   | no       | Filter by status name (e.g., `"proposed"`, `"canonical"`).  |
| kind      | string   | no       | Filter by kind. For shapes: `intent.kind`. For constraints: `kind`. For amendments: `intent.kind`. For profiles: `intent.kind`. |

**Return type:**

```
[
    {
        node_type: string
        id:        NodeId
        name:      string
        status:    string
        kind:      string
    }
]
```

**Preconditions:**
A shapes graph MUST exist.

**Postconditions:**
None. This is a pure read operation.

**Error conditions:**
None beyond transport-level errors. An empty result is not an error.

**Agent question it answers:**
"What shapes/constraints exist? Filter by status or kind."

---

### tree

**Requirement level:** MUST

**Description:**
Render the composition hierarchy as a tree structure. When `node_type` is
`"shape"` (the default), displays the shape DAG with constraint references
shown inline at each node. When `node_type` is `"constraint"`, displays the
constraint composition hierarchy.

**Parameters:**

| Name      | Type     | Required | Description                                                     |
|-----------|----------|----------|-----------------------------------------------------------------|
| node_type | NodeType | no       | Which DAG to render. MUST be `"shape"` or `"constraint"`. Defaults to `"shape"`. |
| root      | NodeId   | no       | Show only the subtree rooted at this node. Omit to show all root nodes (nodes with no parents). |
| depth     | integer  | no       | Maximum depth to render. Defaults to 10.                        |

**Return type:**
A hierarchical tree structure. Each tree node contains:

```
type TreeNode = {
    node_type:   string
    id:          NodeId
    name:        string
    status:      string
    kind:        string
    constraints: [NodeId]      // shape trees only; constraint IDs referenced by this shape
    children:    [TreeNode]
}
```

Implementations MAY return this as structured data or render it as formatted
text, depending on the binding.

**Preconditions:**
A shapes graph MUST exist.

**Postconditions:**
None. This is a pure read operation.

**Error conditions:**

| Error    | Condition                                               |
|----------|---------------------------------------------------------|
| NotFound | The specified root node does not exist.                 |

**Agent question it answers:**
"Show me the project structure."

---

### query.ancestors

**Requirement level:** MUST

**Description:**
Walk up the parent chain of a node and return all ancestor IDs. Traversal uses
BFS: starting from the given node's `parents[]`, visit each parent, then each
parent's parents, and so on until all reachable ancestors are collected.

The given node's own ID is NOT included in the result.

**Parameters:**

| Name      | Type     | Required | Description                                                     |
|-----------|----------|----------|-----------------------------------------------------------------|
| node_type | NodeType | yes      | MUST be `"shape"` or `"constraint"`. Determines which DAG to traverse. |
| id        | NodeId   | yes      | The starting node ID.                                           |

**Return type:**

```
[NodeId]
```

All ancestor IDs, in BFS order.

**Preconditions:**
- A shapes graph MUST exist.
- A node of the specified type with the specified ID MUST exist.

**Postconditions:**
None. This is a pure read operation.

**Error conditions:**

| Error    | Condition                                     |
|----------|-----------------------------------------------|
| NotFound | The specified node does not exist.             |

**Agent question it answers:**
"What is this shape/constraint part of?"

---

### query.descendants

**Requirement level:** MUST

**Description:**
Walk down the child tree of a node and return all descendant IDs. Traversal
uses BFS: starting from the given node's `children[]`, visit each child, then
each child's children, and so on until all reachable descendants are collected.

The given node's own ID is NOT included in the result.

**Parameters:**

| Name      | Type     | Required | Description                                                     |
|-----------|----------|----------|-----------------------------------------------------------------|
| node_type | NodeType | yes      | MUST be `"shape"` or `"constraint"`. Determines which DAG to traverse. |
| id        | NodeId   | yes      | The starting node ID.                                           |

**Return type:**

```
[NodeId]
```

All descendant IDs, in BFS order.

**Preconditions:**
- A shapes graph MUST exist.
- A node of the specified type with the specified ID MUST exist.

**Postconditions:**
None. This is a pure read operation.

**Error conditions:**

| Error    | Condition                                     |
|----------|-----------------------------------------------|
| NotFound | The specified node does not exist.             |

**Agent question it answers:**
"What does this shape/constraint decompose into?"

---

### query.constraints

**Requirement level:** MUST

**Description:**
Compute the full set of effective constraints for a shape, including those
inherited from ancestor shapes. Walks up the shape's parent chain via BFS,
collecting all constraint references from each shape visited. Each constraint
is annotated with whether it was directly attached to the queried shape or
inherited from an ancestor.

Constraints are deduplicated by constraint ID. If the same constraint is
referenced by both the queried shape and an ancestor, the direct reference
takes precedence (`inherited: false`).

**Parameters:**

| Name     | Type   | Required | Description                              |
|----------|--------|----------|------------------------------------------|
| shape_id | NodeId | yes      | The shape ID to query constraints for.   |

**Return type:**

```
[
    {
        constraint_id:   NodeId
        constraint_name: string
        source_shape_id: NodeId
        inherited:       boolean
    }
]
```

- `constraint_id`: The ID of the constraint node.
- `constraint_name`: The name of the constraint node. If the constraint cannot
  be loaded (dangling reference), implementations SHOULD return a placeholder.
- `source_shape_id`: The ID of the shape that references this constraint. For
  directly attached constraints, this equals `shape_id`. For inherited
  constraints, this is the ancestor shape ID.
- `inherited`: `false` if the constraint is directly referenced by the queried
  shape, `true` if it comes from an ancestor.

**Preconditions:**
- A shapes graph MUST exist.
- A Shape node with the specified `shape_id` MUST exist.

**Postconditions:**
None. This is a pure read operation.

**Error conditions:**

| Error    | Condition                                |
|----------|------------------------------------------|
| NotFound | The specified shape does not exist.       |

**Agent question it answers:**
"What rules apply to this shape?"

---

### validate

**Requirement level:** MUST

**Description:**
Check the entire graph against all defined invariants (INV-001 through
INV-011, as specified in `invariants.md`). Returns a list of all violations
found. An empty list means the graph is valid.

Implementations MUST check all invariants in a single pass. The operation MUST
NOT stop at the first violation; it MUST report all detectable violations so
that a user or agent can address them in batch.

**Parameters:**
None.

**Return type:**

```
[ValidationIssue]
```

Where each `ValidationIssue` is:

```
{
    severity:  "error" | "warning"
    node_type: string
    node_id:   string
    message:   string
}
```

An empty array indicates the graph is valid.

**Preconditions:**
A shapes graph MUST exist.

**Postconditions:**
None. This is a pure read operation. Validation does not modify the graph.

**Error conditions:**
None beyond transport-level errors. Invariant violations are returned as data,
not as errors.

**Agent question it answers:**
"Is this graph consistent?"

---

## Write Operations (SHOULD implement)

### init

**Requirement level:** SHOULD

**Description:**
Initialize a new, empty Shapes graph. Creates the storage structure (in the
file-based binding: a `.shapes/` directory with `meta.yaml` and subdirectories
for each node type). After `init`, the graph contains zero nodes and is valid.

**Parameters:**

| Name | Type   | Required | Description                                                    |
|------|--------|----------|----------------------------------------------------------------|
| path | string | no       | Directory in which to initialize. Defaults to current working directory. |

**Return type:**

```
{
    version: string
    path:    string
}
```

- `version`: The Shapes Specification version of the newly created graph.
- `path`: The absolute or relative path to the created graph root.

**Preconditions:**
None.

**Postconditions:**
- A shapes graph exists at the specified path.
- The graph is empty (zero nodes of every type).
- The graph is valid (all invariants trivially satisfied).

**Error conditions:**

| Error         | Condition                                                |
|---------------|----------------------------------------------------------|
| AlreadyExists | A shapes graph already exists at the specified path.     |

**Agent question it answers:**
"Set up Shapes for this project."

---

### create

**Requirement level:** SHOULD

**Description:**
Create a new node and add it to the graph. The implementation assigns the next
available ID for the given node type. The node starts in `"proposed"` status
unless the provided data specifies otherwise.

After creation, the implementation MUST verify that the graph remains valid.
At minimum, if the new node includes `parents` or `children` references, the
implementation MUST check for cycles (INV-001, INV-002) and dangling
references (INV-004, INV-005). If validation fails, the creation MUST be
rejected and the graph MUST remain unchanged.

**Parameters:**

| Name      | Type     | Required | Description                                            |
|-----------|----------|----------|--------------------------------------------------------|
| node_type | NodeType | yes      | The type of node to create.                            |
| data      | object   | yes      | The node definition. The `id` field, if present, is ignored; the implementation assigns the ID. All other fields follow the JSON Schema for the specified node type. |

**Return type:**
The full node object as stored, including the assigned ID.

**Preconditions:**
- A shapes graph MUST exist.

**Postconditions:**
- A new node with a unique, auto-assigned ID exists in the graph.
- The node's status is `"proposed"` unless explicitly overridden in `data`.
- All graph invariants remain satisfied.

**Error conditions:**

| Error            | Condition                                                           |
|------------------|---------------------------------------------------------------------|
| ValidationError  | The new node would violate one or more invariants (e.g., creating a cycle, referencing non-existent parents). |
| SchemaError      | The provided `data` does not conform to the JSON Schema for the specified node type. |

**Agent question it answers:**
"Create a new shape/constraint/amendment/profile."
