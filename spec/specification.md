# Shapes Specification

**Version:** 0.1.0 -- Working Draft -- March 2026

## Notation

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

---

## 1. The Intent Layer

Shapes is an open specification that defines a semantic layer for
collaboration between agents and humans. This layer -- the **intent layer** --
captures *what* is to be built and *why*, while lower layers address *how* it
is done.

The specification captures five categories of structured information:

- **Intent** -- Purpose and motivation behind each unit of work.
- **Constraints** -- Strict invariants and enforcement rules that work MUST
  satisfy.
- **Bindings** -- Connections to external artifacts: Realization for
  deliverables, Evidence for proof of satisfaction, Provenance for decision
  history.
- **Amendments** -- Immutable change records that preserve lineage while
  keeping targeted nodes lean.
- **Boundaries** -- Scope derived from graph structure: parent-child
  composition, constraint inheritance, and profile governance.

Shapes sits between agents and the work that must be done. It provides a
structured representation of the entire history and current state of a
project, enabling agents to operate with full context.

The specification does not require datetime values at the base level. Profiles
MAY declare date fields as required within status metadata or other sections.

---

## 2. Motivation

Existing tools record *what changed* but not *why it matters*.
Provenance-only systems capture session history but do not transform it into
a queryable, semantically structured context layer.

As projects grow, agents require a shared semantic surface to plan, build,
and review work. Human review does not scale to concurrent agent workflows.
Local AI reviewers lack visibility into broader intent and constraints.

Shapes addresses this gap by capturing intent, constraints, bindings, and
amendment history as first-class structured records forming a queryable graph
that any agent can read, evaluate, and act on.

---

## 3. The DAG

The specification is format-agnostic and storage-agnostic. Examples in this
document use YAML for readability; implementations MAY use any serialization
format.

The specification maintains two composition DAGs:

1. **Shape composition graph** -- Shapes compose hierarchically through
   parent-child relationships, representing decomposition of work (systems
   into components, features into tasks).
2. **Constraint composition graph** -- Constraints compose hierarchically
   through parent-child relationships, representing refinement of rules
   (broad architectural rules into specific enforcement policies).

These DAGs express systems of systems. Nodes compose hierarchically and
cross-reference laterally without cycles.

Implementations MUST reject any mutation that would introduce a cycle in
either DAG. Mutual dependencies are modeled through shared parents or lateral
cross-references via Bindings, not through cycles.

### 3.1 Identifiers

IDs are unique per node type. A Shape and a Constraint MAY share the same ID
value; they occupy separate namespaces. IDs are opaque -- implementations MAY
use non-negative integers, non-empty strings, UUIDs, or any other scheme. Two
IDs are equal if and only if they have the same type and the same value.

### 3.2 Inline and Standalone References

Both Shapes and Constraints support inline and standalone node references.
When a child entry is an object containing a full node definition, it is an
**inline node** owned by its parent. When it is a scalar (an ID value), it is
a **reference by ID** to a standalone node defined elsewhere in the graph.

---

## 4. Node Types

The specification defines four node types. Each is specified below with its
required and optional fields. Formal schemas are in the `schema/` directory.

### 4.1 Shape

The primary node describing what is being built and why. Shapes compose
through children forming a DAG.

**Required fields:** `id`, `name`, `description`, `status`, `intent`.

**Optional fields:** `profile`, `version`, `predecessors`, `constraints`,
`realization`, `evidence`, `provenance`, `amendment_log`, `parents`,
`children`, `metadata`.

A Shape's `constraints` array references Constraint nodes by ID. These
constraints are in scope for that Shape and all its descendants (constraint
inheritance).

A Shape's `children` array contains child references. Each child reference
wraps either a scalar ID (referencing a standalone Shape) or a full inline
Shape definition. Child references MAY carry `role` and `reason` metadata.

A Shape's `parents` array contains parent references. Each parent reference
carries a required `id` and optional `role` and `reason` metadata.

See `schema/shape.json`.

### 4.2 Constraint

A strict enforcement rule that MUST be satisfied by any Shape referencing it.
Constraints are referenced by ID from Shapes and discovered during graph
traversal. Constraints referenced by a Shape are in scope for that Shape and
all its descendants (inheritance). Constraints MAY form their own composition
hierarchy through parents and children.

**Required fields:** `id`, `name`, `description`, `kind`, `rule`,
`enforcement`, `status`, `intent`.

**Optional fields:** `profile`, `version`, `realization`, `evidence`,
`provenance`, `amendment_log`, `parents`, `children`, `metadata`.

