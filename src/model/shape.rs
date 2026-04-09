//! The [`Shape`] node — the primary "what to build" type in the graph.
//!
//! A `Shape` carries intent, lifecycle status, parent/child links into
//! the shape DAG, the constraints that govern it, and realization /
//! evidence / provenance bindings to its concrete deliverables.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    AmendmentId, ConstraintId, Evidence, Intent, ParentRef, ProfileId, Provenance, Realization,
    ShapeId, Status,
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
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub predecessors: Vec<ShapeId>,
    pub status: Status,
    pub intent: Intent,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub constraints: Vec<ConstraintId>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub realization: Vec<Realization>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub evidence: Vec<Evidence>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub provenance: Vec<Provenance>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub amendment_log: Vec<AmendmentId>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub parents: Vec<ParentRef<ShapeId>>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub children: Vec<ShapeChildRef>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

impl Shape {
    /// Returns the IDs of every parent recorded in `parents`.
    #[must_use]
    pub fn parent_ids(&self) -> Vec<ShapeId> {
        self.parents.iter().map(|p| p.id).collect()
    }

    /// Returns the IDs of every direct child, resolving inline children
    /// to their stored ID.
    #[must_use]
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

impl super::traits::GraphNode for Shape {
    type Id = ShapeId;

    fn raw_id(&self) -> u64 {
        self.id.get()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn status(&self) -> &Status {
        &self.status
    }

    fn intent(&self) -> &Intent {
        &self.intent
    }
}

impl super::traits::DagNode for Shape {
    fn parent_ids(&self) -> Vec<ShapeId> {
        self.parent_ids()
    }

    fn child_ids(&self) -> Vec<ShapeId> {
        self.child_ids()
    }

    fn constraint_ids(&self) -> Vec<super::ConstraintId> {
        self.constraints.clone()
    }

    fn profile_id(&self) -> Option<super::ProfileId> {
        self.profile
    }

    fn realization(&self) -> &[super::Realization] {
        &self.realization
    }

    fn evidence(&self) -> &[super::Evidence] {
        &self.evidence
    }

    fn provenance(&self) -> &[super::Provenance] {
        &self.provenance
    }

    fn metadata(&self) -> &std::collections::BTreeMap<String, serde_yml::Value> {
        &self.metadata
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
