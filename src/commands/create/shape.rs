//! `shapes create shape` — scaffolds a new Shape node, either from a
//! YAML file (`--from`) or from CLI flags using the active template.

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::scaffold;
use crate::commands::shared::{read_from, report_created};
use crate::model::{NodeType, Shape, ShapeId};
use crate::store::{FileStore, NodeStore};
use crate::templates::TemplateKind;

use super::profile_helpers::{load_profile, resolve_template, validate_kind_against_profile};

/// Field bag for `shapes create shape`.
pub struct CreateShapeArgs {
    /// `--name` value (required when `--from` is absent; clap enforces).
    pub name: Option<String>,
    /// Optional `--kind` override.
    pub kind: Option<String>,
    /// Optional `--summary`.
    pub summary: Option<String>,
    /// `--source` value.
    pub source: String,
    /// Optional `--profile` ID.
    pub profile: Option<u64>,
    /// Optional `--description`.
    pub description: Option<String>,
    /// Optional per-call `--template`.
    pub template: Option<TemplateKind>,
    /// Optional `--from` path or `-` for stdin.
    pub from: Option<String>,
}

/// Creates a new Shape node and writes it to the store.
pub fn create_shape(
    store: &FileStore,
    args: CreateShapeArgs,
    id_only: bool,
    format: OutputFormat,
) -> Result<()> {
    let id = ShapeId::new(store.next_id(NodeType::Shape)?);

    if let Some(path) = args.from {
        let content = read_from(&path)?;
        let mut s: Shape = serde_yml::from_str(&content)?;
        s.id = id;
        let saved_path = store.save(NodeType::Shape, id.get(), &s)?;
        report_created(id_only, &id.to_string(), &saved_path, &s, format)?;
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
        .unwrap_or_else(|| template.default_shape_kind.to_owned());

    if let Some(pid) = args.profile {
        let p = load_profile(store, pid)?;
        validate_kind_against_profile(&p, "shape", &kind_str)?;
    }

    let yaml = scaffold::scaffold_shape(&scaffold::ShapeScaffold {
        id,
        name: &name,
        kind: &kind_str,
        summary: args.summary.as_deref(),
        source: &args.source,
        description: args.description.as_deref(),
        profile: args.profile,
        template,
    });
    let saved_path = store.save_raw(NodeType::Shape, id.get(), &name, &yaml)?;
    if id_only {
        println!("{id}");
    } else {
        eprintln!("Created {}", saved_path.display());
        print!("{yaml}");
    }
    Ok(())
}
