# Shapes Specification: Graph Invariants

**Version:** 0.1.0
**Status:** Draft

## Introduction

This document defines the structural invariants that a conforming Shapes
Specification graph MUST satisfy. Invariants are properties that hold at
all times after any completed operation. An implementation MUST enforce every
invariant listed here; a graph that violates any invariant is in an invalid
state and MUST be reported by the `validate` operation.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

## Invariant Summary

| ID      | Level | Severity | Rule                                                            |
|---------|-------|----------|-----------------------------------------------------------------|
| INV-001 | MUST  | error    | Shape composition graph MUST be acyclic                         |
| INV-002 | MUST  | error    | Constraint composition graph MUST be acyclic                    |
| INV-003 | MUST  | error    | Shape constraint references MUST resolve to existing Constraints|
| INV-004 | MUST  | error    | Parent references MUST resolve to existing nodes of same type   |
| INV-005 | MUST  | error    | Child references MUST resolve to existing nodes of same type    |
| INV-006 | MUST  | error    | Profile references MUST resolve to existing Profiles            |
| INV-007 | MUST  | error    | Every Amendment MUST target at least one node                   |
| INV-008 | MUST  | error    | Amendment target references MUST resolve to existing nodes      |
| INV-009 | MUST  | error    | Parent-child links MUST be reciprocal                           |
| INV-010 | MUST  | error    | Nodes governed by a Profile MUST satisfy required field declarations |
| INV-011 | MUST  | error    | IDs MUST be unique within their node type namespace             |
| INV-019 | MUST  | error    | Amendment targets and node `amendment_log` entries MUST be reciprocal |

## Invariant Definitions

### INV-001: Shape DAG Acyclicity

**Level:** MUST
**Severity:** error
**Applies to:** Shape nodes

**Formal statement:**
The directed graph formed by following `children[].shape` references (by ID or
inline) from every Shape node MUST be a directed acyclic graph (DAG). No
sequence of child-edge traversals starting from any Shape node may revisit
that node.

**Rationale:**
A cycle in the shape composition graph causes infinite traversal in ancestor
and descendant queries. Any BFS or DFS walk that encounters a cycle will never
terminate without explicit cycle detection. Since the composition hierarchy is
the primary navigation structure for agents discovering project intent, cycles
render the graph unusable.

**Detection method:**
Three-color DFS (white/gray/black). For each unvisited (white) node, begin
DFS. Mark nodes gray on entry, black on exit. If a child edge leads to a gray
node, a cycle exists. The edge from the current node to the gray node MUST be
reported.

```
for each node in shapes:
    if node.color == white:
        dfs(node)

dfs(node):
    node.color = gray
    for child_id in node.children[].shape:
        if child.color == gray:
            report cycle: node.id -> child_id
        if child.color == white:
            dfs(child)
    node.color = black
```

---

### INV-002: Constraint DAG Acyclicity

**Level:** MUST
**Severity:** error
**Applies to:** Constraint nodes

**Formal statement:**
The directed graph formed by following `children[].constraint` references (by
ID or inline) from every Constraint node MUST be a directed acyclic graph
(DAG). No sequence of child-edge traversals starting from any Constraint node
may revisit that node.

**Rationale:**
Same as INV-001. The constraint composition hierarchy supports inheritance of
rules through parent-child relationships. A cycle prevents determination of
the full constraint set for any node within the cycle.

**Detection method:**
Three-color DFS, identical to INV-001 but applied to the Constraint node set
and their `children[].constraint` edges.

---

### INV-003: Shape Constraint Reference Integrity

**Level:** MUST
**Severity:** error
**Applies to:** Shape nodes

**Formal statement:**
For every Shape node S, every element in `S.constraints[]` MUST be the ID of
an existing Constraint node in the graph.

**Rationale:**
A shape's `constraints` array is the mechanism by which rules are bound to
work items. The `query.constraints` operation resolves these references to
return the full set of applicable rules. If a referenced constraint does not
exist, the operation cannot determine what rules apply, and an agent will
either receive incomplete information or encounter an error.

**Detection method:**
For each shape, iterate `constraints[]`. For each ID, verify a Constraint node
with that ID exists. Report each unresolvable reference.

---

### INV-004: Parent Reference Integrity

**Level:** MUST
**Severity:** error
**Applies to:** Shape nodes, Constraint nodes

**Formal statement:**
For every node N with a `parents[]` array:
- If N is a Shape, every `parents[].id` MUST be the ID of an existing Shape node.
- If N is a Constraint, every `parents[].id` MUST be the ID of an existing Constraint node.

