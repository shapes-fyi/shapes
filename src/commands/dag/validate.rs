//! Graph integrity checks (`shapes validate`).
//!
//! Implements every cross-node invariant: cycle detection, reciprocal
//! parent/child links, valid ID references, append-only ID
//! discipline, and the assorted INV-* checks emitted into
//! [`ValidationIssue`].

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::model::bindings::Binding;
use crate::model::profile::{FieldDef, FieldSection};
use crate::model::*;
use crate::store::NodeStore;

/// Severity classification for a [`ValidationIssue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// A hard violation that breaks an invariant.
    Error,
    /// A soft violation that should be looked at.
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

/// Runs every graph integrity check against `store` and returns the
/// list of issues found.
///
/// `workspace_root` is the absolute path of the directory that contains
/// the `.shapes/` store (i.e. the project root). It is used by the
/// path-binding existence check (INV-017) to resolve repo-relative
/// `scheme: path` binding values. Pass `None` for in-memory test stores
/// that have no on-disk workspace; INV-017 will be skipped in that case.
pub fn validate(
    store: &impl NodeStore,
    workspace_root: Option<&Path>,
) -> Result<Vec<ValidationIssue>> {
    let mut issues = Vec::new();

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

    let shapes: BTreeMap<ShapeId, Shape> = load_all_or_warn(
        store,
        &shape_ids,
        NodeType::Shape,
        ShapeId::new,
        &mut issues,
    );
    let constraints: BTreeMap<ConstraintId, Constraint> = load_all_or_warn(
        store,
        &constraint_ids,
        NodeType::Constraint,
        ConstraintId::new,
        &mut issues,
    );
    let amendments: BTreeMap<AmendmentId, Amendment> = load_all_or_warn(
        store,
        &amendment_ids,
        NodeType::Amendment,
        AmendmentId::new,
        &mut issues,
    );
    let profiles: BTreeMap<ProfileId, Profile> = load_all_or_warn(
        store,
        &profile_ids,
        NodeType::Profile,
        ProfileId::new,
        &mut issues,
    );

    // ID uniqueness (INV-011)
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

    // Cycle detection
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

    // Dangling references and constraint refs (INV-003/004/005/006)
    check_dag_refs(&shapes, &constraints, &profiles, &mut issues);
    check_dag_refs(&constraints, &constraints, &profiles, &mut issues);

    // Amendment validation
    for (&id, amendment) in &amendments {
        if amendment.targets.is_empty() {
            issues.push(ValidationIssue {
                invariant: "INV-007".into(),
                severity: Severity::Error,
                node_type: "amendment".into(),
                node_id: id.to_string(),
                message: "amendment has no targets".into(),
            });
        }
        for &sid in &amendment.targets.shape_ids {
            if !shapes.contains_key(&sid) {
                issues.push(ValidationIssue {
                    invariant: "INV-008".into(),
                    severity: Severity::Error,
                    node_type: "amendment".into(),
                    node_id: id.to_string(),
                    message: format!("targets non-existent shape {sid}"),
                });
            }
        }
        for &cid in &amendment.targets.constraint_ids {
            if !constraints.contains_key(&cid) {
                issues.push(ValidationIssue {
                    invariant: "INV-008".into(),
                    severity: Severity::Error,
                    node_type: "amendment".into(),
                    node_id: id.to_string(),
                    message: format!("targets non-existent constraint {cid}"),
                });
            }
        }
        for &pid in &amendment.targets.profile_ids {
            if !profiles.contains_key(&pid) {
                issues.push(ValidationIssue {
                    invariant: "INV-008".into(),
                    severity: Severity::Error,
                    node_type: "amendment".into(),
                    node_id: id.to_string(),
                    message: format!("targets non-existent profile {pid}"),
                });
            }
        }
    }

    // Reciprocal parent/child link checks (INV-009 — both directions).
    check_reciprocal_links(&shapes, &mut issues);
    check_reciprocal_links(&constraints, &mut issues);

    // Amendment-log reciprocity (INV-019 — both directions).
    // Forward: if amendment A targets node N, N.amendment_log must contain A.
    // Reverse: if N.amendment_log contains A, A must target N.
    for (&aid, amendment) in &amendments {
        for &sid in &amendment.targets.shape_ids {
            if let Some(shape) = shapes.get(&sid)
                && !shape.amendment_log.contains(&aid)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-019".into(),
                    severity: Severity::Error,
                    node_type: "amendment".into(),
                    node_id: aid.to_string(),
                    message: format!(
                        "targets shape {sid}, but shape {sid}.amendment_log does not contain {aid}"
                    ),
                });
            }
        }
        for &cid in &amendment.targets.constraint_ids {
            if let Some(constraint) = constraints.get(&cid)
                && !constraint.amendment_log.contains(&aid)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-019".into(),
                    severity: Severity::Error,
                    node_type: "amendment".into(),
                    node_id: aid.to_string(),
                    message: format!(
                        "targets constraint {cid}, but constraint {cid}.amendment_log does not contain {aid}"
                    ),
                });
            }
        }
        for &pid in &amendment.targets.profile_ids {
            if let Some(profile) = profiles.get(&pid)
                && !profile.amendment_log.contains(&aid)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-019".into(),
                    severity: Severity::Error,
                    node_type: "amendment".into(),
                    node_id: aid.to_string(),
                    message: format!(
                        "targets profile {pid}, but profile {pid}.amendment_log does not contain {aid}"
                    ),
                });
            }
        }
    }

    for (&sid, shape) in &shapes {
        for &aid in &shape.amendment_log {
            if let Some(amendment) = amendments.get(&aid)
                && !amendment.targets.shape_ids.contains(&sid)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-019".into(),
                    severity: Severity::Error,
                    node_type: "shape".into(),
                    node_id: sid.to_string(),
                    message: format!(
                        "amendment_log contains {aid}, but amendment {aid} does not target shape {sid}"
                    ),
                });
            }
        }
    }
    for (&cid, constraint) in &constraints {
        for &aid in &constraint.amendment_log {
            if let Some(amendment) = amendments.get(&aid)
                && !amendment.targets.constraint_ids.contains(&cid)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-019".into(),
                    severity: Severity::Error,
                    node_type: "constraint".into(),
                    node_id: cid.to_string(),
                    message: format!(
                        "amendment_log contains {aid}, but amendment {aid} does not target constraint {cid}"
                    ),
                });
            }
        }
    }
    for (&pid, profile) in &profiles {
        for &aid in &profile.amendment_log {
            if let Some(amendment) = amendments.get(&aid)
                && !amendment.targets.profile_ids.contains(&pid)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-019".into(),
                    severity: Severity::Error,
                    node_type: "profile".into(),
                    node_id: pid.to_string(),
                    message: format!(
                        "amendment_log contains {aid}, but amendment {aid} does not target profile {pid}"
                    ),
                });
            }
        }
    }

    // Profile field validation (INV-010, INV-012, INV-013, INV-014, INV-015)
    validate_profile_fields_for(&shapes, &profiles, &mut issues);
    validate_profile_fields_for(&constraints, &profiles, &mut issues);

    // Profile self-consistency (INV-016)
    for profile in profiles.values() {
        validate_profile_self_consistency(profile, &mut issues);
    }

    // Binding target existence (INV-017 path / INV-018 url) — walks every
    // shape, constraint, and amendment binding holder. Path checks are
    // skipped when the caller did not supply a workspace_root (in-memory
    // test stores), since there is no on-disk repo to resolve against.
    check_dag_bindings(&shapes, workspace_root, &mut issues);
    check_dag_bindings(&constraints, workspace_root, &mut issues);
    for (&id, amendment) in &amendments {
        check_node_bindings(
            "amendment",
            id.get(),
            &amendment.realization,
            &amendment.evidence,
            &amendment.provenance,
            workspace_root,
            &mut issues,
        );
    }

    Ok(issues)
}

