//! `shapes create amendment` — records a new Amendment node.

use std::collections::BTreeMap;

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::shared::{output, read_from};
use crate::model::{
    Amendment, AmendmentId, AmendmentTargets, Archived, ConstraintId, InitiatedBy, InitiatedType,
    Intent, NodeType, ProfileId, ShapeId, Status, VersionImpact,
};
use crate::store::{FileStore, NodeStore};

/// Field bag for `shapes create amendment`.
pub struct CreateAmendmentArgs {
    /// `--name` value.
    pub name: Option<String>,
    /// `--target-shape` IDs.
    pub target_shapes: Vec<ShapeId>,
    /// `--target-constraint` IDs.
    pub target_constraints: Vec<ConstraintId>,
    /// `--target-profile` IDs.
    pub target_profiles: Vec<ProfileId>,
    /// Optional `--summary`.
    pub summary: Option<String>,
    /// `--source` value.
    pub source: String,
    /// Optional `--version-impact`.
    pub version_impact: Option<VersionImpact>,
    /// Optional `--description`.
    pub description: Option<String>,
    /// Optional `--from` path or `-` for stdin.
    pub from: Option<String>,
}

/// Creates a new Amendment node and writes it to the store.
pub fn create_amendment(
    store: &FileStore,
    args: CreateAmendmentArgs,
    id_only: bool,
    format: OutputFormat,
) -> Result<()> {
    let id = AmendmentId::new(store.next_id(NodeType::Amendment)?);
    let amendment: Amendment = if let Some(path) = args.from {
        let content = read_from(&path)?;
        let mut a: Amendment = serde_yaml_ng::from_str(&content)?;
        a.id = id;
        a
    } else {
        // clap requires --name when --from is absent.
        let name = args
            .name
            .expect("clap requires --name when --from is absent");
        Amendment {
            id,
            name: name.clone(),
            description: args.description.unwrap_or_else(|| name.clone()),
            targets: AmendmentTargets {
                shape_ids: args.target_shapes,
                constraint_ids: args.target_constraints,
                profile_ids: args.target_profiles,
            },
            status: Status::proposed(),
            version_impact: args.version_impact,
            intent: Intent {
                kind: "amendment".into(),
                summary: args.summary.unwrap_or(name),
                source: serde_yaml_ng::Value::String(args.source),
                uris: vec![],
                extra: BTreeMap::default(),
            },
            constraints: vec![],
            realization: vec![],
            evidence: vec![],
            provenance: vec![],
            initiated_by: InitiatedBy {
                initiated_type: InitiatedType::Human,
                identity: None,
                provenance: None,
            },
            archived: Archived::No,
            metadata: BTreeMap::new(),
        }
    };
    let path = store.save(NodeType::Amendment, id.get(), &amendment)?;
    if id_only {
        println!("{id}");
    } else {
        eprintln!("Created {}", path.display());
        output(&amendment, format)?;
    }
    Ok(())
}
