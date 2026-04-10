---
name: shapes-maintain
description: >
  Keeps the shapes graph in sync with code changes. Triggers when editing
  files, creating shapes, preparing commits, or completing tasks in a project
  with .shapes/. Covers the decision framework for amendments, realizations,
  and validation.
user-invocable: true
---

```!
shapes preflight 2>/dev/null || echo "Shapes CLI not found. Install it: cargo install shapes-cli"
```

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
| Change behavior or scope | proposed (same session) | Edit the shape YAML directly |
| Change behavior or scope | proposed (prior session) | `shapes create amendment --target-shape <id>` — amendments document cross-session changes |
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
If it's `proposed` and you created it in this session, edit directly.

### Continuation Sessions

When you are continuing work on a project you did not initialize in this
session, **always create amendments** for shapes you modify, even if they
are `proposed`. Amendments are the only cross-session change log. Without
them, future agents have no way to understand what changed between iterations.

Run `shapes list amendment` at the start of each continuation session to
read the change history. This tells you what already changed and why,
preventing redundant or conflicting modifications.

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

### Realization timing

Write realization bindings **after** the code exists, not during planning.
If you add bindings during planning as placeholders, you must re-verify
them during the shapes-maintain step:

1. Read each binding's `metadata.summary`
2. Read the actual file it points to
3. Verify the summary describes the current code — not the planned code
4. Update any summaries that diverged from the plan

A stale summary is worse than no summary — it misleads future agents.

## Before Every Commit

### 1. Verify constraints

Read each constraint in `.shapes/constraints/` and verify the code satisfies
it. Constraints are the rules the project has committed to — they are the
source of truth for what is and isn't acceptable.

For each constraint:

1. Read the `rule` field — it states a specific, falsifiable condition.
2. Check the code you wrote or modified against that rule.
3. If the code violates the rule, fix the code before committing.

Also verify scope: did you modify code outside the shapes you planned to
change in Step 2? If so, either update the graph to reflect the broader
scope or revert the unplanned changes. The graph defines what you intended
to do — the diff should match the graph.

### 2. Validate the graph

```
shapes validate
```

If exit code is 2, fix the reported issues before committing. Common fixes:

- **Dangling references** — a shape references a constraint or parent that
  doesn't exist. Remove the reference or create the missing node.
- **Missing reciprocal links** — if a parent lists a child, the child must
  list the parent back. Add the missing link.
- **Profile field violations** — add required fields declared by the Profile.

### 3. Update realizations

Verify realization bindings are accurate for any files you created, modified,
renamed, or deleted per the "Updating Realizations" section above.

## Deep Audit

For periodic health checks (after major refactors, before releases, or when
the graph feels stale), run the full audit process described in
[DEEP-AUDIT.md](DEEP-AUDIT.md). This covers duplicate detection, coverage
gaps, shallow node enrichment, and structural organization.

A deep audit SHOULD also include an **amendment archival pass**. Invoke
`/shapes:shapes-archive` to walk every unarchived amendment and decide
which ones to archive per its decision framework.

Invoke with `/shapes:shapes-maintain` and request a deep audit explicitly.

## CLI Quick Reference

```bash
# Query
shapes tree shape                        # Full shape hierarchy
shapes tree constraint                   # Full constraint hierarchy
shapes get shape <id>                    # Read shape definition
shapes get constraint <id>               # Read constraint definition
shapes query constraints <shape-id>      # Constraints that apply (inherited)
shapes list amendment                    # Unarchived amendments (change history)
shapes list amendment --archived         # Include archived amendments too
shapes get shape <id>                    # Shape with archived amendments hidden
shapes get shape <id> --archived         # Shape with archived amendments annotated
shapes validate                          # Check integrity (exit 0 = clean)

# Create (exact required flags)
shapes create shape --name "X" --kind feature --summary "Y"
shapes create constraint --name "X" --kind invariant --rule "Y" --enforcement machine
shapes create amendment --name "X" --target-shape <id> --summary "Y"
shapes create profile --name "X"
```

There is no `shapes edit` command. Edit YAML files directly at
`.shapes/shapes/<id>-<name>.yaml` after creating them.
