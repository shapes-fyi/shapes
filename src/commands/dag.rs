use std::collections::{BTreeMap, HashSet, VecDeque};
use std::fmt;

use anyhow::Result;
use serde::Serialize;

use crate::DagType;
use crate::model::*;
use crate::store::NodeStore;

// ---------------------------------------------------------------------------
// Validation types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)] // Warning variant will be used in future validation improvements
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => f.write_str("error"),
            Severity::Warning => f.write_str("warning"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ValidationIssue {
    pub invariant: String,
    pub severity: Severity,
    pub node_type: String,
    pub node_id: String,
    pub message: String,
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] [{}] {}:{} — {}",
            self.severity, self.invariant, self.node_type, self.node_id, self.message
        )
    }
}

// ---------------------------------------------------------------------------
// Ancestors
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Descendants
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Effective constraints (shape-specific)
// ---------------------------------------------------------------------------

pub fn effective_constraints(store: &impl NodeStore, shape_id: u64) -> Result<Vec<ConstraintWithSource>> {
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

#[derive(Debug, Serialize)]
pub struct ConstraintWithSource {
    pub constraint_id: ConstraintId,
    pub constraint_name: String,
    pub source_shape_id: ShapeId,
    pub inherited: bool,
}

// ---------------------------------------------------------------------------
// Reverse query: which shapes reference a constraint?
// ---------------------------------------------------------------------------

pub fn shapes_for_constraint(store: &impl NodeStore, constraint_id: u64) -> Result<Vec<ShapeForConstraint>> {
    // Verify the constraint exists
    let _: Constraint = store.load(NodeType::Constraint, constraint_id)?;

    // Load all shapes
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

    // Find all descendants of direct shapes (they inherit the constraint)
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

    // Build result
    let mut result = Vec::new();
    for &sid in direct_shapes.iter().chain(inherited_shapes.iter()) {
        let name = shapes_map
            .get(&sid)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "???".into());
        result.push(ShapeForConstraint {
            shape_id: sid,
            shape_name: name,
            inherited: !direct_shapes.contains(&sid),
        });
    }
    result.sort_by_key(|r| r.shape_id);
    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct ShapeForConstraint {
    pub shape_id: u64,
    pub shape_name: String,
    pub inherited: bool,
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Validate
// ---------------------------------------------------------------------------

pub fn validate(store: &impl NodeStore) -> Result<Vec<ValidationIssue>> {
    let mut issues = Vec::new();

    // Load all nodes
    let shape_ids = store.list_ids(NodeType::Shape)?;
    let constraint_ids = store.list_ids(NodeType::Constraint)?;
    let amendment_ids = store.list_ids(NodeType::Amendment)?;
    let profile_ids = store.list_ids(NodeType::Profile)?;

    fn load_all_or_warn<Id: Ord, T: serde::de::DeserializeOwned>(
        store: &impl NodeStore,
        ids: &[u64],
        node_type: NodeType,
        make_id: impl Fn(u64) -> Id,
        issues: &mut Vec<ValidationIssue>,
    ) -> BTreeMap<Id, T> {
        let mut map = BTreeMap::new();
        for &id in ids {
            match store.load::<T>(node_type, id) {
                Ok(node) => {
                    map.insert(make_id(id), node);
                }
                Err(e) => {
                    issues.push(ValidationIssue {
                        invariant: "PARSE".into(),
                        severity: Severity::Warning,
                        node_type: node_type.to_string(),
                        node_id: id.to_string(),
                        message: format!("failed to parse: {e}"),
                    });
                }
            }
        }
        map
    }

    let shapes: BTreeMap<ShapeId, Shape> =
        load_all_or_warn(store, &shape_ids, NodeType::Shape, ShapeId::new, &mut issues);
    let constraints: BTreeMap<ConstraintId, Constraint> =
        load_all_or_warn(store, &constraint_ids, NodeType::Constraint, ConstraintId::new, &mut issues);
    let amendments: BTreeMap<AmendmentId, Amendment> =
        load_all_or_warn(store, &amendment_ids, NodeType::Amendment, AmendmentId::new, &mut issues);
    let profiles: BTreeMap<ProfileId, Profile> =
        load_all_or_warn(store, &profile_ids, NodeType::Profile, ProfileId::new, &mut issues);

    // --- ID uniqueness (INV-011) ---
    fn check_duplicate_ids(ids: &[u64], type_name: &str, issues: &mut Vec<ValidationIssue>) {
        let mut seen = HashSet::new();
        for &id in ids {
            if !seen.insert(id) {
                issues.push(ValidationIssue {
                    invariant: "INV-011".into(),
                    severity: Severity::Error,
                    node_type: type_name.into(),
                    node_id: id.to_string(),
                    message: format!("duplicate id {id}"),
                });
            }
        }
    }

    check_duplicate_ids(&shape_ids, "shape", &mut issues);
    check_duplicate_ids(&constraint_ids, "constraint", &mut issues);
    check_duplicate_ids(&amendment_ids, "amendment", &mut issues);
    check_duplicate_ids(&profile_ids, "profile", &mut issues);

    // --- Cycle detection (DFS three-color) ---
    detect_cycles_in(
        &shapes,
        "shape",
        "INV-001",
        |s: &Shape| s.child_ids(),
        &mut issues,
    );
    detect_cycles_in(
        &constraints,
        "constraint",
        "INV-002",
        |c: &Constraint| c.child_ids(),
        &mut issues,
    );

    // --- Dangling references ---
    for (&id, shape) in &shapes {
        for &cid in &shape.constraints {
            if !constraints.contains_key(&cid) {
                issues.push(ValidationIssue { invariant: "INV-003".into(), severity: Severity::Error, node_type: "shape".into(), node_id: id.to_string(), message: format!("references non-existent constraint {cid}") });
            }
        }
        for p in &shape.parents {
            if !shapes.contains_key(&p.id) {
                issues.push(ValidationIssue { invariant: "INV-004".into(), severity: Severity::Error, node_type: "shape".into(), node_id: id.to_string(), message: format!("references non-existent parent shape {}", p.id) });
            }
        }
        for child_id in shape.child_ids() {
            if !shapes.contains_key(&child_id) {
                issues.push(ValidationIssue { invariant: "INV-005".into(), severity: Severity::Error, node_type: "shape".into(), node_id: id.to_string(), message: format!("references non-existent child shape {child_id}") });
            }
        }
        if let Some(pid) = shape.profile
            && !profiles.contains_key(&pid)
        {
            issues.push(ValidationIssue { invariant: "INV-006".into(), severity: Severity::Error, node_type: "shape".into(), node_id: id.to_string(), message: format!("references non-existent profile {pid}") });
        }
    }

    for (&id, constraint) in &constraints {
        for p in &constraint.parents {
            if !constraints.contains_key(&p.id) {
                issues.push(ValidationIssue { invariant: "INV-004".into(), severity: Severity::Error, node_type: "constraint".into(), node_id: id.to_string(), message: format!("references non-existent parent constraint {}", p.id) });
            }
        }
        for child_id in constraint.child_ids() {
            if !constraints.contains_key(&child_id) {
                issues.push(ValidationIssue { invariant: "INV-005".into(), severity: Severity::Error, node_type: "constraint".into(), node_id: id.to_string(), message: format!("references non-existent child constraint {child_id}") });
            }
        }
        if let Some(pid) = constraint.profile
            && !profiles.contains_key(&pid)
        {
            issues.push(ValidationIssue { invariant: "INV-006".into(), severity: Severity::Error, node_type: "constraint".into(), node_id: id.to_string(), message: format!("references non-existent profile {pid}") });
        }
    }

    // --- Amendment validation ---
    for (&id, amendment) in &amendments {
        if amendment.targets.is_empty() {
            issues.push(ValidationIssue { invariant: "INV-007".into(), severity: Severity::Error, node_type: "amendment".into(), node_id: id.to_string(), message: "amendment has no targets".into() });
        }
        for &sid in &amendment.targets.shape_ids {
            if !shapes.contains_key(&sid) {
                issues.push(ValidationIssue { invariant: "INV-008".into(), severity: Severity::Error, node_type: "amendment".into(), node_id: id.to_string(), message: format!("targets non-existent shape {sid}") });
            }
        }
        for &cid in &amendment.targets.constraint_ids {
            if !constraints.contains_key(&cid) {
                issues.push(ValidationIssue { invariant: "INV-008".into(), severity: Severity::Error, node_type: "amendment".into(), node_id: id.to_string(), message: format!("targets non-existent constraint {cid}") });
            }
        }
        for &pid in &amendment.targets.profile_ids {
            if !profiles.contains_key(&pid) {
                issues.push(ValidationIssue { invariant: "INV-008".into(), severity: Severity::Error, node_type: "amendment".into(), node_id: id.to_string(), message: format!("targets non-existent profile {pid}") });
            }
        }
    }

    // --- Reciprocal parent/child link checks ---
    for (&id, shape) in &shapes {
        for child_id in shape.child_ids() {
            if let Some(child) = shapes.get(&child_id) {
                let child_lists_parent = child.parents.iter().any(|p| p.id == id);
                if !child_lists_parent {
                    issues.push(ValidationIssue { invariant: "INV-009".into(), severity: Severity::Error, node_type: "shape".into(), node_id: id.to_string(), message: format!("lists shape {child_id} as child, but child does not list {id} as parent") });
                }
            }
        }
    }

    for (&id, constraint) in &constraints {
        for child_id in constraint.child_ids() {
            if let Some(child) = constraints.get(&child_id) {
                let child_lists_parent = child.parents.iter().any(|p| p.id == id);
                if !child_lists_parent {
                    issues.push(ValidationIssue { invariant: "INV-009".into(), severity: Severity::Error, node_type: "constraint".into(), node_id: id.to_string(), message: format!("lists constraint {child_id} as child, but child does not list {id} as parent") });
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
                id.get(),
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
                id.get(),
                &constraint.intent,
                &constraint.metadata,
                &mut issues,
            );
        }
    }

    Ok(issues)
}

// ---------------------------------------------------------------------------
// DFS three-color cycle detection — generic over ID type
// ---------------------------------------------------------------------------

fn detect_cycles_in<Id: Copy + Eq + Ord + std::hash::Hash + fmt::Display, T>(
    nodes: &BTreeMap<Id, T>,
    type_name: &str,
    invariant: &str,
    get_children: impl Fn(&T) -> Vec<Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    let mut colors: BTreeMap<Id, Color> = nodes.keys().map(|&id| (id, Color::White)).collect();

    fn dfs<Id: Copy + Eq + Ord + std::hash::Hash + fmt::Display, T>(
        id: Id,
        nodes: &BTreeMap<Id, T>,
        colors: &mut BTreeMap<Id, Color>,
        get_children: &impl Fn(&T) -> Vec<Id>,
        type_name: &str,
        invariant: &str,
        issues: &mut Vec<ValidationIssue>,
    ) {
        colors.insert(id, Color::Gray);
        if let Some(node) = nodes.get(&id) {
            for child_id in get_children(node) {
                match colors.get(&child_id) {
                    Some(Color::Gray) => {
                        issues.push(ValidationIssue {
                            invariant: invariant.into(),
                            severity: Severity::Error,
                            node_type: type_name.into(),
                            node_id: id.to_string(),
                            message: format!("cycle detected: {id} -> {child_id}"),
                        });
                    }
                    Some(Color::White) => {
                        dfs(child_id, nodes, colors, get_children, type_name, invariant, issues);
                    }
                    _ => {}
                }
            }
        }
        colors.insert(id, Color::Black);
    }

    let ids: Vec<Id> = nodes.keys().copied().collect();
    for id in ids {
        if colors.get(&id) == Some(&Color::White) {
            dfs(id, nodes, &mut colors, &get_children, type_name, invariant, issues);
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
    metadata: &BTreeMap<String, serde_yml::Value>,
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
        check_required_fields(group, &intent.extra, node_type, node_id, "intent", issues);
    }

    // Check metadata fields
    if let Some(ref group) = section.metadata {
        check_required_fields(group, metadata, node_type, node_id, "metadata", issues);
    }
}

fn check_required_fields(
    group: &FieldGroup,
    map: &BTreeMap<String, serde_yml::Value>,
    node_type: &str,
    node_id: u64,
    section_name: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for fd in &group.fields {
        if fd.required && !map.contains_key(&fd.name) {
            issues.push(ValidationIssue {
                invariant: "INV-010".into(),
                severity: Severity::Error,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use serde::de::DeserializeOwned;
    use crate::model::{ShapeId, ConstraintId};
    use crate::model::common::{Intent, Status};
    use crate::store::NodeStore;

    struct MockStore {
        nodes: HashMap<(NodeType, u64), String>,
    }

    impl MockStore {
        fn new() -> Self {
            MockStore {
                nodes: HashMap::new(),
            }
        }

        fn insert<T: Serialize>(&mut self, node_type: NodeType, id: u64, node: &T) {
            let yaml = serde_yml::to_string(node).unwrap();
            self.nodes.insert((node_type, id), yaml);
        }
    }

    impl NodeStore for MockStore {
        fn load<T: DeserializeOwned>(&self, node_type: NodeType, id: u64) -> Result<T> {
            let yaml = self
                .nodes
                .get(&(node_type, id))
                .ok_or_else(|| anyhow::anyhow!("{} {} not found", node_type, id))?;
            Ok(serde_yml::from_str(yaml)?)
        }

        fn list_ids(&self, node_type: NodeType) -> Result<Vec<u64>> {
            let mut ids: Vec<u64> = self
                .nodes
                .keys()
                .filter(|(nt, _)| *nt == node_type)
                .map(|(_, id)| *id)
                .collect();
            ids.sort();
            Ok(ids)
        }
    }

    fn make_shape(id: u64) -> Shape {
        Shape {
            id: ShapeId::new(id),
            name: format!("shape-{id}"),
            description: format!("shape-{id}"),
            profile: None,
            version: None,
            predecessors: vec![],
            status: Status::proposed(),
            intent: Intent {
                kind: "feature".into(),
                summary: format!("shape-{id}"),
                source: serde_yml::Value::String("human".into()),
                uris: vec![],
                extra: Default::default(),
            },
            constraints: vec![],
            realization: vec![],
            evidence: vec![],
            provenance: vec![],
            amendment_log: vec![],
            parents: vec![],
            children: vec![],
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn validate_detects_dangling_constraint_ref() {
        let mut store = MockStore::new();
        let mut shape = make_shape(1);
        shape.constraints = vec![ConstraintId::new(999)]; // dangling
        store.insert(NodeType::Shape, 1, &shape);

        let issues = validate(&store).unwrap();
        assert!(issues.iter().any(|i| i.invariant == "INV-003"));
    }

    #[test]
    fn validate_clean_graph() {
        let store = MockStore::new();
        let issues = validate(&store).unwrap();
        assert!(issues.is_empty());
    }
}
