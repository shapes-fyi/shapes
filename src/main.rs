mod commands;
mod model;
mod store;

use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use model::{AmendmentModel, ConstraintId, Enforcement, NodeType, ShapeId, VersionImpact};

#[derive(Parser)]
#[command(
    name = "shapes",
    version,
    about = "Shapes Context Protocol CLI — create, query, and navigate project intent, constraints, and boundaries",
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
    /// Creates meta.yaml and subdirectories for shapes, constraints, amendments, and profiles.
    Init,

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
}

#[derive(Subcommand)]
enum CreateCommand {
    /// Create a Shape — captures what is being built and why.
    /// Shapes are the primary nodes in the Shape DAG.
    /// Kinds: system, service, feature, module, interface, data-flow, pattern.
    /// Edit the generated YAML to add children, constraints, realizations, and rich intent.
    Shape {
        /// Shape name
        #[arg(long)]
        name: String,
        /// Intent kind (default: feature). Common kinds: system, service, feature, module, interface, data-flow, pattern
        #[arg(long, default_value = "feature")]
        kind: String,
        /// Brief summary of the shape's purpose
        #[arg(long)]
        summary: Option<String>,
        /// Origin: human, ai, or system
        #[arg(long, default_value = "human")]
        source: String,
        /// Full description (defaults to name if omitted)
        #[arg(long)]
        description: Option<String>,
        /// Read full YAML definition from file (use - for stdin). Mutually exclusive with other flags.
        #[arg(long, conflicts_with_all = &["name", "kind", "summary", "source", "description"])]
        from: Option<String>,
    },

    /// Create a Constraint — a rule or invariant that must be satisfied.
    /// Constraints form their own DAG and are referenced by shapes.
    /// Kinds: invariant, requirement, boundary, guideline, limit, policy.
    /// The 'rule' field should be specific enough to verify by reading code.
    Constraint {
        /// Constraint name
        #[arg(long)]
        name: String,
        /// Constraint kind (default: requirement). Kinds: invariant, requirement, boundary, guideline, limit, policy
        #[arg(long, default_value = "requirement")]
        kind: String,
        /// The rule text — a specific, falsifiable statement of what must hold
        #[arg(long)]
        rule: Option<String>,
        /// How enforced: manual (human review) or machine (automated checks)
        #[arg(long, default_value = "manual")]
        enforcement: Enforcement,
        /// Brief summary
        #[arg(long)]
        summary: Option<String>,
        /// Origin: human, ai, or system
        #[arg(long, default_value = "human")]
        source: String,
        /// Full description (defaults to name if omitted)
        #[arg(long)]
        description: Option<String>,
        /// Read full YAML definition from file (use - for stdin)
        #[arg(long, conflicts_with_all = &["name", "kind", "rule", "enforcement", "summary", "source", "description"])]
        from: Option<String>,
    },

    /// Create an Amendment — an immutable change record targeting shapes, constraints, or profiles.
    /// Use amendments to record significant changes to canonical nodes.
    Amendment {
        /// Amendment name
        #[arg(long)]
        name: String,
        /// Target shape IDs (repeatable: --target-shape 1 --target-shape 2)
        #[arg(long = "target-shape")]
        target_shapes: Vec<ShapeId>,
        /// Target constraint IDs (repeatable)
        #[arg(long = "target-constraint")]
        target_constraints: Vec<ConstraintId>,
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
        #[arg(long, conflicts_with_all = &["name", "target_shapes", "target_constraints", "summary", "source", "version_impact", "description"])]
        from: Option<String>,
    },

    /// Create a Profile — governance configuration defining lifecycle gates, field requirements, and amendment rules.
    Profile {
        /// Profile name
        #[arg(long)]
        name: String,
        /// Brief summary
        #[arg(long)]
        summary: Option<String>,
        /// Origin: human, ai, or system
        #[arg(long, default_value = "human")]
        source: String,
        /// How amendments are applied: merge, overlay, edition, or append-only
        #[arg(long, default_value = "merge")]
        amendment_model: AmendmentModel,
        /// Full description (defaults to name if omitted)
        #[arg(long)]
        description: Option<String>,
        /// Read full YAML definition from file (use - for stdin)
        #[arg(long, conflicts_with_all = &["name", "summary", "source", "amendment_model", "description"])]
        from: Option<String>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum DagType {
    Shape,
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
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init => commands::init(),

        Command::Create { node, id_only } => commands::create(node, id_only, cli.format),

        Command::Get { node_type, id } => commands::get(node_type, id, cli.format),

        Command::List {
            node_type,
            status,
            kind,
        } => commands::list(node_type, status, kind, cli.format),

        Command::Tree {
            node_type,
            root,
            depth,
        } => commands::tree(node_type, root, depth),

        Command::Query { operation } => commands::query(operation, cli.format),

        Command::Validate => commands::validate(cli.format),
    }
}