/// Checks dangling references (INV-003/004/005/006) for a set of DAG
/// nodes. Validates that parent/child/constraint/profile references all
/// point to existing nodes.
fn check_dag_refs<N: DagNode>(
    nodes: &BTreeMap<N::Id, N>,
    constraints: &BTreeMap<ConstraintId, Constraint>,
    profiles: &BTreeMap<ProfileId, Profile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let type_name = N::Id::NODE_TYPE.to_string();
    for (id, node) in nodes {
        // INV-003: constraint references (only meaningful for shapes)
        for cid in node.constraint_ids() {
            if !constraints.contains_key(&cid) {
                issues.push(ValidationIssue {
                    invariant: "INV-003".into(),
                    severity: Severity::Error,
                    node_type: type_name.clone(),
                    node_id: id.to_string(),
                    message: format!("references non-existent constraint {cid}"),
                });
            }
        }
        // INV-004: parent references
        for parent_id in node.parent_ids() {
            if !nodes.contains_key(&parent_id) {
                issues.push(ValidationIssue {
                    invariant: "INV-004".into(),
                    severity: Severity::Error,
                    node_type: type_name.clone(),
                    node_id: id.to_string(),
                    message: format!("references non-existent parent {} {}", type_name, parent_id),
                });
            }
        }
        // INV-005: child references
        for child_id in node.child_ids() {
            if !nodes.contains_key(&child_id) {
                issues.push(ValidationIssue {
                    invariant: "INV-005".into(),
                    severity: Severity::Error,
                    node_type: type_name.clone(),
                    node_id: id.to_string(),
                    message: format!("references non-existent child {} {}", type_name, child_id),
                });
            }
        }
        // INV-006: profile references
        if let Some(pid) = node.profile_id()
            && !profiles.contains_key(&pid)
        {
            issues.push(ValidationIssue {
                invariant: "INV-006".into(),
                severity: Severity::Error,
                node_type: type_name.clone(),
                node_id: id.to_string(),
                message: format!("references non-existent profile {pid}"),
            });
        }
    }
}

