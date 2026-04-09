//! `shapes list` — lists nodes across one or more node types with
//! optional `--status`, `--kind`, and `--archived` filters. Archived
//! amendments are hidden by default; pass `--archived` to include them.

use anyhow::Result;
use serde::Serialize;

use crate::OutputFormat;
use crate::commands::shared::{open_store, output};
use crate::model::{Amendment, Constraint, NodeType, Profile, Shape};
use crate::store::NodeStore;

/// One row in the `list` output.
#[derive(Serialize)]
struct ListEntry {
    #[serde(rename = "type")]
    node_type: String,
    id: u64,
    name: String,
    status: String,
    kind: String,
}

/// Lists nodes filtered by type, status, kind, and archived state.
/// When `include_archived` is `false` (the default), amendments whose
/// `archived` flag is true are skipped — they stay on disk for audit
/// but do not clutter routine listings.
pub fn list(
    node_type: Option<NodeType>,
    status_filter: Option<String>,
    kind_filter: Option<String>,
    include_archived: bool,
    format: OutputFormat,
) -> Result<()> {
    let store = open_store()?;
    let types = match node_type {
        Some(t) => vec![t],
        None => vec![
            NodeType::Shape,
            NodeType::Constraint,
            NodeType::Amendment,
            NodeType::Profile,
        ],
    };

    let mut entries = Vec::new();

    for t in types {
        let ids = store.list_ids(t)?;
        for id in ids {
            let (name, status, kind) = match t {
                NodeType::Shape => {
                    let s: Shape = store.load(t, id)?;
                    (s.name, s.status.name().to_owned(), s.intent.kind)
                }
                NodeType::Constraint => {
                    let c: Constraint = store.load(t, id)?;
                    (c.name, c.status.name().to_owned(), c.kind)
                }
                NodeType::Amendment => {
                    let a: Amendment = store.load(t, id)?;
                    if a.is_archived() && !include_archived {
                        continue;
                    }
                    (a.name, a.status.name().to_owned(), a.intent.kind)
                }
                NodeType::Profile => {
                    let p: Profile = store.load(t, id)?;
                    (p.name, p.status.name().to_owned(), p.intent.kind)
                }
            };

            if let Some(ref sf) = status_filter
                && &status != sf
            {
                continue;
            }
            if let Some(ref kf) = kind_filter
                && &kind != kf
            {
                continue;
            }

            entries.push(ListEntry {
                node_type: t.to_string(),
                id,
                name,
                status,
                kind,
            });
        }
    }

    output(&entries, format)
}