The `kind` field classifies the constraint (e.g., `testing`, `architecture`,
`style`, `security`). The `rule` field states the concrete condition that
MUST hold. The `enforcement` field declares how the constraint is enforced
(e.g., `ci`, `manual-review`, `lint`, `test`).

See `schema/constraint.json`.

### 4.3 Amendment

An immutable change record applied to Shapes, Constraints, or Profiles.
Amendments preserve lineage while keeping targeted nodes lean. Once created,
amendments are append-only -- they MUST NOT be modified or deleted.

**Required fields:** `id`, `name`, `description`, `targets`, `status`,
`intent`, `initiated_by`.

**Optional fields:** `version_impact`, `constraints`, `realization`,
`evidence`, `provenance`, `archived`, `metadata`.

The `archived` field is display-only metadata. It accepts either a bare
boolean (`true`) or an object with a `reason` field
(`{reason: "Changes integrated into target shapes"}`). When present and
truthy, the amendment is hidden from default listing output — `shapes list
amendment` drops it, and `shapes get <parent>` omits it from the rendered
`amendment_log` unless `--archived` is passed. Archiving is not deletion:
the record stays on disk, reciprocity (INV-019) still applies, and CI-002
still counts the amendment toward satisfying changes to promoted or
canonical targets. Archiving exists so stale audit entries whose insight
value has decayed can be suppressed from routine reads without losing the
audit trail. Toggling `archived` (including adding or updating its
`reason`) is the sole permitted mutation of a canonical amendment under
CI-003; every other field remains strictly immutable.

The `targets` field specifies which nodes are affected. It is an object with
three optional arrays: `shape_ids`, `constraint_ids`, `profile_ids`. At
least one array MUST be present and non-empty (INV-007). All referenced IDs
MUST resolve to existing nodes (INV-008).

The `initiated_by` field identifies the actor who initiated the amendment.
It requires a `type` (e.g., `human`, `agent`, `ci`, `system`) and optionally
carries `identity` and `provenance`.

See `schema/amendment.json`.

### 4.4 Profile

A governance configuration defining lifecycle gates, field requirements,
versioning rules, and amendment models. Profiles govern how nodes in the
graph evolve.

**Required fields:** `id`, `name`, `description`, `status`, `intent`.

**Optional fields:** `version`, `provenance`, `lifecycle`, `fields`,
`versioning`, `amendment_rules`, `amendment_log`, `metadata`.

A Profile's `lifecycle` section defines custom statuses and gates (transition
rules with preconditions and postconditions). The `fields` section declares
required and optional fields for Shapes and Constraints governed by the
Profile. The `versioning` section configures the versioning scheme. The
`amendment_rules` section specifies the amendment application strategy
(`merge`, `overlay`, `edition`, or `append-only`).

See `schema/profile.json`.

---

## 5. Intent

Intent captures the purpose of a node -- *what* is to be built and *why*.

Every Intent MUST include:

- **`kind`** (string) -- A domain label classifying the intent (e.g.,
  `feature`, `bugfix`, `refactor`, `system`, `component`).
- **`summary`** (string) -- A human-readable description of what the node
  intends.
- **`source`** -- The origin of this intent (e.g., `human`, `ai`, `system`).
  The value is free-form; Profiles MAY constrain allowed sources.

Intent MAY include:

- **`uris`** (array of strings) -- URIs providing additional context or
  references.

Beyond required fields, Intent is an open map. The schema sets
`additionalProperties: true`. Each domain extends it with its own
vocabulary: software teams add `goals` and `non_goals`, research labs add
`hypotheses`, editorial teams add `themes`.

When a Shape is decomposed into sub-Shapes, each sub-Shape's Intent MUST
remain coherent with its parent's Intent.

See `schema/intent.json`.

---

## 6. Status & Lifecycle

Status is a tagged union with 7 states divided into progressive and terminal:

**Progressive states:** `proposed` -> `promoted` -> `canonical`

**Terminal states:** `rejected`, `superseded`, `abandoned`, `reverted`

### 6.1 Serialization

Status MAY be serialized as:

- A bare string when no detail is needed: `"proposed"`
- A single-key object when detail fields are present:
  `{canonical: {reason: "...", metadata: {...}}}`

### 6.2 Detail Fields

Progressive states (`proposed`, `promoted`, `canonical`) carry optional
detail: `reason` (string), `uris` (array of strings), `metadata` (object).

Terminal states (`rejected`, `superseded`, `abandoned`, `reverted`) carry
optional detail: `reason` (string), `uris` (array of strings), `successors`
(array of NodeIds), `metadata` (object). For `superseded` status, the
`successors` field SHOULD be present to identify replacement nodes.

### 6.3 Transition Rules

