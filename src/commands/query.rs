//! `shapes query` — DAG queries (ancestors, descendants, inherited
//! constraints, reverse constraint lookup).

use anyhow::Result;

use crate::OutputFormat;
use crate::QueryCommand;
use crate::commands::dag;
use crate::commands::shared::{open_store, output};

/// Dispatches the chosen [`QueryCommand`] against the open store.
pub fn query(op: QueryCommand, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    match op {
        QueryCommand::Ancestors { node_type, id } => {
            let result = dag::ancestors(&store, node_type, id)?;
            output(&result, format)
        }
        QueryCommand::Descendants { node_type, id } => {
            let result = dag::descendants(&store, node_type, id)?;
            output(&result, format)
        }
        QueryCommand::Constraints { shape_id } => {
            let result = dag::effective_constraints(&store, shape_id)?;
            output(&result, format)
        }
        QueryCommand::ShapesForConstraint { constraint_id } => {
            let result = dag::shapes_for_constraint(&store, constraint_id)?;
            output(&result, format)
        }
    }
}
