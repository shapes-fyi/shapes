---
name: shapes-context
description: >
  Teaches the Shapes Context Protocol and shapes-first workflow. Triggers
  when starting work in a project with a .shapes/ directory, or when exploring
  project architecture, intent, constraints, or shape graph structure. Covers
  node types, DAGs, lifecycle, bindings, profiles, and the shapes-first
  principle: plan changes in the graph before writing any code.
user-invocable: true
---

# Shapes Context Protocol

## Contents

- The Shapes-First Principle
- The Protocol (node types, DAGs, intent, profiles, bindings, lifecycle)
- Shapes-First Workflow
- Writing Good Shapes and Constraints
- Related Skills
- CLI Reference

## Related Skills

- `/shapes:shapes-init` — bootstrap a new project with shapes, profiles, and constraints
- `/shapes:shapes-maintain` — keep the shapes graph in sync with code changes;
  includes the decision framework for amendments vs new shapes vs direct edits

## The Shapes-First Principle

**No code changes without shapes changes first.** The graph is the plan;
code is the execution. Before writing or modifying any source code:

1. Create or amend the relevant shapes, constraints, or amendments
2. Capture what you intend to build, change, or enforce
3. Only then write the code that realizes that intent

This applies to every kind of work:
- **New feature** — create the shape first, then implement it
- **Bug fix** — amend the affected shape or add a constraint first, then fix
- **Refactor** — update shapes to reflect the new structure first, then move code
- **New rule** — create the constraint first, then enforce it in code

The graph is not documentation written after the fact. It is the source of
intent that drives implementation.

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

- **Shape DAG** — composition hierarchy (system > services > features > ...)
- **Constraint DAG** — policy decomposition (policy > sub-rules > ...)

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

Seven states: proposed > promoted > canonical (progressive), plus rejected,
superseded, abandoned, reverted (terminal). Direct edits allowed while
`proposed`; `promoted`/`canonical` changes require Amendments.

## Shapes-First Workflow

When a project has `.shapes/`, the graph drives all work. No code changes
happen until the graph reflects what you intend to do.

### Step 1: Discover Context

1. **See the big picture** — `shapes tree shape`
2. **Find the relevant shape** — `shapes list` with filters
3. **Read the intent** — `shapes get shape <id>`
4. **Discover constraints** — `shapes query constraints <shape-id>`, then
   `shapes get constraint <id>` for the actual rules

### Step 2: Plan in the Graph

Before writing any code, update the graph to capture your planned changes:

- **New feature** — `shapes create shape` with intent, parent links, constraints
- **Changing an existing feature** — create an amendment (if promoted/canonical)
  or edit the shape directly (if proposed)
- **New rule or invariant** — `shapes create constraint` with a specific,
  falsifiable rule
- **Bug fix** — amend the shape to document the fix, or add a constraint to
  prevent recurrence

Edit the YAML to flesh out intent fields, add parent/child references, and
link relevant constraints. This is your plan.

### Step 3: Write Code

Now implement what the graph describes. The shapes you created or amended
define the scope — don't exceed it without updating the graph first.

### Step 4: Bind Realizations

After code is written, add realization bindings to connect shapes to the
files you created or modified. The shapes-maintain skill auto-triggers here
with the full decision framework for updating realizations.

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