State transitions are governed by Profile-defined gates with preconditions
and postconditions. The following rules apply:

- Nodes in `proposed` status MAY be edited directly.
- Nodes in `promoted` or `canonical` status MUST use Amendments for changes.
- Terminal states are immutable. A node in a terminal state MUST NOT be
  modified.

See `schema/status.json`.

---

## 7. Bindings

Bindings connect nodes to external artifacts using scheme-based references.
Three binding types are defined.

### 7.1 Binding

A single artifact reference. The atomic unit of all binding types.

**Required fields:** `scheme` (string), `value` (string).

**Optional fields:** `metadata` (object).

The `scheme` identifies the type of artifact reference (e.g., `path`, `url`,
`commit`, `git`). The `value` locates the artifact within that scheme (e.g.,
a file path, URL, or git ref). The scheme vocabulary is open; implementations
MAY define additional schemes.

See `schema/binding.json`.

### 7.2 Realization

Maps nodes to deliverables -- code, documents, configurations, or any
concrete artifact that fulfills intent.

**Required fields:** `bindings` (array of Binding), `role` (string).

The `role` field carries a semantic label: `primary`, `supporting`, `test`,
`implementation`, `documentation`, or any domain-specific value.

See `schema/realization.json`.

### 7.3 Evidence

Proves constraint satisfaction. Links a verification record to supporting
artifacts.

**Required fields:** `id` (string), `type` (string), `bindings` (array of
Binding).

**Optional fields:** `trusted` (boolean), `metadata` (object).

The `type` classifies the evidence (e.g., `test-result`, `review`, `audit`).

See `schema/evidence.json`.

### 7.4 Provenance

Records decision history and origin. Documents how and why a node was created
or changed.

**Required fields:** `type` (string), `bindings` (array of Binding).

**Optional fields:** `metadata` (object).

The `type` classifies the provenance record (e.g., `decision`, `migration`,
`import`).

See `schema/provenance.json`.

---

## 8. Operations

The specification defines 10 abstract operations: 8 core read operations that
conforming implementations MUST implement, and 2 write operations that
implementations SHOULD implement.

Operations use `node_type` (`shape`, `constraint`, `amendment`, `profile`)
which, for DAG traversal operations, implicitly selects the correct
composition graph.

### 8.1 Core Read Operations (MUST)

| Operation | Purpose |
|-----------|---------|
| `discover` | Probe whether a Shapes graph exists and return summary metadata. |
| `get` | Retrieve the full definition of a single node by type and ID. |
| `list` | List nodes with optional filters by type, status, and kind. |
| `tree` | Render the composition hierarchy. Defaults to the shape DAG with constraints shown inline. |
| `query.ancestors` | Walk up the parent chain and return all ancestor IDs in BFS order. |
| `query.descendants` | Walk down the child tree and return all descendant IDs in BFS order. |
| `query.constraints` | Compute the full set of effective constraints for a shape, including those inherited from ancestors. |
| `validate` | Check the entire graph against all invariants (INV-001 through INV-011). Return all violations. |

The `validate` operation MUST check all invariants in a single pass and MUST
NOT stop at the first violation.

### 8.2 Write Operations (SHOULD)

| Operation | Purpose |
|-----------|---------|
| `init` | Initialize a new, empty Shapes graph. |
| `create` | Create a new node with an auto-assigned ID and add it to the graph. |

The `create` operation MUST verify that the graph remains valid after
insertion. If validation fails, the creation MUST be rejected and the graph
MUST remain unchanged.

See [operations.md](operations.md) for the full operation catalog with
parameters, return types, preconditions, postconditions, and error
conditions.

---

## 9. Invariants

The specification defines 11 structural invariants (INV-001 through INV-011). All
are MUST-level with error severity. A graph that violates any invariant is in
an invalid state and MUST be reported by the `validate` operation.

| ID | Rule |
|----|------|
| INV-001 | Shape composition graph MUST be acyclic. |
| INV-002 | Constraint composition graph MUST be acyclic. |
| INV-003 | Shape constraint references MUST resolve to existing Constraints. |
| INV-004 | Parent references MUST resolve to existing nodes of the same type. |
| INV-005 | Child references MUST resolve to existing nodes of the same type. |
| INV-006 | Profile references MUST resolve to existing Profiles. |
| INV-007 | Every Amendment MUST target at least one node. |
| INV-008 | Amendment target references MUST resolve to existing nodes. |
| INV-009 | Parent-child links MUST be reciprocal. |
| INV-010 | Nodes governed by a Profile MUST satisfy required field declarations. |
| INV-011 | IDs MUST be unique within their node type namespace. |

