use anyhow::Result;

use crate::DagType;
use crate::model::*;
use crate::store::NodeStore;

pub fn print_tree(store: &impl NodeStore, dag_type: DagType, root: Option<u64>, max_depth: usize) -> Result<()> {
    let label = match dag_type {
        DagType::Shape => "shape",
        DagType::Constraint => "constraint",
    };

    let roots = if let Some(root_id) = root {
        vec![root_id]
    } else {
        find_roots(store, dag_type)?
    };

    if roots.is_empty() {
        eprintln!("No {label} nodes found.");
        return Ok(());
    }

    for (i, root_id) in roots.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let (root_label, child_ids, constraint_ids) =
            get_node_info(store, dag_type, *root_id)?;
        println!("{root_label}");

        if max_depth == 0 {
            continue;
        }

        let total = constraint_ids.len() + child_ids.len();
        for (ci, cid) in constraint_ids.iter().enumerate() {
            let is_last = ci + 1 == total;
            let connector = if is_last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " };
            let cname = store
                .load::<Constraint>(NodeType::Constraint, *cid)
                .map(|c| c.name)
                .unwrap_or_else(|_| "???".into());
            println!("{connector}constraint:{cid} {cname}");
        }
        for (ci, child_id) in child_ids.iter().enumerate() {
            let is_last = constraint_ids.len() + ci + 1 == total;
            print_subtree(store, dag_type, *child_id, max_depth - 1, "", is_last)?;
        }
    }

    Ok(())
}

fn find_roots(store: &impl NodeStore, dag_type: DagType) -> Result<Vec<u64>> {
    let node_type = match dag_type {
        DagType::Shape => NodeType::Shape,
        DagType::Constraint => NodeType::Constraint,
    };
    let ids = store.list_ids(node_type)?;

    let mut roots = Vec::new();
    for id in ids {
        let has_parents = match dag_type {
            DagType::Shape => {
                let s: Shape = store.load(node_type, id)?;
                !s.parents.is_empty()
            }
            DagType::Constraint => {
                let c: Constraint = store.load(node_type, id)?;
                !c.parents.is_empty()
            }
        };
        if !has_parents {
            roots.push(id);
        }
    }
    Ok(roots)
}

fn get_node_info(
    store: &impl NodeStore,
    dag_type: DagType,
    id: u64,
) -> Result<(String, Vec<u64>, Vec<u64>)> {
    let node_type = match dag_type {
        DagType::Shape => NodeType::Shape,
        DagType::Constraint => NodeType::Constraint,
    };
    match dag_type {
        DagType::Shape => {
            let s: Shape = store.load(node_type, id)?;
            let label = format!(
                "shape:{} {} [{}] kind={}",
                s.id,
                s.name,
                s.status.name(),
                s.intent.kind
            );
            let children: Vec<u64> = s.child_ids().into_iter().map(|c| c.get()).collect();
            let constraints: Vec<u64> = s.constraints.iter().map(|c| c.get()).collect();
            Ok((label, children, constraints))
        }
        DagType::Constraint => {
            let c: Constraint = store.load(node_type, id)?;
            let label = format!(
                "constraint:{} {} [{}] kind={}",
                c.id,
                c.name,
                c.status.name(),
                c.kind
            );
            let children: Vec<u64> = c.child_ids().into_iter().map(|c| c.get()).collect();
            Ok((label, children, vec![]))
        }
    }
}

fn print_subtree(
    store: &impl NodeStore,
    dag_type: DagType,
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

    let (label, child_ids, constraint_ids) = get_node_info(store, dag_type, id)?;

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
            .load::<Constraint>(NodeType::Constraint, *cid)
            .map(|c| c.name)
            .unwrap_or_else(|_| "???".into());
        println!("{child_prefix}{c_connector}constraint:{cid} {cname}");
    }

    for (i, child_id) in child_ids.iter().enumerate() {
        let is_last_child = constraint_ids.len() + i + 1 == total_items;
        print_subtree(
            store,
            dag_type,
            *child_id,
            depth_remaining - 1,
            &child_prefix,
            is_last_child,
        )?;
    }

    Ok(())
}
