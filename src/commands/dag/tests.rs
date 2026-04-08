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
