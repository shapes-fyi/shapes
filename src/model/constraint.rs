use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    AmendmentId, ConstraintId, ProfileId,
    common::{Evidence, Intent, ParentRef, Provenance, Realization, Status},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub id: ConstraintId,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub rule: String,
    pub enforcement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub status: Status,
    pub intent: Intent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realization: Option<Vec<Realization>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<Evidence>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Vec<Provenance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amendment_log: Option<Vec<AmendmentId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<ParentRef<ConstraintId>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ConstraintChildRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintChildRef {
    pub constraint: ConstraintRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConstraintRef {
    Id(ConstraintId),
    Inline(Box<Constraint>),
}