/// Checks reciprocal parent/child links (INV-009) for a set of DAG
/// nodes. If A lists B as child, B must list A as parent, and vice
/// versa.
fn check_reciprocal_links<N: DagNode>(
    nodes: &BTreeMap<N::Id, N>,
    issues: &mut Vec<ValidationIssue>,
) {
    let type_name = N::Id::NODE_TYPE.to_string();
    for (id, node) in nodes {
        // Forward: child must list this node as parent
        for child_id in node.child_ids() {
            if let Some(child) = nodes.get(&child_id)
                && !child.parent_ids().contains(id)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-009".into(),
                    severity: Severity::Error,
                    node_type: type_name.clone(),
                    node_id: id.to_string(),
                    message: format!(
                        "lists {type_name} {child_id} as child, but child does not list {id} as parent"
                    ),
                });
            }
        }
        // Reverse: parent must list this node as child
        for parent_id in node.parent_ids() {
            if let Some(parent) = nodes.get(&parent_id)
                && !parent.child_ids().contains(id)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-009".into(),
                    severity: Severity::Error,
                    node_type: type_name.clone(),
                    node_id: id.to_string(),
                    message: format!(
                        "lists {type_name} {parent_id} as parent, but parent does not list {id} as child"
                    ),
                });
            }
        }
    }
}

/// Validates profile-driven field requirements for a set of DAG nodes.
fn validate_profile_fields_for<N: DagNode>(
    nodes: &BTreeMap<N::Id, N>,
    profiles: &BTreeMap<ProfileId, Profile>,
    issues: &mut Vec<ValidationIssue>,
) {
    for (id, node) in nodes {
        if let Some(pid) = node.profile_id()
            && let Some(profile) = profiles.get(&pid)
        {
            validate_profile_fields(
                profile,
                N::Id::NODE_TYPE,
                id.get(),
                node.intent(),
                node.metadata(),
                node.realization(),
                node.evidence(),
                node.provenance(),
                issues,
            );
        }
    }
}

