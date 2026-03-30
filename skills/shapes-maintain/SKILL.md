---
name: shapes-maintain
description: >
  Keeps the shapes graph in sync with code changes. Triggers when planning
  work in the graph, editing source files, creating new files, refactoring
  code, preparing git commits, or completing coding tasks in a project with
  a .shapes/ directory. Provides the decision framework for when to create
  shapes, amendments, or constraints before writing code, how to bind
  realizations after, and ensures the graph stays valid and current.
user-invocable: true
---

# Keeping the Shapes Graph in Sync

Shapes are planned before code is written. After implementation, the graph
needs realizations bound, summaries updated, and validation passed. This
skill handles that post-implementation sync and the ongoing decision
framework for how to evolve the graph.

## Contents

- Decision Framework
- Amendment Rules
- Updating Realizations
- Before Every Commit
- Deep Audit (periodic)

## Decision Framework

### Before writing code (plan in the graph)

| What you plan to do | Shape status | Action |
|---|---|---|
| Build a new feature or component | — | `shapes create shape` + link to parent |
| Change behavior or scope | proposed | Edit the shape YAML directly |
| Change behavior or scope | promoted or canonical | `shapes create amendment --target-shape <id>` |
| Introduce a new rule or invariant | — | `shapes create constraint` |
| Fix a bug | any | Amend the shape or add a constraint to prevent recurrence |
| Remove a feature | any | Set shape status to `abandoned` or `superseded` |

### After writing code (bind realizations)

| What happened in code | Action |
|---|---|
| Created a new source file | Add a realization binding to the appropriate shape |
| Modified an existing file | Update the realization `metadata.summary` if meaning changed |
| Renamed or moved a file | Update the realization binding `value` (path) |
| Refactored without behavior change | Update realization paths and summaries only |
| Deleted a file | Remove the stale realization binding |

**When in doubt:** if the shape is `promoted` or `canonical`, use an amendment.
If it's `proposed`, edit directly.

## Amendment Rules

Amendments are **immutable change records** for nodes that have graduated past
`proposed` status. They preserve lineage without bloating the original node.

### When amendments are required

- The target shape or constraint is `promoted` or `canonical`
- The change affects intent, scope, goals, or non-goals
- A constraint's `rule` is being modified

### When amendments are NOT needed

- The shape is still `proposed` — edit it directly
- You're only adding or updating realization bindings (file paths, summaries)
- You're fixing metadata (typos, formatting) that doesn't change meaning
- You're adding evidence for a constraint

### Creating an amendment

```bash
shapes create amendment \
  --name "Add multi-tenant support" \
  --target-shape 5 \
  --summary "Auth service now supports multiple tenants" \
  --version-impact minor
```

Then edit the amendment YAML to flesh out intent and add realization
bindings pointing to the changed files.

**Multiple targets:** use `--target-shape` and `--target-constraint` flags
repeatedly to target multiple nodes in one amendment.

## Updating Realizations

Realizations bind shapes to source files. Keep them accurate as code changes.

### Adding a realization

Edit the shape's YAML and add a binding:

```yaml
realization:
  - bindings:
      - scheme: path
        value: src/auth/oauth.rs
        metadata:
          summary: OAuth2 token exchange, session creation, refresh logic
    role: primary
```

The `metadata.summary` should describe the specific constructs in the file
that are relevant to this shape — not just "the auth module" but what's in
it that matters.

### When to update summaries

Update `metadata.summary` when:
- Functions or structs relevant to the shape were renamed
- Significant logic was added or removed
- The file's role in the shape changed

Don't update summaries for trivial changes (formatting, comments, imports).

## Before Every Commit

```
shapes validate
```

If exit code is 2, fix the reported issues before committing. Common fixes:

- **Dangling references** — a shape references a constraint or parent that
  doesn't exist. Remove the reference or create the missing node.
- **Missing reciprocal links** — if a parent lists a child, the child must
  list the parent back. Add the missing link.
- **Profile field violations** — add required fields declared by the Profile.

## Deep Audit

For periodic health checks (after major refactors, before releases, or when
the graph feels stale), run the full audit process described in
[DEEP-AUDIT.md](DEEP-AUDIT.md). This covers duplicate detection, coverage
gaps, shallow node enrichment, and structural organization.

Invoke with `/shapes:shapes-maintain` and request a deep audit explicitly.

## CLI Quick Reference

```bash
shapes tree shape                        # See project structure
shapes get shape <id>                    # Read a shape's full definition
shapes query constraints <id>            # Constraints that apply to a shape
shapes create shape --name "X" --kind feature --summary "Y"
shapes create amendment --name "X" --target-shape <id> --summary "Y"
shapes create constraint --name "X" --kind invariant --rule "Y"
shapes validate                          # Check graph integrity (exit 0 = clean)
```