A parent reference MUST NOT cross node types. A Shape MUST NOT reference a
Constraint as a parent, and vice versa.

**Rationale:**
Parent references are the upward edges in the composition DAG. The
`query.ancestors` operation follows these edges via BFS. A dangling parent
reference (pointing to a non-existent node) silently truncates the ancestor
chain, causing `query.ancestors` and `query.constraints` (which walks up shape
ancestors to collect inherited constraints) to return incomplete results.

**Detection method:**
For each shape, iterate `parents[]` and verify each `id` exists in the shape
node set. For each constraint, iterate `parents[]` and verify each `id` exists
in the constraint node set. Report each unresolvable reference with the source
node and the missing target ID.

---

### INV-005: Child Reference Integrity

**Level:** MUST
**Severity:** error
**Applies to:** Shape nodes, Constraint nodes

**Formal statement:**
For every node N with a `children[]` array:
- If N is a Shape, every `children[].shape` that is an ID reference (not an
  inline definition) MUST resolve to an existing Shape node.
- If N is a Constraint, every `children[].constraint` that is an ID reference
  (not an inline definition) MUST resolve to an existing Constraint node.

A child reference MUST NOT cross node types.

**Rationale:**
Child references are the downward edges in the composition DAG. The
`query.descendants` operation and `tree` rendering follow these edges. A
dangling child reference causes the tree to display missing nodes and the
descendant query to either error or silently omit subtrees.

**Detection method:**
For each shape, resolve each `children[].shape`. If it is a numeric ID (not an
inline Shape object), verify the ID exists in the shape node set. Apply the
same logic for constraints and their `children[].constraint` references.

---

### INV-006: Profile Reference Integrity

**Level:** MUST
**Severity:** error
**Applies to:** Shape nodes, Constraint nodes

**Formal statement:**
For every node N that has a `profile` field set to a value P, a Profile node
with ID equal to P MUST exist in the graph.

**Rationale:**
The `profile` field binds a node to its governance configuration. Profiles
define required fields (INV-010), lifecycle gates, and amendment rules. If the
referenced profile does not exist, field validation cannot run, lifecycle gates
cannot be enforced, and the governance model for that node is undefined.

**Detection method:**
For each shape and constraint with a non-null `profile` value, verify a
Profile node with that ID exists. Report each unresolvable reference.

---

### INV-007: Amendment Non-Empty Targets

**Level:** MUST
**Severity:** error
**Applies to:** Amendment nodes

**Formal statement:**
Every Amendment node MUST have at least one target. The `targets` object MUST
contain at least one non-empty array among `shape_ids`, `constraint_ids`, and
`profile_ids`.

An amendment where all three target arrays are either absent or empty is
invalid.

**Rationale:**
An amendment is a change record. Its purpose is to document a modification to
one or more existing nodes. An amendment with no targets has no effect, carries
no meaning, and cannot be applied. Allowing empty-target amendments would
pollute the amendment log without recording any actual change.

**Detection method:**
For each amendment, check whether `targets.shape_ids`,
`targets.constraint_ids`, and `targets.profile_ids` are all either absent or
empty. If so, report the amendment as invalid.

```
is_empty(targets) =
    (targets.shape_ids is absent OR targets.shape_ids == [])
    AND (targets.constraint_ids is absent OR targets.constraint_ids == [])
    AND (targets.profile_ids is absent OR targets.profile_ids == [])
```

---

### INV-008: Amendment Target Reference Integrity

**Level:** MUST
**Severity:** error
**Applies to:** Amendment nodes

**Formal statement:**
For every Amendment node A:
- Every ID in `A.targets.shape_ids` MUST be the ID of an existing Shape node.
- Every ID in `A.targets.constraint_ids` MUST be the ID of an existing Constraint node.
- Every ID in `A.targets.profile_ids` MUST be the ID of an existing Profile node.

**Rationale:**
An amendment targeting a non-existent node cannot be applied. The target
reference is the link between the change record and the thing it changes. A
dangling target means the amendment describes a change to something that does
not exist, which is logically inconsistent.

**Detection method:**
For each amendment, iterate each target ID array and verify the referenced node
exists in the corresponding node set. Report each unresolvable reference with
the amendment ID and the missing target.

---

### INV-009: Parent-Child Link Reciprocity

**Level:** MUST
**Severity:** error
**Applies to:** Shape nodes, Constraint nodes

**Formal statement:**
For every node A that lists node B as a child (via `children[].shape` or
`children[].constraint` by ID), node B MUST list node A in its `parents[]`
array.