/// Checks bindings (INV-017/018) for a set of DAG nodes.
fn check_dag_bindings<N: DagNode>(
    nodes: &BTreeMap<N::Id, N>,
    workspace_root: Option<&Path>,
    issues: &mut Vec<ValidationIssue>,
) {
    let type_name = N::Id::NODE_TYPE.to_string();
    for (id, node) in nodes {
        check_node_bindings(
            &type_name,
            id.get(),
            node.realization(),
            node.evidence(),
            node.provenance(),
            workspace_root,
            issues,
        );
    }
}

/// Walks every binding-holding array on a node (realization, evidence,
/// provenance) and runs the path-existence (INV-017) and url-well-formed
/// (INV-018) checks against each binding. Path checks are skipped when
/// `workspace_root` is `None`.
fn check_node_bindings(
    node_type: &str,
    node_id: u64,
    realization: &[Realization],
    evidence: &[Evidence],
    provenance: &[Provenance],
    workspace_root: Option<&Path>,
    issues: &mut Vec<ValidationIssue>,
) {
    for (idx, r) in realization.iter().enumerate() {
        let location = format!("realization[{idx}]");
        check_url_binding_well_formed(&r.bindings, node_type, node_id, &location, issues);
        if let Some(root) = workspace_root {
            check_path_binding_exists(root, &r.bindings, node_type, node_id, &location, issues);
        }
    }
    for (idx, e) in evidence.iter().enumerate() {
        let location = format!("evidence[{idx}]");
        check_url_binding_well_formed(&e.bindings, node_type, node_id, &location, issues);
        if let Some(root) = workspace_root {
            check_path_binding_exists(root, &e.bindings, node_type, node_id, &location, issues);
        }
    }
    for (idx, p) in provenance.iter().enumerate() {
        let location = format!("provenance[{idx}]");
        check_url_binding_well_formed(&p.bindings, node_type, node_id, &location, issues);
        if let Some(root) = workspace_root {
            check_path_binding_exists(root, &p.bindings, node_type, node_id, &location, issues);
        }
    }
}

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
                        dfs(
                            child_id,
                            nodes,
                            colors,
                            get_children,
                            type_name,
                            invariant,
                            issues,
                        );
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
            dfs(
                id,
                nodes,
                &mut colors,
                &get_children,
                type_name,
                invariant,
                issues,
            );
        }
    }
}

