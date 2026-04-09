//! The [`Constraint`] node — a strict, falsifiable rule the project
//! must satisfy.
//!
//! Constraints form their own DAG (independent of the shape DAG) and
//! are linked from shapes via `constraints: [id, ...]`. They inherit
//! down the constraint DAG and are looked up via
//! `shapes query constraints <shape-id>`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use clap::ValueEnum;

use super::{
    AmendmentId, ConstraintId, Evidence, Intent, ParentRef, ProfileId, Provenance, Realization,
    Status,
};

/// Whether a constraint is enforced by humans (`manual`) or by an
/// automated check (`machine`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Enforcement {
    /// Verified by code review or human inspection.
    Manual,
    /// Verified by an automated check (lint, test, CI rule).
    Machine,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub id: ConstraintId,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub rule: String,
    pub enforcement: Enforcement,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub status: Status,
    pub intent: Intent,
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
    pub parents: Vec<ParentRef<ConstraintId>>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub children: Vec<ConstraintChildRef>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

impl Constraint {
    /// Returns the IDs of every parent recorded in `parents`.
    #[must_use]
    pub fn parent_ids(&self) -> Vec<ConstraintId> {
        self.parents.iter().map(|p| p.id).collect()
    }

    /// Returns the IDs of every direct child, resolving inline children
    /// to their stored ID.
    #[must_use]
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

impl super::traits::GraphNode for Constraint {
    type Id = ConstraintId;

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

    fn amendment_log(&self) -> &[super::AmendmentId] {
        &self.amendment_log
    }
}

impl super::traits::DagNode for Constraint {
    fn parent_ids(&self) -> Vec<ConstraintId> {
        self.parent_ids()
    }

    fn child_ids(&self) -> Vec<ConstraintId> {
        self.child_ids()
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

    fn tree_label(&self) -> String {
        format!(
            "constraint:{} {} [{}] kind={}",
            self.id,
            self.name,
            self.status.name(),
            self.kind,
        )
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
