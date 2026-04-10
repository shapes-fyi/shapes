use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::model::*;
use crate::model::{Intent, Status};
use crate::store::NodeStore;

use super::validate::validate;

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
        let yaml = serde_yaml_ng::to_string(node).unwrap();
        self.nodes.insert((node_type, id), yaml);
    }
}

impl NodeStore for MockStore {
    fn load<T: DeserializeOwned>(&self, node_type: NodeType, id: u64) -> Result<T> {
        let yaml = self
            .nodes
            .get(&(node_type, id))
            .ok_or_else(|| anyhow::anyhow!("{} {} not found", node_type, id))?;
        Ok(serde_yaml_ng::from_str(yaml)?)
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
            source: serde_yaml_ng::Value::String("human".into()),
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
    shape.constraints = vec![ConstraintId::new(999)];
    store.insert(NodeType::Shape, 1, &shape);

    let issues = validate(&store, None).unwrap();
    assert!(issues.iter().any(|i| i.invariant == "INV-003"));
}

#[test]
fn validate_clean_graph() {
    let store = MockStore::new();
    let issues = validate(&store, None).unwrap();
    assert!(issues.is_empty());
}

fn make_amendment(id: u64, shape_targets: Vec<u64>) -> Amendment {
    Amendment {
        id: AmendmentId::new(id),
        name: format!("amendment-{id}"),
        description: format!("amendment-{id}"),
        targets: AmendmentTargets {
            shape_ids: shape_targets.into_iter().map(ShapeId::new).collect(),
            constraint_ids: vec![],
            profile_ids: vec![],
        },
        status: Status::proposed(),
        version_impact: None,
        intent: Intent {
            kind: "amendment".into(),
            summary: format!("amendment-{id}"),
            source: serde_yaml_ng::Value::String("ai".into()),
            uris: vec![],
            extra: Default::default(),
        },
        constraints: vec![],
        realization: vec![],
        evidence: vec![],
        provenance: vec![],
        initiated_by: InitiatedBy {
            initiated_type: InitiatedType::Ai,
            identity: None,
            provenance: None,
        },
        archived: false,
        metadata: BTreeMap::new(),
    }
}

#[test]
fn validate_detects_missing_child_in_parent() {
    // Parent A lists child B via parents-only (A says "my parent is B"),
    // but B does not list A as a child. INV-009 reverse direction.
    let mut store = MockStore::new();
    let mut a = make_shape(1);
    a.parents = vec![ParentRef {
        id: ShapeId::new(2),
        role: None,
        reason: None,
    }];
    let b = make_shape(2);
    store.insert(NodeType::Shape, 1, &a);
    store.insert(NodeType::Shape, 2, &b);

    let issues = validate(&store, None).unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i.invariant == "INV-009" && i.node_id == "1" && i.message.contains("parent")),
        "expected INV-009 parent-without-child violation, got {issues:?}"
    );
}

#[test]
fn validate_detects_missing_amendment_in_log() {
    // Amendment A targets shape S, but S.amendment_log is empty.
    let mut store = MockStore::new();
    let shape = make_shape(1);
    let amendment = make_amendment(10, vec![1]);
    store.insert(NodeType::Shape, 1, &shape);
    store.insert(NodeType::Amendment, 10, &amendment);

    let issues = validate(&store, None).unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i.invariant == "INV-019" && i.node_type == "amendment" && i.node_id == "10"),
        "expected INV-019 missing-log violation, got {issues:?}"
    );
}

#[test]
fn validate_detects_orphan_entry_in_amendment_log() {
    // Shape lists amendment A in amendment_log, but A does not target S.
    let mut store = MockStore::new();
    let mut shape = make_shape(1);
    shape.amendment_log = vec![AmendmentId::new(10)];
    let amendment = make_amendment(10, vec![]); // targets nothing (also fires INV-007)
    store.insert(NodeType::Shape, 1, &shape);
    store.insert(NodeType::Amendment, 10, &amendment);

    let issues = validate(&store, None).unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i.invariant == "INV-019" && i.node_type == "shape" && i.node_id == "1"),
        "expected INV-019 orphan-log-entry violation, got {issues:?}"
    );
}

#[test]
fn validate_accepts_reciprocal_amendment_log() {
    let mut store = MockStore::new();
    let mut shape = make_shape(1);
    shape.amendment_log = vec![AmendmentId::new(10)];
    let amendment = make_amendment(10, vec![1]);
    store.insert(NodeType::Shape, 1, &shape);
    store.insert(NodeType::Amendment, 10, &amendment);

    let issues = validate(&store, None).unwrap();
    assert!(
        !issues.iter().any(|i| i.invariant == "INV-019"),
        "expected no INV-019 issues, got {issues:?}"
    );
}
