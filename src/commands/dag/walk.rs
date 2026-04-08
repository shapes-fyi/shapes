//! Graph traversal helpers for the shape and constraint DAGs.
//!
//! Implements ancestor / descendant walks, the inherited-constraint
//! lookup that powers `shapes query constraints <shape-id>`, and the
//! reverse "which shapes reference this constraint" lookup.

use std::collections::{BTreeMap, HashSet, VecDeque};

use anyhow::Result;
use serde::Serialize;

use crate::DagType;
use crate::model::*;
use crate::store::NodeStore;

/// Returns all ancestor IDs of `id` in the chosen DAG, breadth-first.
pub fn ancestors(store: &impl NodeStore, dag_type: DagType, id: u64) -> Result<Vec<u64>> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    let parents: Vec<u64> = match dag_type {
        DagType::Shape => {
            let shape: Shape = store.load(NodeType::Shape, id)?;
            shape.parent_ids().into_iter().map(|p| p.get()).collect()
        }
        DagType::Constraint => {
            let c: Constraint = store.load(NodeType::Constraint, id)?;
            c.parent_ids().into_iter().map(|p| p.get()).collect()
        }
    };

    for p in parents {
        queue.push_back(p);
    }

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current) {
            continue;
        }
        result.push(current);

        let parents: Vec<u64> = match dag_type {
            DagType::Shape => {
                if let Ok(s) = store.load::<Shape>(NodeType::Shape, current) {
                    s.parent_ids().into_iter().map(|p| p.get()).collect()
                } else {
                    vec![]
                }
            }
            DagType::Constraint => {
                if let Ok(c) = store.load::<Constraint>(NodeType::Constraint, current) {
                    c.parent_ids().into_iter().map(|p| p.get()).collect()
                } else {
                    vec![]
                }
            }
        };
        for p in parents {
            queue.push_back(p);
        }
    }

    Ok(result)
}

/// Returns all descendant IDs of `id` in the chosen DAG, breadth-first.
pub fn descendants(store: &impl NodeStore, dag_type: DagType, id: u64) -> Result<Vec<u64>> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    let children: Vec<u64> = match dag_type {
        DagType::Shape => {
            let shape: Shape = store.load(NodeType::Shape, id)?;
            shape.child_ids().into_iter().map(|c| c.get()).collect()
        }
        DagType::Constraint => {
            let c: Constraint = store.load(NodeType::Constraint, id)?;
            c.child_ids().into_iter().map(|c| c.get()).collect()
        }
    };

    for ch in children {
        queue.push_back(ch);
    }

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current) {
            continue;
        }
        result.push(current);

        let children: Vec<u64> = match dag_type {
            DagType::Shape => {
                if let Ok(s) = store.load::<Shape>(NodeType::Shape, current) {
                    s.child_ids().into_iter().map(|c| c.get()).collect()
                } else {
                    vec![]
                }
            }
            DagType::Constraint => {
                if let Ok(c) = store.load::<Constraint>(NodeType::Constraint, current) {
                    c.child_ids().into_iter().map(|c| c.get()).collect()
                } else {
                    vec![]
                }
            }
        };
        for ch in children {
            queue.push_back(ch);
        }
    }

    Ok(result)
}

/// Returns every constraint that applies to `shape_id`, including the
/// constraints inherited from ancestor shapes. Each result records
/// which shape it came from and whether it was inherited.
pub fn effective_constraints(
    store: &impl NodeStore,
    shape_id: u64,
) -> Result<Vec<ConstraintWithSource>> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(shape_id);

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current) {
            continue;
        }

        if let Ok(shape) = store.load::<Shape>(NodeType::Shape, current) {
            for &cid in &shape.constraints {
                if seen.insert(cid) {
                    let name = store
                        .load::<Constraint>(NodeType::Constraint, cid.get())
                        .map(|c| c.name)
                        .unwrap_or_else(|_| "???".into());
                    result.push(ConstraintWithSource {
                        constraint_id: cid,
                        constraint_name: name,
                        source_shape_id: ShapeId::new(current),
                        inherited: current != shape_id,
                    });
                }
            }
            for pid in shape.parent_ids() {
                queue.push_back(pid.get());
            }
        }
    }

    Ok(result)
}

/// Result row from [`effective_constraints`].
#[derive(Debug, Serialize)]
pub struct ConstraintWithSource {
    /// Identifier of the inherited or directly-applied constraint.
    pub constraint_id: ConstraintId,
    /// Human-readable name pulled from the constraint node, or `???`
    /// if the constraint cannot be loaded.
    pub constraint_name: String,
    /// Shape that originally declared the constraint.
    pub source_shape_id: ShapeId,
    /// `true` when the constraint was inherited from an ancestor.
    pub inherited: bool,
}

/// Reverse lookup — returns every shape that references `constraint_id`,
/// including shapes that inherit it through descent in the shape DAG.
pub fn shapes_for_constraint(
    store: &impl NodeStore,
    constraint_id: u64,
) -> Result<Vec<ShapeForConstraint>> {
    let _: Constraint = store.load(NodeType::Constraint, constraint_id)?;

    let shape_ids = store.list_ids(NodeType::Shape)?;
    let mut shapes_map: BTreeMap<u64, Shape> = BTreeMap::new();
    let mut direct_shapes: HashSet<u64> = HashSet::new();

    let cid = ConstraintId::new(constraint_id);
    for &sid in &shape_ids {
        if let Ok(shape) = store.load::<Shape>(NodeType::Shape, sid) {
            if shape.constraints.contains(&cid) {
                direct_shapes.insert(sid);
            }
            shapes_map.insert(sid, shape);
        }
    }

    let mut inherited_shapes: HashSet<u64> = HashSet::new();
    for &direct_id in &direct_shapes {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        if let Some(s) = shapes_map.get(&direct_id) {
            for child_id in s.child_ids() {
                queue.push_back(child_id.get());
            }
        }
        while let Some(current) = queue.pop_front() {
            if !visited.insert(current) {
                continue;
            }
            if !direct_shapes.contains(&current) {
                inherited_shapes.insert(current);
            }
            if let Some(s) = shapes_map.get(&current) {
                for child_id in s.child_ids() {
                    queue.push_back(child_id.get());
                }
            }
        }
    }

    let mut result = Vec::new();
    for &sid in direct_shapes.iter().chain(inherited_shapes.iter()) {
        let name = shapes_map
            .get(&sid)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "???".into());
        result.push(ShapeForConstraint {
            shape_id: ShapeId::new(sid),
            shape_name: name,
            inherited: !direct_shapes.contains(&sid),
        });
    }
    result.sort_by_key(|r| r.shape_id);
    Ok(result)
}

/// Result row from [`shapes_for_constraint`].
#[derive(Debug, Serialize)]
pub struct ShapeForConstraint {
    /// Identifier of the matching shape.
    pub shape_id: ShapeId,
    /// Human-readable shape name, or `???` if the shape cannot be
    /// loaded.
    pub shape_name: String,
    /// `true` when this shape only matches because of DAG inheritance.
    pub inherited: bool,
}