Conversely, for every node B that lists node A as a parent (via
`parents[].id`), node A MUST list node B in its `children[]` array.

**Rationale:**
The composition DAG is navigable in both directions: upward via `parents[]`
(used by `query.ancestors` and `query.constraints`) and downward via
`children[]` (used by `query.descendants` and `tree`). If these links are not
reciprocal, the two traversal directions give inconsistent results. An agent
walking up from a child would not reach a parent that claims to own it, or
vice versa. This breaks the fundamental assumption that the graph is a single
consistent structure.

**Detection method:**
For each node A with children, for each child ID B:
1. Load node B.
2. Check that B's `parents[]` contains A's ID.
3. If not, report: "A lists B as child, but B does not list A as parent."

The reverse check (parent claims child) follows the same pattern.

---

### INV-010: Profile Required Field Satisfaction

**Level:** MUST
**Severity:** error
**Applies to:** Shape nodes, Constraint nodes

**Formal statement:**
For every node N that is governed by a Profile P (i.e., `N.profile == P.id`):
- For each field declaration in P's `fields` section that applies to N's node
  type and is marked `required: true`, the corresponding field MUST be present
  and non-null on node N.

Specifically, if Profile P declares a required field named F in section S
(e.g., `fields.shape.intent.fields[name=F, required=true]`), then every Shape
governed by P MUST have the key F present in the corresponding section of its
data (e.g., in `intent`'s extra fields).

**Rationale:**
Profiles are governance contracts. When a profile declares a field as required,
it expresses an organizational policy that certain information must be present
for a node to be considered well-formed. Allowing nodes to omit required fields
defeats the purpose of the governance layer and permits incomplete
specifications to pass validation.

**Detection method:**
For each node with a `profile` reference:
1. Load the referenced Profile.
2. Look up the `fields` section for the node's type (shape or constraint).
3. For each field group (intent, metadata, etc.), iterate field definitions.
4. For each field marked `required: true`, verify the key exists in the
   corresponding section of the node.
5. Report each missing required field.

---

### INV-011: ID Uniqueness Within Node Type

**Level:** MUST
**Severity:** error
**Applies to:** All node types (Shape, Constraint, Amendment, Profile)

**Formal statement:**
Within each node type namespace, every node MUST have a unique ID. No two
Shape nodes may share the same ID. No two Constraint nodes may share the same
ID. The same rule applies to Amendments and Profiles independently.

A Shape and a Constraint MAY have the same numeric ID value, because they
occupy separate namespaces.

**Rationale:**
Node IDs are the primary addressing mechanism. Every operation that takes a
`(node_type, id)` pair assumes it resolves to exactly one node. Duplicate IDs
within a type would make `get`, `query.ancestors`, `query.descendants`, and
all reference resolution ambiguous. The file-based storage format (one file per
node, named `{id}.yaml`) inherently prevents duplicates at the filesystem
level, but implementations using other storage backends MUST enforce this
explicitly.

**Detection method:**
For each node type, collect all node IDs and check for duplicates. In
file-based storage, this is equivalent to checking for duplicate filenames
within a subdirectory. For other storage backends, query all IDs per type and
verify uniqueness.

---

### INV-019: Amendment-Log Reciprocity

**Level:** MUST
**Severity:** error
**Applies to:** Amendment nodes, Shape nodes, Constraint nodes, Profile nodes

**Formal statement:**
For every Amendment node A and every node N in the union of
`A.targets.shape_ids`, `A.targets.constraint_ids`, and
`A.targets.profile_ids`, `N.amendment_log` MUST contain `A.id`.

Conversely, for every Shape, Constraint, or Profile node N and every
amendment ID in `N.amendment_log`, the referenced Amendment A MUST list
N in the corresponding target array (`shape_ids`, `constraint_ids`, or
`profile_ids`).

**Rationale:**
The amendment graph is navigable in both directions: forward from an
Amendment through `targets` to the nodes it modifies, and reverse from a
node through `amendment_log` to the amendments that have modified it.
If these two traversal directions disagree, the question "which
amendments have modified this node?" cannot be answered consistently,
and agents relying on the amendment history will see partial or
inconsistent results depending on which direction they walk.

**Detection method:**
For each amendment A, for each target node N: verify that
`N.amendment_log` contains `A.id`. For each node N, for each amendment
ID in `N.amendment_log`: verify that the corresponding Amendment's
`targets` array for N's node type contains N's ID. Report each missing
back-link with the offending node and the missing ID.
