//! Crate root for the `shapes` CLI binary.
//!
//! Wires together the four top-level modules ([`commands`], [`error`],
//! [`model`], [`store`], [`templates`]), defines the [`clap`] parser, and
//! dispatches each subcommand to its handler in [`commands`]. The crate
//! also pins the project-wide lint baseline below — see constraint 24
//! (Crate-Root Lint Discipline) in `.shapes/`.

#![deny(unsafe_code)]
#![warn(rust_2018_idioms)]
#![deny(missing_docs)]

mod commands;
mod error;
mod model;
mod store;
mod templates;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use error::CliError;

use model::{ConstraintId, Enforcement, NodeType, ProfileId, ShapeId, VersionImpact};
use templates::KitKind;

#[derive(Parser)]
#[command(
    name = "shapes",
    version,
    about = "Shapes Specification CLI — create, query, and navigate project intent, constraints, and boundaries",
    long_about = "Shapes captures the intent, constraints, and boundaries of a project as a structured,\n\
                  queryable graph (DAG). Four node types: Shape (what to build), Constraint (rules that\n\
                  must hold), Amendment (change records), Profile (governance). Use this CLI to create\n\
                  nodes, explore the graph, and validate integrity.\n\n\
                  Storage: .shapes/ directory with YAML files. Run `shapes init` to get started.\n\
                  Exit codes: 0 = success, 1 = error, 2 = validation failures."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output format (yaml or json)
    #[arg(long, default_value = "yaml", global = true)]
    format: OutputFormat,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Yaml,
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new .shapes/ directory in the current working directory.
    /// Creates meta.yaml, subdirectories for shapes / constraints / amendments / profiles,
    /// and a starter Profile (id 1) seeded from the chosen kit. The seeded Profile becomes
    /// the store's active profile — every subsequent `shapes create` reads from it.
    Init {
        /// Starter kit for the seeded Profile — software (default), research, editorial, or minimal
        #[arg(long, value_enum, default_value = "software")]
        kit: KitKind,
    },

    /// Create a new node from flags or a YAML file.
    /// The node gets an auto-assigned ID and starts in 'proposed' status.
    /// Prints the full node to stdout and the file path to stderr.
    Create {
        #[command(subcommand)]
        node: CreateCommand,

        /// Output only the assigned numeric ID (useful for scripting)
        #[arg(long, global = true)]
        id_only: bool,
    },

    /// Read and display a single node by type and ID.
    /// Use this to see full intent, constraints, realizations, and metadata.
    Get {
        /// Node type to retrieve
        #[arg(value_enum)]
        node_type: NodeType,
        /// Numeric node ID
        id: u64,
    },

    /// List nodes with optional filters.
    /// Shows a summary table: type, id, name, status, and kind for each node.
    /// Without arguments, lists all nodes across all types.
    List {
        /// Filter to a specific node type
        #[arg(value_enum)]
        node_type: Option<NodeType>,

        /// Filter by status name (proposed, promoted, canonical, rejected, superseded, abandoned, reverted)
        #[arg(long)]
        status: Option<String>,

        /// Filter by intent kind (system, service, feature, module, interface, pattern, etc.)
        #[arg(long)]
        kind: Option<String>,
    },

    /// Display the DAG as an ASCII tree.
    /// Shows parent-child hierarchy with status and kind for each node.
    /// Shape trees also show constraint references inline.
    /// Defaults to showing the Shape composition graph when no node type is given.
    /// Use this to understand the overall structure before diving into details.
    Tree {
        /// Which node type to display: 'shape' (default) or 'constraint'
        #[arg(value_enum, default_value = "shape")]
        node_type: DagType,

        /// Show only the subtree rooted at this node ID (default: show all roots)
        #[arg(long)]
        root: Option<u64>,

        /// Maximum depth to display (default: 10)
        #[arg(long, default_value = "10")]
        depth: usize,
    },

    /// Query DAG relationships — ancestors, descendants, or effective constraints.
    /// Use 'ancestors' and 'descendants' to navigate the Shape or Constraint DAG.
    /// Use 'constraints' to see all constraints that apply to a shape, including
    /// those inherited from parent shapes.
    Query {
        #[command(subcommand)]
        operation: QueryCommand,
    },

    /// Check both DAGs for integrity issues.
    /// Detects cycles, dangling references, missing reciprocal links,
    /// empty amendment targets, and profile field requirement violations.
    /// Exit code 0 if clean, 2 if issues found.
    Validate,

    /// Run PR-level shape-graph checks against a base ref.
    ///
    /// Compares the working tree to `--base` and reports CI-* issues
    /// for missing-amendment-on-promoted-or-canonical-change (CI-002),
    /// modified-amendment-immutability (CI-003), and optionally
    /// no-shapes-changes (CI-001 when `--require-shapes-changes` is
    /// passed). Exit 0 if clean, 2 if issues found.
    ///
    /// CI-002 is strict: every field on a promoted or canonical node
    /// counts, including `realization`, `evidence`, and `provenance`
    /// bindings. There are no opt-out flags — the whole point of the
    /// check is to force explicit maintenance through the amendment
    /// workflow. To edit a node without an amendment, leave it in
    /// `proposed` state.
    CiCheck {
        /// Base ref to diff against (e.g. origin/main, the PR base sha)
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// Path to the shapes directory, relative to cwd
        #[arg(long, default_value = ".shapes")]
        shapes_dir: PathBuf,
        /// Fail when the PR does not touch the shapes directory
        #[arg(long)]
        require_shapes_changes: bool,
    },
}

