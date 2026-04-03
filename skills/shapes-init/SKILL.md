---
name: shapes-init
description: >
  Bootstraps the Shapes Context Protocol for an existing project. Explores
  the codebase, interviews the engineer about architecture and constraints,
  creates a Profile defining what fields matter, then generates a rich graph
  of shapes and constraints capturing the project's intent, rules, and
  boundaries.
user-invocable: true
argument-hint: "[project-path]"
---

# Bootstrap Shapes for a Project

The goal is to capture not just what the code does, but the **meaning behind
it** — the intent, the unwritten rules, the domain knowledge that lives in
the engineer's head. Code tells you *what* exists. Shapes capture *why*.

## Contents

- Step 1: Initialize the Store
- Step 2: Explore the Project
- Step 3: Interview the Engineer
- Step 4: Define the Profile
- Step 5: Create Shapes
- Step 6: Create Constraints
- Step 7: Link Everything
- Step 8: Validate and Show

## Progress Checklist

Copy this checklist and track your progress:

```
Bootstrap Progress:
- [ ] Step 1: Store initialized (`shapes init`)
- [ ] Step 2: Project explored (manifest, docs, structure, source, tests, CI)
- [ ] Step 3: Engineer interviewed (all rounds complete, understanding confirmed)
- [ ] Step 4: Profile defined (fields, kinds, lifecycle chosen and created)
- [ ] Step 5: Shapes created (root + children, rich descriptions, realizations)
- [ ] Step 6: Constraints created (specific rules, descriptions with "why")
- [ ] Step 7: Everything linked (reciprocal parent/child, constraint refs, realizations)
- [ ] Step 8: Validated clean (`shapes validate` exit code 0)
```

## Step 1: Initialize the Store

```bash
shapes init
```

This creates the `.shapes/` directory with `meta.yaml` and subdirectories for
shapes, constraints, amendments, and profiles.

## Step 2: Explore the Project

Read the project thoroughly before asking questions:

- **Project manifest** — `Cargo.toml`, `package.json`, `pyproject.toml`, etc.
- **README and docs** — understand purpose and architecture
- **Directory structure** — `ls` the top-level and key subdirectories
- **Key source files** — entry points, config, module definitions
- **Test structure** — reveals component boundaries
- **CI/CD config** — `.github/workflows/`, `Makefile`, build scripts

Build an initial mental model of what the project is, how it's structured,
and what patterns you can observe.

## Step 3: Interview the Engineer

This is the most important step. Code tells you *what* exists. Only the
engineer can tell you *why*.

**Autonomous mode:** If you are working autonomously (no interactive user,
CI/CD, benchmark, or headless context), skip the interview. Instead, infer
intent from the specification, README, code, and project structure. Document
your inferences in shape intent fields and mark `source: ai`. The engineer
can refine these later. Do not use AskUserQuestion when there is no human
to answer.

**Interactive mode:** Use the **AskUserQuestion** tool to ask questions
interactively in batches of 2-4 at a time. Wait for answers before asking
the next batch. Continue until you truly understand the project.

**Round 1 — Purpose and identity**:
- What problem does this project solve? Who uses it?
- What must this system get right above all else?
- What is deliberately out of scope?

**Round 2 — Architecture and trade-offs**:
- Why was this architecture chosen? What alternatives were considered?
- What are the key trade-offs?
- How does data flow through the system?

**Round 3 — Constraints and rules** (the most valuable round):
- What rules must every contributor follow but aren't documented?
- What mistakes shaped current conventions?
- What invariants must always hold? (security, performance, correctness)

**Round 4 — Domain knowledge**:
- What domain concepts does this model? Non-obvious business rules?
- What terminology needs explanation?

**Round 5 — History and evolution**:
- Most significant change the system has undergone?
- Known tech debt that shapes how new work is done?
- What would a new team member need to know on day one?

After each round, summarize and check understanding with AskUserQuestion.
Continue with additional rounds if answers reveal unexplored areas.

## Step 4: Define the Profile

Before creating shapes, decide with the engineer what fields matter for this
project. The protocol's Intent is an open map — a Profile declares which
domain-specific fields are required vs optional, what kinds are valid, and
how the lifecycle works.

Use **AskUserQuestion** with `multiSelect: true` to let the engineer pick
from sensible defaults, then ask if they want to add any custom fields.

### Shape Intent Fields

Start with the recommended defaults for the project's domain. Present these
as pre-selected and let the engineer remove what doesn't apply, then add
custom fields.

**Software projects (recommended defaults):**
- `goals` — what this shape must achieve
- `non_goals` — what is explicitly out of scope
- `rationale` — why this approach was chosen over alternatives
- `requirements` — specific functional requirements

**Software projects (optional, add if relevant):**
- `acceptance_criteria` — measurable conditions for completion
- `data_flow` — how data moves through this component
- `failure_modes` — what can go wrong and how it's mitigated
- `dependencies` — what this component depends on
- `api_contract` — the interface this component exposes

**Research projects (recommended defaults):**
- `hypotheses`, `success_criteria`, `methodology`, `variables`

