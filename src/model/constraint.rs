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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub realization: Vec<Realization>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<Provenance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amendment_log: Vec<AmendmentId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<ParentRef<ConstraintId>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ConstraintChildRef>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

impl Constraint {
    pub fn parent_ids(&self) -> Vec<ConstraintId> {
        self.parents.iter().map(|p| p.id).collect()
    }

    pub fn child_ids(&self) -> Vec<ConstraintId> {
        self.children
            .iter()
            .map(|ch| match &ch.constraint {
                ConstraintRef::Id(id) => *id,
                ConstraintRef::Inline(c) => c.id,
            })
            .collect()
    }
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