/// Runs every profile-driven check against a single shape or constraint:
/// required intent / metadata / per-binding metadata fields (INV-010 +
/// INV-014), kind allow-list (INV-012), source allow-list (INV-013),
/// and field type checks (INV-015).
//
// `clippy::too_many_arguments` is allowed because this is the single
// funnel for every profile-driven check against one node. Bundling
// intent / metadata / realization / evidence / provenance / issues +
// node identity into a wrapper struct would be more ceremony than
// signal. Remove this allow if the function ever needs to be split.
#[allow(clippy::too_many_arguments)]
fn validate_profile_fields(
    profile: &Profile,
    node_type: NodeType,
    node_id: u64,
    intent: &Intent,
    metadata: &BTreeMap<String, serde_yaml_ng::Value>,
    realization: &[Realization],
    evidence: &[Evidence],
    provenance: &[Provenance],
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(ref fields) = profile.fields else {
        return;
    };
    let section = match node_type {
        NodeType::Shape => &fields.shape,
        NodeType::Constraint => &fields.constraint,
        NodeType::Amendment | NodeType::Profile => return,
    };
    let Some(section) = section else { return };
    // Convert to &str once for downstream display helpers.
    let type_str = node_type.to_string();
    let node_type = type_str.as_str();

    if let Some(ref group) = section.intent {
        // INV-010: required intent fields.
        check_required_fields(
            group,
            &intent.extra,
            node_type,
            node_id,
            "intent",
            "INV-010",
            issues,
        );
        // INV-015: type checks for declared intent fields.
        check_field_types(
            &group.fields,
            &intent.extra,
            node_type,
            node_id,
            "intent",
            issues,
        );
        // INV-012: intent.kind in allow-list (when one is declared).
        check_kind_in_allow_list(&group.kinds, &intent.kind, node_type, node_id, issues);
        // INV-013: intent.source in allow-list (when one is declared).
        check_source_in_allow_list(&group.sources, &intent.source, node_type, node_id, issues);
    }
    if let Some(ref group) = section.metadata {
        check_required_fields(
            group, metadata, node_type, node_id, "metadata", "INV-010", issues,
        );
        check_field_types(
            &group.fields,
            metadata,
            node_type,
            node_id,
            "metadata",
            issues,
        );
    }
    if let Some(ref group) = section.realization {
        for (idx, r) in realization.iter().enumerate() {
            for (bidx, b) in r.bindings.iter().enumerate() {
                let location = format!("realization[{idx}].bindings[{bidx}].metadata");
                check_required_fields(
                    group,
                    &b.metadata,
                    node_type,
                    node_id,
                    &location,
                    "INV-014",
                    issues,
                );
                check_field_types(
                    &group.fields,
                    &b.metadata,
                    node_type,
                    node_id,
                    &location,
                    issues,
                );
            }
        }
    }
    if let Some(ref group) = section.evidence {
        for (idx, e) in evidence.iter().enumerate() {
            for (bidx, b) in e.bindings.iter().enumerate() {
                let location = format!("evidence[{idx}].bindings[{bidx}].metadata");
                check_required_fields(
                    group,
                    &b.metadata,
                    node_type,
                    node_id,
                    &location,
                    "INV-014",
                    issues,
                );
                check_field_types(
                    &group.fields,
                    &b.metadata,
                    node_type,
                    node_id,
                    &location,
                    issues,
                );
            }
        }
    }
    if let Some(ref group) = section.provenance {
        for (idx, p) in provenance.iter().enumerate() {
            for (bidx, b) in p.bindings.iter().enumerate() {
                let location = format!("provenance[{idx}].bindings[{bidx}].metadata");
                check_required_fields(
                    group,
                    &b.metadata,
                    node_type,
                    node_id,
                    &location,
                    "INV-014",
                    issues,
                );
                check_field_types(
                    &group.fields,
                    &b.metadata,
                    node_type,
                    node_id,
                    &location,
                    issues,
                );
            }
        }
    }
}

/// Walks `group.fields` and emits an issue for any `required: true`
/// field whose name is missing from `map`.
fn check_required_fields(
    group: &FieldGroup,
    map: &BTreeMap<String, serde_yaml_ng::Value>,
    node_type: &str,
    node_id: u64,
    location: &str,
    invariant: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for fd in &group.fields {
        if fd.required && !map.contains_key(&fd.name) {
            issues.push(ValidationIssue {
                invariant: invariant.into(),
                severity: Severity::Error,
                node_type: node_type.into(),
                node_id: node_id.to_string(),
                message: format!(
                    "missing required {location} field '{}' (defined by profile)",
                    fd.name
                ),
            });
        }
    }
}

/// For every field in `fields` with a declared `field_type`, checks
/// that the corresponding value in `map` (if present) matches.
fn check_field_types(
    fields: &[FieldDef],
    map: &BTreeMap<String, serde_yaml_ng::Value>,
    node_type: &str,
    node_id: u64,
    location: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for fd in fields {
        let Some(ref expected_type) = fd.field_type else {
            continue;
        };
        let Some(value) = map.get(&fd.name) else {
            continue;
        };
        if !value_matches_type(value, expected_type) {
            issues.push(ValidationIssue {
                invariant: "INV-015".into(),
                severity: Severity::Error,
                node_type: node_type.into(),
                node_id: node_id.to_string(),
                message: format!(
                    "{location}.{} has type {actual}, expected {expected_type} (declared by profile)",
                    fd.name,
                    actual = yaml_value_kind(value),
                ),
            });
        }
    }
}

