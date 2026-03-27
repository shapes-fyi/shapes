# Shapes

**Record the intent, constraints, and decisions that shape a project.**

Shapes stores a queryable graph of your project's intent and constraints as plain YAML files in a `.shapes/` directory, version-controlled alongside your code. Agents query it before making changes; humans maintain it as the project evolves.

Read the full specification at **[shapes.fyi](https://shapes.fyi)**.

## Quick Start

### Install the CLI

```bash
cargo install --git https://github.com/shapes-fyi/shapes
```

### Install the Claude Code skills

```bash
npx skills add shapes-fyi/shapes
```

### Use it

In any project, run `/shapes:init` in Claude Code. The agent will:

1. Explore your project structure and source code
2. Interview you about architecture, constraints, and domain knowledge
3. Create a Profile defining what fields matter for your project
4. Generate a context graph of shapes and constraints
5. Validate the graph and show you the result

After initialization, the agent automatically discovers and uses shapes context before doing any work.

Run `/shapes:maintain` periodically to audit the graph for consistency, duplicates, coverage gaps, and stale realizations.

## CLI Reference

```bash
shapes init                              # Create .shapes/ directory
shapes create shape --name "Auth" --kind service --summary "JWT auth service"
shapes create constraint --name "No DB in Auth" --kind invariant --rule "..."
shapes get shape 1                       # Read a node's full intent
shapes list shape --kind feature         # List with filters
shapes tree shape                        # ASCII DAG visualization
shapes query constraints 2              # All constraints (with inheritance)
shapes validate                          # Check graph integrity
```

Run `shapes --help` for the full command reference. Output formats: `--format yaml` (default) or `--format json`.

## Repository Structure

This is a monorepo containing:

- **`src/`** — `shapes-cli`, a Rust CLI to create and query the context graph
- **`apps/web/`** — the [shapes.fyi](https://shapes.fyi) specification website (TanStack Start, React 19, Vite)
- **`packages/ui/`** — shared UI component library (shadcn/ui, Base UI, Tailwind CSS v4)
- **`skills/`** — Claude Code skills (`/shapes:init`, `/shapes:context`, `/shapes:maintain`)
- **`.shapes/`** — the project's own context graph (shapes eating its own dog food)
