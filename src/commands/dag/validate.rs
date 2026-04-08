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

    // Profile field validation (INV-010, INV-012, INV-013, INV-014, INV-015)
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
                &shape.realization,
                &shape.evidence,
                &shape.provenance,
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
                &constraint.realization,
                &constraint.evidence,
                &constraint.provenance,
                &mut issues,
            );
        }
    }

    // Profile self-consistency (INV-016)
    for profile in profiles.values() {
        validate_profile_self_consistency(profile, &mut issues);
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
    node_type: &str,
    node_id: u64,
    intent: &Intent,
    metadata: &BTreeMap<String, serde_yml::Value>,
    realization: &[Realization],
    evidence: &[Evidence],
    provenance: &[Provenance],
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
    map: &BTreeMap<String, serde_yml::Value>,
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
    map: &BTreeMap<String, serde_yml::Value>,
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
fn value_matches_type(value: &serde_yml::Value, expected_type: &str) -> bool {
    match expected_type {
        "string" => matches!(value, serde_yml::Value::String(_)),
        "bool" | "boolean" => matches!(value, serde_yml::Value::Bool(_)),
        "integer" | "int" => {
            matches!(value, serde_yml::Value::Number(n) if n.is_i64() || n.is_u64())
        }
        "number" | "float" => matches!(value, serde_yml::Value::Number(_)),
        "list" | "sequence" | "array" => matches!(value, serde_yml::Value::Sequence(_)),
        "map" | "mapping" | "object" => matches!(value, serde_yml::Value::Mapping(_)),
        // Unknown type tag — accept silently.
        _ => true,
    }
}

/// Returns a short human-readable name for a YAML value's variant.
/// Used in INV-015 messages.
fn yaml_value_kind(value: &serde_yml::Value) -> &'static str {
    match value {
        serde_yml::Value::Null => "null",
        serde_yml::Value::Bool(_) => "bool",
        serde_yml::Value::Number(_) => "number",
        serde_yml::Value::String(_) => "string",
        serde_yml::Value::Sequence(_) => "sequence",
        serde_yml::Value::Mapping(_) => "mapping",
        serde_yml::Value::Tagged(_) => "tagged",
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
    actual_source: &serde_yml::Value,
    node_type: &str,
    node_id: u64,
    issues: &mut Vec<ValidationIssue>,
) {
    if sources.is_empty() {
        return;
    }
    let source_str = match actual_source {
        serde_yml::Value::String(s) => s.as_str(),
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