/// Returns `true` when `value` matches the declared `expected_type`.
/// Recognized type tags: `string`, `integer` / `int`, `number` /
/// `float`, `bool` / `boolean`, `list` / `sequence` / `array`,
/// `map` / `mapping` / `object`. Unknown type tags are accepted —
/// profiles may use arbitrary type strings as documentation hints
/// without tripping the validator.
fn value_matches_type(value: &serde_yaml_ng::Value, expected_type: &str) -> bool {
    match expected_type {
        "string" => matches!(value, serde_yaml_ng::Value::String(_)),
        "bool" | "boolean" => matches!(value, serde_yaml_ng::Value::Bool(_)),
        "integer" | "int" => {
            matches!(value, serde_yaml_ng::Value::Number(n) if n.is_i64() || n.is_u64())
        }
        "number" | "float" => matches!(value, serde_yaml_ng::Value::Number(_)),
        "list" | "sequence" | "array" => matches!(value, serde_yaml_ng::Value::Sequence(_)),
        "map" | "mapping" | "object" => matches!(value, serde_yaml_ng::Value::Mapping(_)),
        // Unknown type tag — accept silently.
        _ => true,
    }
}

/// Returns a short human-readable name for a YAML value's variant.
/// Used in INV-015 messages.
fn yaml_value_kind(value: &serde_yaml_ng::Value) -> &'static str {
    match value {
        serde_yaml_ng::Value::Null => "null",
        serde_yaml_ng::Value::Bool(_) => "bool",
        serde_yaml_ng::Value::Number(_) => "number",
        serde_yaml_ng::Value::String(_) => "string",
        serde_yaml_ng::Value::Sequence(_) => "sequence",
        serde_yaml_ng::Value::Mapping(_) => "mapping",
        serde_yaml_ng::Value::Tagged(_) => "tagged",
    }
}

/// INV-012: emits an issue when a node's `intent.kind` is not in the
/// profile's allow-list. No-op when the profile declares an empty
/// allow-list (kinds are unrestricted).
fn check_kind_in_allow_list(
    kinds: &[FieldDef],
    actual_kind: &str,
    node_type: &str,
    node_id: u64,
    issues: &mut Vec<ValidationIssue>,
) {
    if kinds.is_empty() {
        return;
    }
    if !kinds.iter().any(|k| k.name == actual_kind) {
        let allowed: Vec<&str> = kinds.iter().map(|k| k.name.as_str()).collect();
        issues.push(ValidationIssue {
            invariant: "INV-012".into(),
            severity: Severity::Error,
            node_type: node_type.into(),
            node_id: node_id.to_string(),
            message: format!(
                "intent.kind '{actual_kind}' not in profile allow-list ({})",
                allowed.join(", "),
            ),
        });
    }
}

/// INV-013: emits an issue when a node's `intent.source` is not in
/// the profile's allow-list. Non-string sources also fail (when an
/// allow-list is declared, sources must be strings).
fn check_source_in_allow_list(
    sources: &[FieldDef],
    actual_source: &serde_yaml_ng::Value,
    node_type: &str,
    node_id: u64,
    issues: &mut Vec<ValidationIssue>,
) {
    if sources.is_empty() {
        return;
    }
    let source_str = match actual_source {
        serde_yaml_ng::Value::String(s) => s.as_str(),
        _ => {
            issues.push(ValidationIssue {
                invariant: "INV-013".into(),
                severity: Severity::Error,
                node_type: node_type.into(),
                node_id: node_id.to_string(),
                message: "intent.source must be a string when profile defines an allow-list".into(),
            });
            return;
        }
    };
    if !sources.iter().any(|s| s.name == source_str) {
        let allowed: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
        issues.push(ValidationIssue {
            invariant: "INV-013".into(),
            severity: Severity::Error,
            node_type: node_type.into(),
            node_id: node_id.to_string(),
            message: format!(
                "intent.source '{source_str}' not in profile allow-list ({})",
                allowed.join(", "),
            ),
        });
    }
}

