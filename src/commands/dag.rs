use std::collections::{BTreeMap, HashSet, VecDeque};
use std::fmt;

use anyhow::Result;
use serde::Serialize;

use crate::DagType;
use crate::model::*;
use crate::store::Store;

// ---------------------------------------------------------------------------
// Validation issue
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub node_type: String,
    pub node_id: String,
    pub message: String,
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}:{} — {}",
            self.severity, self.node_type, self.node_id, self.message
        )
    }
}

// ---------------------------------------------------------------------------
// Ancestor / descendant helpers for shapes
// ---------------------------------------------------------------------------

fn shape_parent_ids(shape: &Shape) -> Vec<u64> {
    shape
        .parents
        .as_ref()
        .map(|ps| ps.iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

fn shape_child_ids(shape: &Shape) -> Vec<u64> {
    shape
        .children
        .as_ref()
        .map(|cs| {
            cs.iter()
                .map(|c| match &c.shape {
                    ShapeRef::Id(id) => *id,
                    ShapeRef::Inline(s) => s.id,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn constraint_parent_ids(c: &Constraint) -> Vec<u64> {
    c.parents
        .as_ref()
        .map(|ps| ps.iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

fn constraint_child_ids(c: &Constraint) -> Vec<u64> {
    c.children
        .as_ref()
        .map(|cs| {
            cs.iter()
                .map(|ch| match &ch.constraint {
                    ConstraintRef::Id(id) => *id,
                    ConstraintRef::Inline(c) => c.id,
                })
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Ancestors
// ---------------------------------------------------------------------------

pub fn ancestors(store: &Store, dag_type: DagType, id: u64) -> Result<Vec<u64>> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Seed with parents of the given node
    let parents = match dag_type {
        DagType::Shape => {
            let shape: Shape = store.load(NodeType::Shape, id)?;
            shape_parent_ids(&shape)
        }
        DagType::Constraint => {
            let c: Constraint = store.load(NodeType::Constraint, id)?;
            constraint_parent_ids(&c)
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

        let parents = match dag_type {
            DagType::Shape => {
                if let Ok(s) = store.load::<Shape>(NodeType::Shape, current) {
                    shape_parent_ids(&s)
                } else {
                    vec![]
                }
            }
            DagType::Constraint => {
                if let Ok(c) = store.load::<Constraint>(NodeType::Constraint, current) {
                    constraint_parent_ids(&c)
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

// ---------------------------------------------------------------------------
// Descendants
// ---------------------------------------------------------------------------

pub fn descendants(store: &Store, dag_type: DagType, id: u64) -> Result<Vec<u64>> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    let children = match dag_type {
        DagType::Shape => {
            let shape: Shape = store.load(NodeType::Shape, id)?;
            shape_child_ids(&shape)
        }
        DagType::Constraint => {
            let c: Constraint = store.load(NodeType::Constraint, id)?;
            constraint_child_ids(&c)
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

        let children = match dag_type {
            DagType::Shape => {
                if let Ok(s) = store.load::<Shape>(NodeType::Shape, current) {
                    shape_child_ids(&s)
                } else {
                    vec![]
                }
            }
            DagType::Constraint => {
                if let Ok(c) = store.load::<Constraint>(NodeType::Constraint, current) {
                    constraint_child_ids(&c)
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

// ---------------------------------------------------------------------------
// Effective constraints (shape-specific)
// ---------------------------------------------------------------------------

pub fn effective_constraints(store: &Store, shape_id: u64) -> Result<Vec<ConstraintWithSource>> {
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
            if let Some(ref constraint_ids) = shape.constraints {
                for &cid in constraint_ids {
                    if seen.insert(cid) {
                        let name = store
                            .load::<Constraint>(NodeType::Constraint, cid)
                            .map(|c| c.name)
                            .unwrap_or_else(|_| "???".into());
                        result.push(ConstraintWithSource {
                            constraint_id: cid,
                            constraint_name: name,
                            source_shape_id: current,
                            inherited: current != shape_id,
                        });
                    }
                }
            }
            for pid in shape_parent_ids(&shape) {
                queue.push_back(pid);
            }
        }
    }

    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct ConstraintWithSource {
    pub constraint_id: u64,
    pub constraint_name: String,
    pub source_shape_id: u64,
    pub inherited: bool,
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

pub fn print_tree(store: &Store, dag_type: DagType, root: Option<u64>, max_depth: usize) -> Result<()> {
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
        // Print root node label directly (no connector)
        let (root_label, child_ids, constraint_ids) =
            get_node_info(store, dag_type, *root_id)?;
        println!("{root_label}");

        if max_depth == 0 {
            continue;
        }

        // Print constraint refs and children with connectors
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

fn find_roots(store: &Store, dag_type: DagType) -> Result<Vec<u64>> {
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
                s.parents.as_ref().is_some_and(|p| !p.is_empty())
            }
            DagType::Constraint => {
                let c: Constraint = store.load(node_type, id)?;
                c.parents.as_ref().is_some_and(|p| !p.is_empty())
            }
        };
        if !has_parents {
            roots.push(id);
        }
    }
    Ok(roots)
}

fn get_node_info(
    store: &Store,
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
            let children = shape_child_ids(&s);
            let constraints = s.constraints.clone().unwrap_or_default();
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
            let children = constraint_child_ids(&c);
            Ok((label, children, vec![]))
        }
    }
}

fn print_subtree(
    store: &Store,
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

// ---------------------------------------------------------------------------
// Validate
// ---------------------------------------------------------------------------

pub fn validate(store: &Store) -> Result<Vec<ValidationIssue>> {
    let mut issues = Vec::new();

    // Load all nodes
    let shape_ids = store.list_ids(NodeType::Shape)?;
    let constraint_ids = store.list_ids(NodeType::Constraint)?;
    let amendment_ids = store.list_ids(NodeType::Amendment)?;
    let profile_ids = store.list_ids(NodeType::Profile)?;

    let shapes: BTreeMap<u64, Shape> = shape_ids
        .iter()
        .filter_map(|&id| store.load::<Shape>(NodeType::Shape, id).ok().map(|s| (id, s)))
        .collect();

    let constraints: BTreeMap<u64, Constraint> = constraint_ids
        .iter()
        .filter_map(|&id| {
            store
                .load::<Constraint>(NodeType::Constraint, id)
                .ok()
                .map(|c| (id, c))
        })
        .collect();

    let amendments: BTreeMap<u64, Amendment> = amendment_ids
        .iter()
        .filter_map(|&id| {
            store
                .load::<Amendment>(NodeType::Amendment, id)
                .ok()
                .map(|a| (id, a))
        })
        .collect();

    let profiles: BTreeMap<u64, Profile> = profile_ids
        .iter()
        .filter_map(|&id| {
            store
                .load::<Profile>(NodeType::Profile, id)
                .ok()
                .map(|p| (id, p))
        })
        .collect();

    // --- Cycle detection (DFS three-color) ---
    detect_cycles_in(
        &shapes,
        "shape",
        shape_child_ids,
        &mut issues,
    );
    detect_cycles_in(
        &constraints,
        "constraint",
        constraint_child_ids,
        &mut issues,
    );

    // --- Dangling references ---
    for (&id, shape) in &shapes {
        // Constraint references
        if let Some(ref cids) = shape.constraints {
            for cid in cids {
                if !constraints.contains_key(cid) {
                    issues.push(ValidationIssue {
                        severity: "error".into(),
                        node_type: "shape".into(),
                        node_id: id.to_string(),
                        message: format!("references non-existent constraint {cid}"),
                    });
                }
            }
        }
        // Parent references
        if let Some(ref parents) = shape.parents {
            for p in parents {
                if !shapes.contains_key(&p.id) {
                    issues.push(ValidationIssue {
                        severity: "error".into(),
                        node_type: "shape".into(),
                        node_id: id.to_string(),
                        message: format!("references non-existent parent shape {}", p.id),
                    });
                }
            }
        }
        // Child references
        for child_id in shape_child_ids(shape) {
            if !shapes.contains_key(&child_id) {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    node_type: "shape".into(),
                    node_id: id.to_string(),
                    message: format!("references non-existent child shape {child_id}"),
                });
            }
        }
        // Profile reference
        if let Some(pid) = shape.profile
            && !profiles.contains_key(&pid)
        {
            issues.push(ValidationIssue {
                severity: "error".into(),
                node_type: "shape".into(),
                node_id: id.to_string(),
                message: format!("references non-existent profile {pid}"),
            });
        }
    }

    for (&id, constraint) in &constraints {
        if let Some(ref parents) = constraint.parents {
            for p in parents {
                if !constraints.contains_key(&p.id) {
                    issues.push(ValidationIssue {
                        severity: "error".into(),
                        node_type: "constraint".into(),
                        node_id: id.to_string(),
                        message: format!("references non-existent parent constraint {}", p.id),
                    });
                }
            }
        }
        for child_id in constraint_child_ids(constraint) {
            if !constraints.contains_key(&child_id) {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    node_type: "constraint".into(),
                    node_id: id.to_string(),
                    message: format!("references non-existent child constraint {child_id}"),
                });
            }
        }
        if let Some(pid) = constraint.profile
            && !profiles.contains_key(&pid)
        {
            issues.push(ValidationIssue {
                severity: "error".into(),
                node_type: "constraint".into(),
                node_id: id.to_string(),
                message: format!("references non-existent profile {pid}"),
            });
        }
    }

    // --- Amendment validation ---
    for (&id, amendment) in &amendments {
        if amendment.targets.is_empty() {
            issues.push(ValidationIssue {
                severity: "error".into(),
                node_type: "amendment".into(),
                node_id: id.to_string(),
                message: "amendment has no targets".into(),
            });
        }
        if let Some(ref sids) = amendment.targets.shape_ids {
            for sid in sids {
                if !shapes.contains_key(sid) {
                    issues.push(ValidationIssue {
                        severity: "error".into(),
                        node_type: "amendment".into(),
                        node_id: id.to_string(),
                        message: format!("targets non-existent shape {sid}"),
                    });
                }
            }
        }
        if let Some(ref cids) = amendment.targets.constraint_ids {
            for cid in cids {
                if !constraints.contains_key(cid) {
                    issues.push(ValidationIssue {
                        severity: "error".into(),
                        node_type: "amendment".into(),
                        node_id: id.to_string(),
                        message: format!("targets non-existent constraint {cid}"),
                    });
                }
            }
        }
        if let Some(ref pids) = amendment.targets.profile_ids {
            for pid in pids {
                if !profiles.contains_key(pid) {
                    issues.push(ValidationIssue {
                        severity: "error".into(),
                        node_type: "amendment".into(),
                        node_id: id.to_string(),
                        message: format!("targets non-existent profile {pid}"),
                    });
                }
            }
        }
    }

    // --- Reciprocal parent/child link checks ---
    for (&id, shape) in &shapes {
        for child_id in shape_child_ids(shape) {
            if let Some(child) = shapes.get(&child_id) {
                let child_lists_parent = child
                    .parents
                    .as_ref()
                    .is_some_and(|ps| ps.iter().any(|p| p.id == id));
                if !child_lists_parent {
                    issues.push(ValidationIssue {
                        severity: "warning".into(),
                        node_type: "shape".into(),
                        node_id: id.to_string(),
                        message: format!(
                            "lists shape {child_id} as child, but child does not list {id} as parent"
                        ),
                    });
                }
            }
        }
    }

    for (&id, constraint) in &constraints {
        for child_id in constraint_child_ids(constraint) {
            if let Some(child) = constraints.get(&child_id) {
                let child_lists_parent = child
                    .parents
                    .as_ref()
                    .is_some_and(|ps| ps.iter().any(|p| p.id == id));
                if !child_lists_parent {
                    issues.push(ValidationIssue {
                        severity: "warning".into(),
                        node_type: "constraint".into(),
                        node_id: id.to_string(),
                        message: format!(
                            "lists constraint {child_id} as child, but child does not list {id} as parent"
                        ),
                    });
                }
            }
        }
    }

    // --- Profile field validation ---
    for (&id, shape) in &shapes {
        if let Some(pid) = shape.profile
            && let Some(profile) = profiles.get(&pid)
        {
            validate_profile_fields(
                profile,
                "shape",
                id,
                &shape.intent,
                &shape.metadata,
                &mut issues,
            );
        }
    }

    for (&id, constraint) in &constraints {
        if let Some(pid) = constraint.profile
            && let Some(profile) = profiles.get(&pid)
        {
            validate_profile_fields(
                profile,
                "constraint",
                id,
                &constraint.intent,
                &constraint.metadata,
                &mut issues,
            );
        }
    }

    Ok(issues)
}

// ---------------------------------------------------------------------------
// DFS three-color cycle detection
// ---------------------------------------------------------------------------

fn detect_cycles_in<T>(
    nodes: &BTreeMap<u64, T>,
    type_name: &str,
    get_children: impl Fn(&T) -> Vec<u64>,
    issues: &mut Vec<ValidationIssue>,
) {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    let mut colors: BTreeMap<u64, Color> = nodes.keys().map(|&id| (id, Color::White)).collect();

    fn dfs<T>(
        id: u64,
        nodes: &BTreeMap<u64, T>,
        colors: &mut BTreeMap<u64, Color>,
        get_children: &impl Fn(&T) -> Vec<u64>,
        type_name: &str,
        issues: &mut Vec<ValidationIssue>,
    ) {
        colors.insert(id, Color::Gray);
        if let Some(node) = nodes.get(&id) {
            for child_id in get_children(node) {
                match colors.get(&child_id) {
                    Some(Color::Gray) => {
                        issues.push(ValidationIssue {
                            severity: "error".into(),
                            node_type: type_name.into(),
                            node_id: id.to_string(),
                            message: format!("cycle detected: {id} -> {child_id}"),
                        });
                    }
                    Some(Color::White) => {
                        dfs(child_id, nodes, colors, get_children, type_name, issues);
                    }
                    _ => {}
                }
            }
        }
        colors.insert(id, Color::Black);
    }

    let ids: Vec<u64> = nodes.keys().copied().collect();
    for id in ids {
        if colors.get(&id) == Some(&Color::White) {
            dfs(id, nodes, &mut colors, &get_children, type_name, issues);
        }
    }
}

// ---------------------------------------------------------------------------
// Profile field validation
// ---------------------------------------------------------------------------

fn validate_profile_fields(
    profile: &Profile,
    node_type: &str,
    node_id: u64,
    intent: &Intent,
    metadata: &Option<BTreeMap<String, serde_yaml::Value>>,
    issues: &mut Vec<ValidationIssue>,
) {
    let fields = match &profile.fields {
        Some(f) => f,
        None => return,
    };

    let section = match node_type {
        "shape" => &fields.shape,
        "constraint" => &fields.constraint,
        _ => return,
    };

    let section = match section {
        Some(s) => s,
        None => return,
    };

    // Check intent fields
    if let Some(ref group) = section.intent {
        check_required_fields_in_extra(
            group,
            &intent.extra,
            node_type,
            node_id,
            "intent",
            issues,
        );
    }

    // Check metadata fields
    if let Some(ref group) = section.metadata {
        let empty = BTreeMap::new();
        let meta = metadata.as_ref().unwrap_or(&empty);
        check_required_fields_in_map(group, meta, node_type, node_id, "metadata", issues);
    }
}

fn check_required_fields_in_extra(
    group: &FieldGroup,
    extra: &BTreeMap<String, serde_yaml::Value>,
    node_type: &str,
    node_id: u64,
    section_name: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if let Some(ref field_defs) = group.fields {
        for fd in field_defs {
            if fd.required && !extra.contains_key(&fd.name) {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    node_type: node_type.into(),
                    node_id: node_id.to_string(),
                    message: format!(
                        "missing required {section_name} field '{}' (defined by profile {})",
                        fd.name, "?"
                    ),
                });
            }
        }
    }
}

fn check_required_fields_in_map(
    group: &FieldGroup,
    map: &BTreeMap<String, serde_yaml::Value>,
    node_type: &str,
    node_id: u64,
    section_name: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if let Some(ref field_defs) = group.fields {
        for fd in field_defs {
            if fd.required && !map.contains_key(&fd.name) {
                issues.push(ValidationIssue {
                    severity: "error".into(),
                    node_type: node_type.into(),
                    node_id: node_id.to_string(),
                    message: format!(
                        "missing required {section_name} field '{}' (defined by profile)",
                        fd.name
                    ),
                });
            }
        }
    }
}
