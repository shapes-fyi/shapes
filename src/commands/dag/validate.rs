//! Graph integrity checks (`shapes validate`).
//!
//! Implements every cross-node invariant: cycle detection, reciprocal
//! parent/child links, valid ID references, append-only ID
//! discipline, and the assorted INV-* checks emitted into
//! [`ValidationIssue`].

use std::collections::{BTreeMap, HashSet};
use std::fmt;

use anyhow::Result;
use serde::Serialize;

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

pub fn validate(store: &impl NodeStore) -> Result<Vec<ValidationIssue>> {
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

    // Dangling references
    for (&id, shape) in &shapes {
        for &cid in &shape.constraints {
            if !constraints.contains_key(&cid) {
                issues.push(ValidationIssue {
                    invariant: "INV-003".into(),
                    severity: Severity::Error,
                    node_type: "shape".into(),
                    node_id: id.to_string(),
                    message: format!("references non-existent constraint {cid}"),
                });
            }
        }
        for p in &shape.parents {
            if !shapes.contains_key(&p.id) {
                issues.push(ValidationIssue {
                    invariant: "INV-004".into(),
                    severity: Severity::Error,
                    node_type: "shape".into(),
                    node_id: id.to_string(),
                    message: format!("references non-existent parent shape {}", p.id),
                });
            }
        }
        for child_id in shape.child_ids() {
            if !shapes.contains_key(&child_id) {
                issues.push(ValidationIssue {
                    invariant: "INV-005".into(),
                    severity: Severity::Error,
                    node_type: "shape".into(),
                    node_id: id.to_string(),
                    message: format!("references non-existent child shape {child_id}"),
                });
            }
        }
        if let Some(pid) = shape.profile
            && !profiles.contains_key(&pid)
        {
            issues.push(ValidationIssue {
                invariant: "INV-006".into(),
                severity: Severity::Error,
                node_type: "shape".into(),
                node_id: id.to_string(),
                message: format!("references non-existent profile {pid}"),
            });
        }
    }

    for (&id, constraint) in &constraints {
        for p in &constraint.parents {
            if !constraints.contains_key(&p.id) {
                issues.push(ValidationIssue {
                    invariant: "INV-004".into(),
                    severity: Severity::Error,
                    node_type: "constraint".into(),
                    node_id: id.to_string(),
                    message: format!("references non-existent parent constraint {}", p.id),
                });
            }
        }
        for child_id in constraint.child_ids() {
            if !constraints.contains_key(&child_id) {
                issues.push(ValidationIssue {
                    invariant: "INV-005".into(),
                    severity: Severity::Error,
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
                invariant: "INV-006".into(),
                severity: Severity::Error,
                node_type: "constraint".into(),
                node_id: id.to_string(),
                message: format!("references non-existent profile {pid}"),
            });
        }
    }

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

    // Reciprocal parent/child link checks
    for (&id, shape) in &shapes {
        for child_id in shape.child_ids() {
            if let Some(child) = shapes.get(&child_id)
                && !child.parents.iter().any(|p| p.id == id)
            {
                issues.push(ValidationIssue {
                    invariant: "INV-009".into(),
                    severity: Severity::Error,
                    node_type: "shape".into(),
                    node_id: id.to_string(),
                    message: format!(
                        "lists shape {child_id} as child, but child does not list {id} as parent"
                    ),
                });
            }
        }
    }

    for (&id, constraint) in &constraints {
        for child_id in constraint.child_ids() {
            if let Some(child) = constraints.get(&child_id)
                && !child.parents.iter().any(|p| p.id == id)
            {
                issues.push(ValidationIssue { invariant: "INV-009".into(), severity: Severity::Error, node_type: "constraint".into(), node_id: id.to_string(), message: format!("lists constraint {child_id} as child, but child does not list {id} as parent") });
            }
        }
    }

    // Profile field validation
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

fn validate_profile_fields(
    profile: &Profile,
    node_type: &str,
    node_id: u64,
    intent: &Intent,
    metadata: &BTreeMap<String, serde_yml::Value>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(ref fields) = profile.fields else {
        return;
    };
    let section = match node_type {
        "shape" => &fields.shape,
        "constraint" => &fields.constraint,
        _ => return,
    };
    let Some(section) = section else { return };

    if let Some(ref group) = section.intent {
        check_required_fields(group, &intent.extra, node_type, node_id, "intent", issues);
    }
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
