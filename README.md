# Shapes

Shapes Context Protocol — structured intent, constraints, and boundaries for projects.

## What It Does

Shapes captures the **meaning behind the code** — the intent, constraints, and decisions that live in engineers' heads but not in the codebase.

## Repository Structure

This is a monorepo containing:

- **`src/`** — `shapes-cli`, a Rust CLI to create and query a structured graph (DAG) of project intent, constraints, and boundaries
- **`apps/web/`** — the [shapes.fyi](https://shapes.fyi) website, built with TanStack Start (React 19), Vite, and Nitro
- **`packages/ui/`** — shared UI component library using shadcn/ui, Base UI, and Tailwind CSS v4
- **`skills/`** — Claude Code skills (`/shapes:init`, `/shapes:context`, `/shapes:maintain`)
- **`.shapes/`** — the project's own context graph (shapes eating its own dog food)

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

## Development

### CLI

```bash
cargo build                # Build the CLI
cargo run -- --help        # Run locally
```

### Web

```bash
bun install               # Install dependencies
bun run dev               # Start dev server (port 3000)
bun run build             # Production build
bun run typecheck         # Type check all packages
bun run lint              # Lint all packages
bun run format            # Format with Prettier
```

## How It Works

Shapes creates a `.shapes/` directory in your project containing YAML files that describe:

- **Shapes** — what is being built and why
- **Constraints** — rules that must be satisfied
- **Amendments** — immutable records of changes
- **Profiles** — governance configurations defining lifecycle gates and field requirements

These form two Directed Acyclic Graphs (DAGs) — one for shape composition, one for constraint composition. Agents traverse these graphs to discover what constraints apply to any piece of work.

## CLI Usage

```bash
shapes init                              # Create .shapes/ directory
shapes create shape --name "Auth" --kind service --summary "JWT auth service"
shapes create constraint --name "No DB in Auth" --kind invariant --rule "..."
shapes get shape 1                       # Read a node
shapes list shape --kind feature         # List with filters
shapes tree shape                        # ASCII DAG visualization
shapes query constraints 2              # Effective constraints (with inheritance)
shapes validate                          # Check integrity
```

Run `shapes --help` for the full command reference.

## Learn More

- [Shapes Specification](https://shapes.fyi) — the Open Context Protocol specification
