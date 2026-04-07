//! `shapes tree` — prints a hierarchical view of the shape or
//! constraint DAG.

use anyhow::Result;

use crate::DagType;
use crate::commands::dag;
use crate::commands::shared::open_store;

/// Prints the requested DAG as an indented tree.
pub fn tree(dag_type: DagType, root: Option<u64>, max_depth: usize) -> Result<()> {
    let store = open_store()?;
    dag::print_tree(&store, dag_type, root, max_depth)
}