/// INV-016: profile self-consistency. Checks that
/// `default_kind` (when set) appears in `intent.kinds`, and that no
/// FieldDef list contains duplicate names.
fn validate_profile_self_consistency(profile: &Profile, issues: &mut Vec<ValidationIssue>) {
    let pid = profile.id.get();
    let Some(ref fields) = profile.fields else {
        return;
    };
    if let Some(ref shape_section) = fields.shape {
        check_section_self_consistency(shape_section, "fields.shape", pid, issues);
    }
    if let Some(ref constraint_section) = fields.constraint {
        check_section_self_consistency(constraint_section, "fields.constraint", pid, issues);
    }
}

fn check_section_self_consistency(
    section: &FieldSection,
    location: &str,
    profile_id: u64,
    issues: &mut Vec<ValidationIssue>,
) {
    // default_kind must be in the kinds allow-list (when both are set).
    if let Some(ref default_kind) = section.default_kind
        && let Some(ref intent_group) = section.intent
        && !intent_group.kinds.is_empty()
        && !intent_group.kinds.iter().any(|k| &k.name == default_kind)
    {
        issues.push(ValidationIssue {
            invariant: "INV-016".into(),
            severity: Severity::Error,
            node_type: "profile".into(),
            node_id: profile_id.to_string(),
            message: format!(
                "{location}.default_kind '{default_kind}' is not in {location}.intent.kinds allow-list",
            ),
        });
    }

    // No duplicate field/kind/source names within any group.
    if let Some(ref group) = section.intent {
        check_no_duplicate_names(&group.fields, location, "intent.fields", profile_id, issues);
        check_no_duplicate_names(&group.kinds, location, "intent.kinds", profile_id, issues);
        check_no_duplicate_names(
            &group.sources,
            location,
            "intent.sources",
            profile_id,
            issues,
        );
    }
    if let Some(ref group) = section.metadata {
        check_no_duplicate_names(
            &group.fields,
            location,
            "metadata.fields",
            profile_id,
            issues,
        );
    }
    if let Some(ref group) = section.realization {
        check_no_duplicate_names(
            &group.fields,
            location,
            "realization.fields",
            profile_id,
            issues,
        );
    }
    if let Some(ref group) = section.evidence {
        check_no_duplicate_names(
            &group.fields,
            location,
            "evidence.fields",
            profile_id,
            issues,
        );
    }
    if let Some(ref group) = section.provenance {
        check_no_duplicate_names(
            &group.fields,
            location,
            "provenance.fields",
            profile_id,
            issues,
        );
    }
}

fn check_no_duplicate_names(
    fields: &[FieldDef],
    location: &str,
    group_name: &str,
    profile_id: u64,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut seen = HashSet::new();
    for fd in fields {
        if !seen.insert(fd.name.as_str()) {
            issues.push(ValidationIssue {
                invariant: "INV-016".into(),
                severity: Severity::Error,
                node_type: "profile".into(),
                node_id: profile_id.to_string(),
                message: format!(
                    "{location}.{group_name} contains duplicate name '{}'",
                    fd.name,
                ),
            });
        }
    }
}