**Editorial/writing projects (recommended defaults):**
- `themes`, `target_audience`, `tone`

After the engineer confirms, ask: "Any custom fields specific to your project
that aren't in this list?" Let them add domain-specific fields.

### Constraint Intent Fields

Similarly, present options for constraint intents:
- `rationale` — why this rule exists (the origin story)
- `impact_if_violated` — what breaks and how badly
- `exceptions` — known cases where the rule doesn't apply
- `verification_method` — how to check compliance

### Shape and Constraint Kinds

Present kind options and let the engineer select which are relevant:

**Shape kinds:**
- `system`, `service`, `feature`, `module`, `interface`, `data-flow`, `pattern`
- Plus any custom kinds the engineer suggests

**Constraint kinds:**
- `invariant`, `requirement`, `boundary`, `guideline`, `limit`, `policy`
- Plus any custom kinds

### Create the Profile

```bash
shapes create profile --name "<ProjectName> Profile" \
  --summary "Governance configuration for <ProjectName>"
```

Edit the Profile YAML to declare the selected fields, allowed kinds,
lifecycle gates, and amendment model. Use this structure:

```yaml
field_defs:
  intent:
    - name: goals
      description: "What this shape must achieve"
      required: true
    - name: non_goals
      description: "What is explicitly out of scope"
      required: false
    - name: rationale
      description: "Why this approach was chosen"
      required: true
allowed_kinds:
  shapes:
    - system
    - module
    - feature
    - interface
  constraints:
    - invariant
    - limit
    - guideline
    - requirement
lifecycle:
  gates:
    - from: proposed
      to: promoted
      preconditions:
        - "All required intent fields populated"
    - from: promoted
      to: canonical
      preconditions:
        - "Realization bindings present"
amendment_model: merge
```

Each gate requires `from` (source state), `to` (target state), and
`preconditions` (list of strings).

The Profile ensures consistency: `shapes validate` checks that all governed
nodes satisfy the Profile's field requirements.

## Step 5: Create Shapes

Create the top-level shape first, then decompose into children:

```bash
shapes create shape --name "<ProjectName>" --kind system \
  --summary "<description>"
```

Edit the YAML to flesh it out. Minimum viable shape structure:

```yaml
intent:
  kind: system
  summary: "<one-line description>"
  source: human
  goals: "<what this must achieve>"
  rationale: "<why this approach>"
profile: <profile-id>
status:
  state: proposed
realization:
  - bindings:
      - scheme: path
        value: src/main.rs
        metadata:
          summary: "<what's in this file relevant to this shape>"
    role: primary
```

Reference the Profile by ID in the `profile` field. Use whatever Intent
fields the Profile declares.

Then create child shapes for components, features, interfaces, patterns —
whatever decomposition captures the project's structure. Link them via
parent/child references in the YAML.

Shapes should be **rich and detailed**. The `description` should be a
paragraph, not a phrase. The `intent` should use the fields the Profile
defines. Include `realization` bindings pointing to actual source files.

Go deep — not just top-level modules, but interfaces, patterns, data flows,
and sub-features. The real value comes from shapes that capture what an
engineer carries in their head.

## Step 6: Create Constraints

Constraints capture rules that prevent bugs, security issues, and
architectural drift. They are the most valuable part for agents.

```bash
shapes create constraint --name "<Name>" --kind invariant \
  --rule "<specific, falsifiable rule>" --enforcement machine
```

Edit each constraint's YAML. Minimum viable constraint structure:

```yaml
intent:
  kind: invariant
  summary: "<one-line description>"
  source: human
  description: "<why this rule exists — the incident or requirement>"
  rule: "<specific, falsifiable invariant>"
  rationale: "<what breaks if violated>"
status:
  state: proposed
```

Each constraint should include:
- A `description` explaining why the rule exists — the incident, decision,
  or requirement that created it
- A `rule` specific enough to verify by reading code
- `realization` pointing to where the constraint is enforced
- `evidence` linking to tests or reviews that verify it

Constraint kinds: invariant, requirement, boundary, guideline, limit, policy.

## Step 7: Link Everything

Edit YAML files to establish relationships:

**Parent/child** (both sides must be set):
```yaml
# In child
parents:
  - id: 1
    role: component

# In parent
children:
  - shape: 2
    role: component
```

**Constraint references**:
```yaml
constraints:
  - 1
  - 2
```

**Realizations** (use `scheme: path` with `metadata.summary`):
```yaml
realization:
  - bindings:
      - scheme: path
        value: src/auth/mod.rs
        metadata:
          summary: OAuth2 login flow — token exchange, session creation, refresh logic
    role: primary
```

## Step 8: Validate and Show

```bash
shapes validate
```

If `shapes validate` reports errors (exit code 2), fix each issue and
re-validate. Repeat until exit code 0 — do not proceed with errors.

Once clean, show the final structure:

```bash
shapes tree shape
shapes tree constraint
```

Summarize what was created for the engineer — the key shapes, constraints,
and how they relate.
