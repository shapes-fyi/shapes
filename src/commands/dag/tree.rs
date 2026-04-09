//! ASCII tree visualization for shape and constraint DAGs.

use anyhow::Result;

use crate::DagType;
use crate::model::*;
use crate::store::NodeStore;

/// Prints the DAG as an ASCII tree with Unicode box-drawing characters.
pub fn print_tree(
    store: &impl NodeStore,
    dag_type: DagType,
    root: Option<u64>,
    max_depth: usize,
) -> Result<()> {
    match dag_type {
        DagType::Shape => print_tree_of::<Shape>(store, root, max_depth),
        DagType::Constraint => print_tree_of::<Constraint>(store, root, max_depth),
    }
}

/// Generic tree printer over any [`DagNode`] type.
fn print_tree_of<N: DagNode>(
    store: &impl NodeStore,
    root: Option<u64>,
    max_depth: usize,
) -> Result<()> {
    let roots = if let Some(root_id) = root {
        vec![root_id]
    } else {
        find_roots_of::<N>(store)?
    };

    if roots.is_empty() {
        eprintln!("No {} nodes found.", N::Id::NODE_TYPE);
        return Ok(());
    }

    for (i, root_id) in roots.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let node: N = store.load(N::Id::NODE_TYPE, *root_id)?;
        println!("{}", node.tree_label());

        if max_depth == 0 {
            continue;
        }

        let constraint_ids = node.constraint_ids();
        let child_ids: Vec<u64> = node.child_ids().into_iter().map(|c| c.get()).collect();
        let total = constraint_ids.len() + child_ids.len();

        for (ci, cid) in constraint_ids.iter().enumerate() {
            let is_last = ci + 1 == total;
            let connector = if is_last {
                "\u{2514}\u{2500}\u{2500} "
            } else {
                "\u{251c}\u{2500}\u{2500} "
            };
            let cname = store
                .load::<Constraint>(NodeType::Constraint, cid.get())
                .map(|c| c.name)
                .unwrap_or_else(|_| "???".into());
            println!("{connector}constraint:{cid} {cname}");
        }
        for (ci, child_id) in child_ids.iter().enumerate() {
            let is_last = constraint_ids.len() + ci + 1 == total;
            print_subtree_of::<N>(store, *child_id, max_depth - 1, "", is_last)?;
        }
    }

    Ok(())
}

/// Finds root nodes (those with no parents) for a given DAG type.
fn find_roots_of<N: DagNode>(store: &impl NodeStore) -> Result<Vec<u64>> {
    let ids = store.list_ids(N::Id::NODE_TYPE)?;
    let mut roots = Vec::new();
    for id in ids {
        let node: N = store.load(N::Id::NODE_TYPE, id)?;
        if DagNode::parent_ids(&node).is_empty() {
            roots.push(id);
        }
    }
    Ok(roots)
}

/// Recursively prints a subtree rooted at `id`.
fn print_subtree_of<N: DagNode>(
    store: &impl NodeStore,
    id: u64,
    depth_remaining: usize,
    prefix: &str,
    is_last: bool,
) -> Result<()> {
    let connector = if is_last {
        "\u{2514}\u{2500}\u{2500} "
    } else {
        "\u{251c}\u{2500}\u{2500} "
    };

    let node: N = store.load(N::Id::NODE_TYPE, id)?;
    let label = node.tree_label();
    let constraint_ids = node.constraint_ids();
    let child_ids: Vec<u64> = node.child_ids().into_iter().map(|c| c.get()).collect();

    println!("{prefix}{connector}{label}");

    let child_prefix = if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}\u{2502}   ")
    };

    if depth_remaining == 0 {
        if !child_ids.is_empty() {
            println!("{child_prefix}... ({} children)", child_ids.len());
        }
        return Ok(());
    }

    let total_items = constraint_ids.len() + child_ids.len();
    for (i, cid) in constraint_ids.iter().enumerate() {
        let is_last_item = i + 1 == total_items;
        let c_connector = if is_last_item {
            "\u{2514}\u{2500}\u{2500} "
        } else {
            "\u{251c}\u{2500}\u{2500} "
        };
        let cname = store
            .load::<Constraint>(NodeType::Constraint, cid.get())
            .map(|c| c.name)
            .unwrap_or_else(|_| "???".into());
        println!("{child_prefix}{c_connector}constraint:{cid} {cname}");
    }

    for (i, child_id) in child_ids.iter().enumerate() {
        let is_last_child = constraint_ids.len() + i + 1 == total_items;
        print_subtree_of::<N>(
            store,
            *child_id,
            depth_remaining - 1,
            &child_prefix,
            is_last_child,
        )?;
    }

    Ok(())
}
