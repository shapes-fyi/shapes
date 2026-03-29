use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    AmendmentId, ConstraintId, ProfileId, ShapeId,
    common::{Evidence, Intent, Provenance, Realization, Status},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Amendment {
    pub id: AmendmentId,
    pub name: String,
    pub description: String,
    pub targets: AmendmentTargets,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_impact: Option<String>,
    pub intent: Intent,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<ConstraintId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub realization: Vec<Realization>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<Provenance>,
    pub initiated_by: InitiatedBy,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmendmentTargets {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shape_ids: Vec<ShapeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraint_ids: Vec<ConstraintId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profile_ids: Vec<ProfileId>,
}

impl AmendmentTargets {
    pub fn is_empty(&self) -> bool {
        self.shape_ids.is_empty()
            && self.constraint_ids.is_empty()
            && self.profile_ids.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitiatedBy {
    #[serde(rename = "type")]
    pub initiated_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}
