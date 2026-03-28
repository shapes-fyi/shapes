---
name: shapes-context
description: >
  Shapes Context Protocol knowledge. Loaded automatically when a project has
  a .shapes/ directory or the user mentions shapes, context, or constraints.
  Teaches the protocol concepts, context-first workflow, and how to use the
  shapes-cli to discover intent and constraints before doing any work. Use
  /shapes:shapes-init to bootstrap a new project. Use /shapes:shapes-maintain
  to audit and organize an existing shapes graph.
user-invocable: true
---

# Shapes Context Protocol

## Prerequisites

The `shapes` CLI must be installed. If `shapes --help` doesn't work, install it:

```bash
cargo install --git https://github.com/shapes-fyi/shapes
```

Run `shapes --help` and `shapes <command> --help` for all commands and flags.

## The Protocol

Shapes is an Open Context Protocol that captures the **intent**, **constraints**,
and **boundaries** of a project as a structured, queryable graph. It is
**domain-agnostic** — it applies to software, research, writing, and any
structured endeavor. Only the Profile configuration and Intent vocabulary
change between domains.

### Four Node Types

| Node | Purpose |
|------|---------|
| **Shape** | What to build and why — the primary work node |
| **Constraint** | Rules that must be satisfied — strict invariants |
| **Amendment** | Immutable change record — evolves the graph over time |
| **Profile** | Governance configuration — defines lifecycle, custom fields, amendment rules |

### Two DAGs

Shapes and Constraints each form their own **Directed Acyclic Graph** through
parent/child relationships. These are independent graphs:

- **Shape DAG** — composition hierarchy (system → services → features → ...)
- **Constraint DAG** — policy decomposition (policy → sub-rules → ...)

Shapes reference Constraints by ID. When you traverse a Shape's ancestors, you
discover all constraints that apply — constraints are inherited downward.

### Intent: The Open Map

Every Shape, Constraint, and Amendment carries an **Intent** with three
required fields:

- **kind** — domain label (free-form string)
- **summary** — human-readable description
- **source** — origin (human, ai, system)

Beyond these, Intent is an **open map**. Each domain extends it with its own
vocabulary. The protocol doesn't prescribe which fields exist — that's the
Profile's job.

### Profiles: Defining What Fields Matter

A **Profile** governs how Shapes and Constraints behave in a domain. It:

- **Declares custom fields** via FieldDef — name, description, type, and
  whether required or optional. Fields can be declared for intent, status,
  constraints, realization, evidence, provenance, and metadata sections.
- **Defines lifecycle gates** — preconditions for state transitions
- **Specifies allowed kinds** — what `kind` values are valid
- **Chooses amendment model** — merge, overlay, edition, or append-only

### Bindings

Bindings connect Shapes to external artifacts via a `scheme` and `value`.
For source code, use `scheme: path` with a repository-relative file path.
Add `metadata.summary` to describe what the binding points to — the
specific structs, functions, or sections relevant to this shape:

```yaml
realization:
  - bindings:
      - scheme: path
        value: src/model/common.rs
        metadata:
          summary: Intent struct — open map with kind/summary/source + flattened extra fields
    role: primary
```

The summary gives agents semantic context about what's in the file without
having to read it. This is especially valuable when multiple shapes point
to the same file — each summary describes the relevant portion.

**Realizations** connect Shapes to deliverables (code, docs, services). Each
has bindings and a `role` (primary, supporting, test).

**Evidence** demonstrates constraint satisfaction — type (test, review, metric),
trusted indicator, and bindings to verification artifacts.

**Provenance** tracks decision history — links to discussions, reviews, sessions.

### Lifecycle

Seven states: proposed → promoted → canonical (progressive), plus rejected,
superseded, abandoned, reverted (terminal). Direct edits allowed while
`proposed`; `promoted`/`canonical` changes require Amendments.

## Context-First Workflow

When a project has `.shapes/`, always discover context before doing any work.
Shapes are an **interactive discovery tool** — query them on demand.

### Before Every Task

1. **See the big picture** — `shapes tree shape`
2. **Find the relevant shape** — `shapes list` with filters
3. **Read the intent** — `shapes get shape <id>`
4. **Discover constraints** — `shapes query constraints <shape-id>`, then
   `shapes get constraint <id>` for the actual rules
5. **Now do the work** — with full understanding of intent and constraints

### After Completing Work

- Edit the shape's YAML to add **realizations** (bindings to files you
  created or modified). Use `scheme: path` with the file path and include
  `metadata.summary` describing the relevant constructs.
- Add **evidence** if you satisfied a constraint
- Create an **amendment** if a canonical shape changed significantly

### When Starting New Work

Create the shape first, work second:

```bash
shapes create shape --name "OAuth Integration" --kind feature \
  --summary "Add OAuth2 login flow"
```

Edit the YAML to add parent references, constraints, and flesh out the
intent with whatever fields the project's Profile defines.

### Respecting Constraints

Constraints from `shapes query constraints <id>` apply to your work. Read
each constraint's `rule` field. Constraints are inherited — a constraint on
a parent shape applies to all descendants.

## Writing Good Shapes and Constraints

### Shapes

A shape should capture enough context that someone who has never seen the
code can understand what it is, why it exists, and what rules govern it.
The specific fields depend on the project's Profile. But thin, one-liner
shapes waste the opportunity to capture the meaning behind the code.

### Constraints

A constraint is only useful if its `rule` is specific enough to verify.
Vague rules like "be secure" aren't actionable. Good rules state a
falsifiable invariant an agent can check by reading the code.

The `description` should explain why the rule exists. Without the "why,"
future contributors won't understand the cost of violating it.

## CLI Reference

Run `shapes --help` for the full command list, and `shapes <command> --help`
for detailed usage. The CLI is self-documenting.
