//! `shapes create shape` — scaffolds a new Shape node, either from a
//! YAML file (`--from`) or from CLI flags using the active Profile.

use anyhow::{Result, anyhow};

use crate::OutputFormat;
use crate::commands::scaffold;
use crate::commands::shared::{read_from, report_created};
use crate::model::{NodeType, Shape, ShapeId};
use crate::store::{FileStore, NodeStore};
use crate::templates::KitKind;

use super::profile_helpers::{
    resolve_active_profile, shape_default_kind, validate_kind_against_profile,
};

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
    /// Optional `--profile` ID — overrides the store's active profile
    /// for this single create.
    pub profile: Option<u64>,
    /// Optional `--description`.
    pub description: Option<String>,
    /// Optional per-call `--kit`. Reserved for future use when
    /// re-scaffolding against a non-active kit; currently ignored
    /// because the Profile owns scaffold content.
    pub kit: Option<KitKind>,
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

    // The active profile (or caller-chosen override) is the sole
    // source of field hints, kind allow-lists, and default kinds.
    let profile = resolve_active_profile(store, args.profile)?;
    // `_kit` is accepted on the CLI for symmetry and future use; it
    // does not currently influence the scaffold because the Profile
    // owns every hint.
    let _ = args.kit;

    let kind_str = match args.kind {
        Some(k) => k,
        None => shape_default_kind(&profile)
            .ok_or_else(|| {
                anyhow!(
                    "active profile {} declares no default shape kind — pass --kind or edit the profile",
                    profile.id
                )
            })?
            .to_owned(),
    };

    validate_kind_against_profile(&profile, crate::model::NodeType::Shape, &kind_str)?;

    let yaml = scaffold::scaffold_shape(&scaffold::ShapeScaffold {
        id,
        name: &name,
        kind: &kind_str,
        summary: args.summary.as_deref(),
        source: &args.source,
        description: args.description.as_deref(),
        profile: &profile,
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
