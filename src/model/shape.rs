use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    AmendmentId, ConstraintId, ProfileId, ShapeId,
    common::{Evidence, Intent, ParentRef, Provenance, Realization, Status},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub id: ShapeId,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub predecessors: Vec<ShapeId>,
    pub status: Status,
    pub intent: Intent,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<ConstraintId>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub realization: Vec<Realization>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<Provenance>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub amendment_log: Vec<AmendmentId>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<ParentRef<ShapeId>>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ShapeChildRef>,
    #[serde(default, deserialize_with = "crate::model::common::null_to_default", skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

impl Shape {
    pub fn parent_ids(&self) -> Vec<ShapeId> {
        self.parents.iter().map(|p| p.id).collect()
    }

    pub fn child_ids(&self) -> Vec<ShapeId> {
        self.children
            .iter()
            .map(|c| match &c.shape {
                ShapeRef::Id(id) => *id,
                ShapeRef::Inline(s) => s.id,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeChildRef {
    pub shape: ShapeRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShapeRef {
    Id(ShapeId),
    Inline(Box<Shape>),
}
