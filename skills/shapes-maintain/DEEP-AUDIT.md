# Deep Audit Process

A comprehensive health check for the shapes graph. Run periodically — after
major refactors, before releases, or when coverage feels stale. Not needed
before every commit; the incremental workflow in SKILL.md handles daily use.

## Progress Checklist

```
Deep Audit Progress:
- [ ] Step 1: Full graph loaded and reviewed
- [ ] Step 2: Validation errors fixed (or none found)
- [ ] Step 3: Duplicates checked (merges proposed if found)
- [ ] Step 4: Shallow nodes identified (enriched or flagged)
- [ ] Step 5: Realization accuracy verified (paths exist, summaries current)
- [ ] Step 6: Coverage gaps identified (uncovered files, missing constraints)
- [ ] Step 7: Structural organization reviewed
- [ ] Step 8: Findings presented to user by severity
- [ ] Step 9: Approved changes applied and validated clean
```

## Step 1: Load the Full Graph

```bash
shapes tree shape
shapes tree constraint
shapes validate
shapes list
```

Read the tree output to understand the current structure. Note the validate
output — any existing integrity issues are the first priority.

## Step 2: Fix Validation Errors

If `shapes validate` reported issues (exit code 2), address them first:

- **Cycles** — restructure parent/child relationships to eliminate cycles
- **Dangling references** — remove references to non-existent nodes or
  create the missing nodes
- **Missing reciprocal links** — if parent lists child, ensure child lists
  parent (and vice versa)
- **Empty amendment targets** — every amendment must target at least one node
- **Profile field violations** — add required fields or update the Profile

## Step 3: Check for Duplicates

Read each shape and constraint. Look for:

- **Duplicate shapes** — different nodes describing the same component or
  feature. Compare names, descriptions, and realizations. Propose merging
  duplicates and updating all references.
- **Overlapping constraints** — rules that cover the same invariant with
  different wording. Propose consolidating into one constraint with a clear,
  comprehensive rule.

## Step 4: Check for Shallow Nodes

Identify shapes and constraints that are too thin to be useful:

- Shapes with one-line descriptions and no intent detail
- Shapes with no realizations (not linked to any source files)
- Constraints with vague rules (not specific enough to verify)
- Constraints with no description (missing the "why")

For each shallow node, read the relevant source code and flesh out the
content, or ask the user for the missing context.

## Step 5: Check Realization Accuracy

For each realization binding, verify the referenced file still exists.
Read each shape's YAML, extract the `value` field from each binding, and
check whether the file is present at that path in the project.

Flag:
- **Stale realizations** — bindings pointing to files that were renamed,
  moved, or deleted. Update or remove them.
- **Missing summaries** — bindings without `metadata.summary`. Every
  binding should describe the specific constructs relevant to the shape.
- **Stale summaries** — summaries that no longer match the file contents
  (e.g., function was renamed or moved). Read the source to verify.
- **Missing realizations** — shapes with no realization bindings at all.
  Find the source files that implement them and add bindings.

## Step 6: Check Coverage

Identify gaps in the shapes graph:

- **Uncovered source files** — scan the project's source tree and find files
  or directories not referenced by any shape's realization. These may need
  new shapes.
- **Shapes without constraints** — components with no rules governing them.
  Ask the user if there are invariants that should be captured.
- **Orphan nodes** — shapes or constraints with no parent and no children
  that aren't the root. These may be misplaced in the DAG.

## Step 7: Check Structural Organization

Review the DAG structure:

- **Flat hierarchies** — if most shapes are direct children of the root,
  consider adding intermediate grouping nodes (services, modules).
- **Deep narrow chains** — long parent-child chains with no branching may
  indicate over-decomposition. Consider flattening.
- **Cross-cutting concerns** — if the same constraint appears on many
  unrelated shapes, consider restructuring the Constraint DAG to use
  parent constraints that apply broadly.

## Step 8: Present Findings

Summarize all findings for the user organized by severity:

1. **Errors** — validation failures, broken references
2. **Duplicates** — shapes or constraints that should be merged
3. **Gaps** — missing coverage, shallow nodes
4. **Suggestions** — structural improvements, reorganization ideas

For each finding, explain what's wrong and propose a specific fix. Wait for
user approval before making any changes.

## Step 9: Apply Approved Changes

After the user approves, apply changes by editing YAML files and validating:

```bash
shapes validate
```

If `shapes validate` reports new errors introduced by the changes, fix them
and re-validate. Repeat until exit code 0.

Once clean, show the updated structure:

```bash
shapes tree shape
shapes tree constraint
```

Confirm the graph is clean and summarize the changes applied.