See [invariants.md](invariants.md) for formal definitions, rationale, and
detection methods for each invariant.

---

## 10. Discovery

Discovery is the prerequisite for all other spec operations. Three
mechanisms are defined. An implementation MUST support at least one.
Consumers SHOULD attempt mechanisms in the order listed and use the first
that succeeds.

1. **Filesystem** -- A `.shapes/` directory containing `meta.yaml` and
   subdirectories for each node type. Discovery uses a walk-up algorithm from
   the current working directory to the filesystem root. This is the primary
   mechanism for local tooling and CLI-based agents.

2. **MCP** -- An MCP server implementing the specification MUST advertise a
   `shapes_discover` tool. The presence of any `shapes_`-prefixed tools in
   the MCP tool list signals spec support.

3. **HTTP** -- `GET /.well-known/shapes` returns graph metadata as JSON with
   a `200 OK` response, or `404 Not Found` if no graph is available. This
   follows [RFC 8615](https://www.rfc-editor.org/rfc/rfc8615).

A consumer MUST NOT combine metadata from multiple discovery sources. A
single discovered graph is the unit of interaction for all subsequent
operations.

See [discovery.md](discovery.md) for the full specification of each
mechanism.

---

## 11. Conformance

An implementation conforms to the Shapes Specification if and only if:

1. It implements all MUST-level operations as defined in
   [operations.md](operations.md).
2. It enforces all MUST-level invariants as defined in
   [invariants.md](invariants.md).
3. It loads every valid test vector without error and reports zero invariant
   violations.
4. It rejects every invalid test vector, reporting at minimum the expected
   invariant violations listed in the fixture's `expected.json`.
5. Its data output validates against the JSON Schemas in `schema/`.

An implementation that passes all valid vectors but misses an expected
invariant on any invalid vector is non-conforming.

See [conformance/conformance.md](conformance/conformance.md) for the test
vector inventory and execution procedure.

---

## 12. Future Extensions

The following are identified extension points for future versions of the
specification. They are non-normative and carry no requirements in v0.1.0.

- **Profile DAG** -- Add `parents` and `children` to Profile for governance
  inheritance (organization -> team -> project profiles).
- **Reverse constraint queries** -- Query which Shapes a Constraint governs.
  Currently, constraint-to-shape resolution is one-directional (shapes
  reference constraints).
- **Cross-project references** -- Allow inline nodes owned by external
  projects, enabling multi-project graphs.
- **Real-time subscriptions** -- Event-based notification of graph changes
  for reactive agent workflows.

---

## Appendix A: JSON Schema Reference

All schemas use JSON Schema Draft 2020-12 and are published under the
`https://shapes.fyi/schema/v0.1.0/` namespace.

| Schema | Description |
|--------|-------------|
| [`schema/shape.json`](schema/shape.json) | Shape node definition. |
| [`schema/constraint.json`](schema/constraint.json) | Constraint node definition. |
| [`schema/amendment.json`](schema/amendment.json) | Amendment node definition. |
| [`schema/profile.json`](schema/profile.json) | Profile node definition. |
| [`schema/intent.json`](schema/intent.json) | Intent open map (kind, summary, source, plus additional properties). |
| [`schema/status.json`](schema/status.json) | Seven-state lifecycle status (bare string or single-key object). |
| [`schema/binding.json`](schema/binding.json) | Single external artifact reference (scheme + value). |
| [`schema/realization.json`](schema/realization.json) | Deliverable binding with semantic role. |
| [`schema/evidence.json`](schema/evidence.json) | Constraint satisfaction proof. |
| [`schema/provenance.json`](schema/provenance.json) | Decision history and origin record. |
| [`schema/node-id.json`](schema/node-id.json) | Opaque node identifier (non-negative integer or non-empty string). |
| [`schema/parent-ref.json`](schema/parent-ref.json) | Parent reference with optional role and reason. |
| [`schema/meta.json`](schema/meta.json) | Graph metadata (spec version and next-ID counters). |

---

## Appendix B: Transport Bindings

Transport bindings map the abstract operations defined in this specification
to concrete interfaces. Each binding document specifies command syntax or
tool schemas, serialization formats, error handling, and transport-specific
conventions.

| Binding | Description |
|---------|-------------|
| [`bindings/cli.md`](bindings/cli.md) | CLI transport binding. Maps operations to the `shapes` command-line tool. Reference implementation. |
| [`bindings/mcp.md`](bindings/mcp.md) | MCP server transport binding. Maps operations to MCP tools with JSON parameters and responses. |
| [`bindings/http.md`](bindings/http.md) | HTTP transport binding. Maps operations to REST endpoints with JSON request and response bodies. |
