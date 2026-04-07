//! `shapes create constraint` — scaffolds a new Constraint node.

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::scaffold;
use crate::commands::shared::{read_from, report_created};
use crate::model::{Constraint, ConstraintId, Enforcement, NodeType};
use crate::store::{FileStore, NodeStore};
use crate::templates::TemplateKind;

use super::profile_helpers::{load_profile, resolve_template, validate_kind_against_profile};

/// Field bag for `shapes create constraint`.
pub struct CreateConstraintArgs {
    /// `--name` value (required when `--from` is absent).
    pub name: Option<String>,
    /// Optional `--kind` override.
    pub kind: Option<String>,
    /// Optional `--rule` body.
    pub rule: Option<String>,
    /// `--enforcement` value.
    pub enforcement: Enforcement,
    /// Optional `--summary`.
    pub summary: Option<String>,
    /// `--source` value.
    pub source: String,
    /// Optional `--intent-kind` override.
    pub intent_kind: Option<String>,
    /// Optional `--profile` ID.
    pub profile: Option<u64>,
    /// Optional `--description`.
    pub description: Option<String>,
    /// Optional per-call `--template`.
    pub template: Option<TemplateKind>,
    /// Optional `--from` path or `-` for stdin.
    pub from: Option<String>,
}

/// Creates a new Constraint node and writes it to the store.
pub fn create_constraint(
    store: &FileStore,
    args: CreateConstraintArgs,
    id_only: bool,
    format: OutputFormat,
) -> Result<()> {
    let id = ConstraintId::new(store.next_id(NodeType::Constraint)?);

    if let Some(path) = args.from {
        let content = read_from(&path)?;
        let mut c: Constraint = serde_yml::from_str(&content)?;
        c.id = id;
        let saved_path = store.save(NodeType::Constraint, id.get(), &c)?;
        report_created(id_only, &id.to_string(), &saved_path, &c, format)?;
        return Ok(());
    }

    // clap requires --name when --from is absent, so this expect()
    // documents the invariant rather than assuming caller goodwill.
    let name = args
        .name
        .expect("clap requires --name when --from is absent");
    let template = resolve_template(store, args.template);
    let kind_str = args
        .kind
        .unwrap_or_else(|| template.default_constraint_kind.to_owned());

    if let Some(pid) = args.profile {
        let p = load_profile(store, pid)?;
        validate_kind_against_profile(&p, "constraint", &kind_str)?;
    }

    let yaml = scaffold::scaffold_constraint(&scaffold::ConstraintScaffold {
        id: id.get(),
        name: &name,
        kind: &kind_str,
        rule: args.rule.as_deref(),
        enforcement: args.enforcement,
        summary: args.summary.as_deref(),
        source: &args.source,
        description: args.description.as_deref(),
        intent_kind: args.intent_kind.as_deref(),
        profile: args.profile,
        template,
    });
    let saved_path = store.save_raw(NodeType::Constraint, id.get(), &name, &yaml)?;
    if id_only {
        println!("{id}");
    } else {
        eprintln!("Created {}", saved_path.display());
        print!("{yaml}");
    }
    Ok(())
}
