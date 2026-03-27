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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Vec<ConstraintId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realization: Option<Vec<Realization>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<Evidence>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Vec<Provenance>>,
    pub initiated_by: InitiatedBy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmendmentTargets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_ids: Option<Vec<ShapeId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_ids: Option<Vec<ConstraintId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_ids: Option<Vec<ProfileId>>,
}

impl AmendmentTargets {
    pub fn is_empty(&self) -> bool {
        self.shape_ids.as_ref().is_none_or(|v| v.is_empty())
            && self.constraint_ids.as_ref().is_none_or(|v| v.is_empty())
            && self.profile_ids.as_ref().is_none_or(|v| v.is_empty())
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