#[derive(Subcommand)]
enum CreateCommand {
    /// Create a Shape — captures what is being built and why.
    /// Shapes are the primary nodes in the Shape DAG.
    /// Without `--from`, emits a YAML scaffold with `TODO:` placeholders for every
    /// expected field, including commented stub sections for parents/children/
    /// constraints/realization. Read the file and fill in the TODOs.
    Shape {
        /// Shape name
        #[arg(long, required_unless_present = "from")]
        name: Option<String>,
        /// Intent kind. Defaults to the active profile's default shape kind.
        /// Common kinds: system, service, feature, module, interface, data-flow, pattern
        #[arg(long)]
        kind: Option<String>,
        /// Brief summary of the shape's purpose
        #[arg(long)]
        summary: Option<String>,
        /// Origin: human, ai, or system
        #[arg(long, default_value = "human")]
        source: String,
        /// Apply a governance profile by ID — sets profile reference and pre-populates required fields
        #[arg(long, conflicts_with = "from")]
        profile: Option<u64>,
        /// Full description (defaults to a TODO placeholder if omitted)
        #[arg(long)]
        description: Option<String>,
        /// Override the active profile's seeding kit for this scaffold only (does not modify meta.yaml)
        #[arg(long, value_enum, conflicts_with = "from")]
        kit: Option<KitKind>,
        /// Read full YAML definition from file (use - for stdin). Mutually exclusive with other flags.
        #[arg(long, conflicts_with_all = &["name", "kind", "summary", "source", "description", "kit"])]
        from: Option<String>,
    },

    /// Create a Constraint — a rule or invariant that must be satisfied.
    /// Constraints form their own DAG and are referenced by shapes.
    /// Without `--from`, emits a YAML scaffold with `TODO:` placeholders.
    /// `--enforcement` accepts only `manual` (human review) or `machine` (automated check).
    Constraint {
        /// Constraint name
        #[arg(long, required_unless_present = "from")]
        name: Option<String>,
        /// Constraint kind. Defaults to the active profile's default constraint kind.
        /// Common kinds: invariant, requirement, boundary, guideline, limit, policy
        #[arg(long)]
        kind: Option<String>,
        /// The rule text — a specific, falsifiable statement of what must hold
        #[arg(long)]
        rule: Option<String>,
        /// How enforced: `manual` (human review) or `machine` (automated checks)
        #[arg(long, default_value = "manual")]
        enforcement: Enforcement,
        /// Brief summary
        #[arg(long)]
        summary: Option<String>,
        /// Origin: human, ai, or system
        #[arg(long, default_value = "human")]
        source: String,
        /// Intent kind — classifies the purpose of this constraint's intent (defaults to --kind value)
        #[arg(long)]
        intent_kind: Option<String>,
        /// Apply a governance profile by ID — sets profile reference and pre-populates required fields
        #[arg(long, conflicts_with = "from")]
        profile: Option<u64>,
        /// Full description (defaults to a TODO placeholder if omitted)
        #[arg(long)]
        description: Option<String>,
        /// Override the active profile's seeding kit for this scaffold only (does not modify meta.yaml)
        #[arg(long, value_enum, conflicts_with = "from")]
        kit: Option<KitKind>,
        /// Read full YAML definition from file (use - for stdin)
        #[arg(long, conflicts_with_all = &["name", "kind", "rule", "enforcement", "summary", "source", "intent_kind", "description", "kit"])]
        from: Option<String>,
    },

