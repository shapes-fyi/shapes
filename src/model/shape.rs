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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predecessors: Option<Vec<ShapeId>>,
    pub status: Status,
    pub intent: Intent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Vec<ConstraintId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realization: Option<Vec<Realization>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<Evidence>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Vec<Provenance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amendment_log: Option<Vec<AmendmentId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<ParentRef<ShapeId>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ShapeChildRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
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