/// INV-017: every `scheme: path` binding must be a non-empty,
/// repo-relative, slash-separated path that lexically stays under
/// `workspace_root` and resolves to a file or directory on disk.
///
/// The check is intentionally lexical for the escape rule (`..`
/// components) and only touches the filesystem to confirm existence —
/// it never calls `canonicalize`, which would mangle the location
/// string in error output and behave differently on case-insensitive
/// volumes.
fn check_path_binding_exists(
    workspace_root: &Path,
    bindings: &[Binding],
    node_type: &str,
    node_id: u64,
    location: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for (bidx, binding) in bindings.iter().enumerate() {
        if binding.scheme != "path" {
            continue;
        }
        let value = binding.value.as_str();
        let where_ = format!("{location}.bindings[{bidx}]");
        let push = |msg: String, issues: &mut Vec<ValidationIssue>| {
            issues.push(ValidationIssue {
                invariant: "INV-017".into(),
                severity: Severity::Error,
                node_type: node_type.into(),
                node_id: node_id.to_string(),
                message: format!("{where_}: {msg}"),
            });
        };

        if value.is_empty() {
            push("path binding value is empty".into(), issues);
            continue;
        }
        if value.contains('\\') {
            push(
                format!("path '{value}' contains '\\'; bindings must use '/' separators"),
                issues,
            );
            continue;
        }
        let candidate = Path::new(value);
        if candidate.is_absolute() {
            push(
                format!("path '{value}' is absolute; bindings must be repo-relative"),
                issues,
            );
            continue;
        }
        // Lexical normalization: walk components, reject any net-negative
        // depth so `../../escape` is caught without touching the filesystem.
        let mut depth: i32 = 0;
        let mut escapes = false;
        for comp in candidate.components() {
            match comp {
                Component::CurDir => {}
                Component::ParentDir => {
                    depth -= 1;
                    if depth < 0 {
                        escapes = true;
                        break;
                    }
                }
                Component::Normal(_) => {
                    depth += 1;
                }
                Component::RootDir | Component::Prefix(_) => {
                    escapes = true;
                    break;
                }
            }
        }
        if escapes {
            push(format!("path '{value}' escapes the workspace root"), issues);
            continue;
        }

        let resolved: PathBuf = workspace_root.join(candidate);
        // `Path::exists` follows symlinks; a dangling symlink fails here,
        // which is the documented INV-017 behavior.
        if !resolved.exists() {
            push(
                format!("path '{value}' does not resolve to an existing file or directory"),
                issues,
            );
        }
    }
}

/// INV-018: every `scheme: url` binding must be a non-empty
/// well-formed absolute URL with scheme in {http, https, git, ssh}.
/// Hand-rolled offline check — no network and no `url` crate
/// dependency. Tighten only if the repo's URL bindings ever grow
/// past trivial cases.
fn check_url_binding_well_formed(
    bindings: &[Binding],
    node_type: &str,
    node_id: u64,
    location: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    const ALLOWED_SCHEMES: &[&str] = &["http", "https", "git", "ssh"];
    for (bidx, binding) in bindings.iter().enumerate() {
        if binding.scheme != "url" {
            continue;
        }
        let value = binding.value.as_str();
        let where_ = format!("{location}.bindings[{bidx}]");
        let push = |msg: String, issues: &mut Vec<ValidationIssue>| {
            issues.push(ValidationIssue {
                invariant: "INV-018".into(),
                severity: Severity::Error,
                node_type: node_type.into(),
                node_id: node_id.to_string(),
                message: format!("{where_}: {msg}"),
            });
        };

        if value.is_empty() {
            push("url binding value is empty".into(), issues);
            continue;
        }
        if value.chars().any(char::is_whitespace) {
            push(format!("url '{value}' contains whitespace"), issues);
            continue;
        }
        let Some((scheme, rest)) = value.split_once("://") else {
            push(
                format!("url '{value}' is not a well-formed absolute URL (missing scheme://)"),
                issues,
            );
            continue;
        };
        if !ALLOWED_SCHEMES.contains(&scheme) {
            push(
                format!(
                    "url '{value}' has unsupported scheme '{scheme}'; allowed: {}",
                    ALLOWED_SCHEMES.join(", ")
                ),
                issues,
            );
            continue;
        }
        if rest.is_empty() {
            push(
                format!("url '{value}' has empty authority/path after '{scheme}://'"),
                issues,
            );
        }
    }
}
