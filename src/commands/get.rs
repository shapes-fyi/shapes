//! `shapes get` — loads a single node by type and ID and emits it in
//! the requested output format.

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::shared::{open_store, output};
use crate::model::{Amendment, Constraint, NodeType, Profile, Shape};
use crate::store::NodeStore;

/// Loads a single node and prints it.
pub fn get(node_type: NodeType, id: u64, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    match node_type {
        NodeType::Shape => output(&store.load::<Shape>(node_type, id)?, format),
        NodeType::Constraint => output(&store.load::<Constraint>(node_type, id)?, format),
        NodeType::Amendment => output(&store.load::<Amendment>(node_type, id)?, format),
        NodeType::Profile => output(&store.load::<Profile>(node_type, id)?, format),
    }
}