    /// Create an Amendment — an immutable change record targeting shapes, constraints, or profiles.
    /// Use amendments to record significant changes to canonical nodes.
    Amendment {
        /// Amendment name
        #[arg(long, required_unless_present = "from")]
        name: Option<String>,
        /// Target shape IDs (repeatable: --target-shape 1 --target-shape 2)
        #[arg(long = "target-shape")]
        target_shapes: Vec<ShapeId>,
        /// Target constraint IDs (repeatable)
        #[arg(long = "target-constraint")]
        target_constraints: Vec<ConstraintId>,
        /// Target profile IDs (repeatable)
        #[arg(long = "target-profile")]
        target_profiles: Vec<ProfileId>,
        /// Brief summary of what changed and why
        #[arg(long)]
        summary: Option<String>,
        /// Origin: human, ai, or system
        #[arg(long, default_value = "human")]
        source: String,
        /// Semantic version impact: major, minor, or patch
        #[arg(long)]
        version_impact: Option<VersionImpact>,
        /// Full description (defaults to name if omitted)
        #[arg(long)]
        description: Option<String>,
        /// Read full YAML definition from file (use - for stdin)
        #[arg(long, conflicts_with_all = &["name", "target_shapes", "target_constraints", "target_profiles", "summary", "source", "version_impact", "description"])]
        from: Option<String>,
    },

    /// Create a new Profile — governance configuration defining field
    /// requirements, kind allow-lists, and amendment rules.
    /// Without `--from`, seeds the new Profile from a starter kit (defaults
    /// to `software`). The seeded Profile is a complete, canonical node;
    /// edit its YAML after creation to customize further.
    Profile {
        /// Profile name
        #[arg(long, required_unless_present = "from")]
        name: Option<String>,
        /// Starter kit used to seed this Profile — software (default),
        /// research, editorial, or minimal
        #[arg(long, value_enum, conflicts_with = "from")]
        kit: Option<KitKind>,
        /// Read full YAML definition from file (use - for stdin)
        #[arg(long, conflicts_with_all = &["name", "kit"])]
        from: Option<String>,
    },
}

/// Selects which DAG (`shape` or `constraint`) a query operates on.
#[derive(Clone, Copy, ValueEnum)]
pub enum DagType {
    /// The shape DAG.
    Shape,
    /// The constraint DAG.
    Constraint,
}

#[derive(Subcommand)]
enum QueryCommand {
    /// Walk up the parent chain of a node.
    /// Returns the IDs of all ancestors in the Shape or Constraint DAG.
    Ancestors {
        /// Which DAG to traverse
        #[arg(value_enum)]
        node_type: DagType,
        /// Starting node ID
        id: u64,
    },
    /// Walk down the child tree of a node.
    /// Returns the IDs of all descendants in the Shape or Constraint DAG.
    Descendants {
        /// Which DAG to traverse
        #[arg(value_enum)]
        node_type: DagType,
        /// Starting node ID
        id: u64,
    },
    /// Get all effective constraints for a shape, including those inherited from ancestors.
    /// For each constraint, shows: constraint_id, name, which shape it came from, and whether inherited.
    /// This is the key command for understanding what rules apply before writing code.
    Constraints {
        /// Shape ID to query constraints for
        shape_id: u64,
    },
    /// Find all shapes that are governed by a constraint — both direct references and inherited.
    /// This is the reverse of `query constraints`: given a constraint, which shapes must satisfy it?
    ShapesForConstraint {
        /// Constraint ID to look up
        constraint_id: u64,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            e.exit_code()
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Init { kit } => commands::init(kit)?,

        Command::Create { node, id_only } => commands::create(node, id_only, cli.format)?,

        Command::Get { node_type, id } => commands::get(node_type, id, cli.format)?,

        Command::List {
            node_type,
            status,
            kind,
        } => commands::list(node_type, status, kind, cli.format)?,

        Command::Tree {
            node_type,
            root,
            depth,
        } => commands::tree(node_type, root, depth)?,

        Command::Query { operation } => commands::query(operation, cli.format)?,

        Command::Validate => commands::validate(cli.format)?,
        Command::CiCheck {
            base,
            shapes_dir,
            require_shapes_changes,
        } => commands::ci_check(&base, &shapes_dir, require_shapes_changes, cli.format)?,
    }

    Ok(())
}
