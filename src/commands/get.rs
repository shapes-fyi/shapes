//! `shapes get` — loads a single node by type and ID and emits it in
//! the requested output format. When the node carries an
//! `amendment_log`, archived amendments are hidden by default and only
//! surfaced (with an `archived: true` annotation) when `--archived` is
//! passed.

use std::collections::BTreeSet;

use anyhow::Result;
use serde::Serialize;

use crate::OutputFormat;
use crate::commands::shared::{open_store, output};
use crate::model::{Amendment, AmendmentId, GraphNode, NodeType};
use crate::store::{FileStore, NodeStore};

/// Loads a single node and prints it. When `archived` is `false` (the
/// default), archived amendments are filtered out of the rendered
/// `amendment_log` on Shape/Constraint/Profile nodes. When `true`, the
/// full log is rendered with archived entries annotated so readers
/// know to defer reading them.
pub fn get(node_type: NodeType, id: u64, archived: bool, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    use crate::model::ids::{AmendmentId, ConstraintId, ProfileId, ShapeId};
    match node_type {
        NodeType::Shape => {
            let node = store.load_shape(ShapeId::new(id))?;
            emit_with_amendment_log(&node, node.amendment_log(), &store, archived, format)
        }
        NodeType::Constraint => {
            let node = store.load_constraint(ConstraintId::new(id))?;
            emit_with_amendment_log(&node, node.amendment_log(), &store, archived, format)
        }
        NodeType::Profile => {
            let node = store.load_profile(ProfileId::new(id))?;
            emit_with_amendment_log(&node, node.amendment_log(), &store, archived, format)
        }
        NodeType::Amendment => {
            // Direct fetch by id is an explicit request: always return
            // the full amendment including its `archived` field.
            output(&store.load_amendment(AmendmentId::new(id))?, format)
        }
    }
}

/// Serializes `node` to the requested format and rewrites its
/// `amendment_log` field to reflect the `--archived` flag:
/// - `show_archived == false`: archived IDs are dropped from the list.
/// - `show_archived == true`: the list becomes a sequence of
///   `{id, archived?: true}` mappings so archived entries are visually
///   flagged.
fn emit_with_amendment_log<T: Serialize>(
    node: &T,
    amendment_log: &[AmendmentId],
    store: &FileStore,
    show_archived: bool,
    format: OutputFormat,
) -> Result<()> {
    let archived_ids = collect_archived_ids(store, amendment_log);

    // Always go through the patching path when an amendment_log exists
    // so that serialization format (key ordering) is consistent
    // regardless of whether any amendment happens to be archived.
    match format {
        OutputFormat::Yaml => {
            let mut value = serde_yml::to_value(node)?;
            patch_yaml_amendment_log(&mut value, &archived_ids, show_archived);
            print!("{}", serde_yml::to_string(&value)?);
            Ok(())
        }
        OutputFormat::Json => {
            let mut value = serde_json::to_value(node)?;
            patch_json_amendment_log(&mut value, &archived_ids, show_archived);
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }
    }
}

/// Loads each referenced amendment and returns the set of IDs that
/// carry `archived: true`. Amendments that fail to load are treated as
/// not-archived — `shapes validate` is the source of truth for dangling
/// references; `get` must not refuse to render a node just because one
/// amendment file is missing.
fn collect_archived_ids(store: &FileStore, amendment_log: &[AmendmentId]) -> BTreeSet<u64> {
    amendment_log
        .iter()
        .filter_map(|id| {
            let a: Amendment = store.load(NodeType::Amendment, id.get()).ok()?;
            if a.is_archived() {
                Some(id.get())
            } else {
                None
            }
        })
        .collect()
}

fn patch_yaml_amendment_log(
    value: &mut serde_yml::Value,
    archived_ids: &BTreeSet<u64>,
    show_archived: bool,
) {
    let Some(mapping) = value.as_mapping_mut() else {
        return;
    };
    let key = serde_yml::Value::String("amendment_log".into());
    let Some(log) = mapping.get_mut(&key) else {
        return;
    };
    let Some(seq) = log.as_sequence_mut() else {
        return;
    };
    if show_archived {
        for entry in seq.iter_mut() {
            let Some(id) = entry.as_u64() else { continue };
            if archived_ids.contains(&id) {
                let mut map = serde_yml::Mapping::new();
                map.insert("id".into(), serde_yml::Value::Number(id.into()));
                map.insert("archived".into(), serde_yml::Value::Bool(true));
                *entry = serde_yml::Value::Mapping(map);
            } else {
                let mut map = serde_yml::Mapping::new();
                map.insert("id".into(), serde_yml::Value::Number(id.into()));
                *entry = serde_yml::Value::Mapping(map);
            }
        }
    } else {
        seq.retain(|entry| match entry.as_u64() {
            Some(id) => !archived_ids.contains(&id),
            None => true,
        });
        if seq.is_empty() {
            mapping.remove(&key);
        }
    }
}

fn patch_json_amendment_log(
    value: &mut serde_json::Value,
    archived_ids: &BTreeSet<u64>,
    show_archived: bool,
) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    let Some(log) = obj.get_mut("amendment_log") else {
        return;
    };
    let Some(arr) = log.as_array_mut() else {
        return;
    };
    if show_archived {
        for entry in arr.iter_mut() {
            let Some(id) = entry.as_u64() else { continue };
            let mut map = serde_json::Map::new();
            map.insert("id".into(), serde_json::Value::from(id));
            if archived_ids.contains(&id) {
                map.insert("archived".into(), serde_json::Value::Bool(true));
            }
            *entry = serde_json::Value::Object(map);
        }
    } else {
        arr.retain(|entry| match entry.as_u64() {
            Some(id) => !archived_ids.contains(&id),
            None => true,
        });
        if arr.is_empty() {
            obj.remove("amendment_log");
        }
    }
}
