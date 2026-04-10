//! Shared trait abstractions for the shapes-graph node types.
//!
//! Three traits capture the structural relationships between node types
//! at the Rust type level:
//!
//! - [`GraphNode`] — fields shared by all four node types (id, name,
//!   status, intent). Implemented by Shape, Constraint, Amendment, and
//!   Profile.
//! - [`DagNode`] — the DAG-navigable subset shared by Shape and
//!   Constraint only (parent/child links, constraint references, binding
//!   accessors). A subtrait of `GraphNode`.
//!
//! See shape 37 (Type-Level Trait Hierarchy) and constraints 38/39 in
//! `.shapes/`.

use std::collections::BTreeMap;

use serde::de::DeserializeOwned;

use super::bindings::{Evidence, Provenance, Realization};
use super::ids::{ConstraintId, NodeId, ProfileId};
use super::intent::Intent;
use super::status::Status;

/// A node in the shapes graph that carries identity, lifecycle status,
/// and intent.
///
/// All four node types (Shape, Constraint, Amendment, Profile) implement
/// this trait. It provides uniform read access for listing, display,
/// and CI-check code that needs to inspect common fields without knowing
/// the concrete type.
pub trait GraphNode {
    /// The concrete identifier type for this node kind.
    type Id: NodeId;

    /// Returns this node's raw `u64` identifier.
    fn raw_id(&self) -> u64;

    /// Returns the human-readable name.
    fn name(&self) -> &str;

    /// Returns a reference to this node's lifecycle status.
    fn status(&self) -> &Status;

    /// Returns a reference to the intent payload.
    fn intent(&self) -> &Intent;
}

/// A node that participates in one of the two DAGs (shape or
/// constraint).
///
/// Only [`Shape`](super::Shape) and [`Constraint`](super::Constraint)
/// implement this trait — they are the two node types that form
/// independent directed acyclic graphs with parent/child links.
/// [`Amendment`](super::Amendment) and [`Profile`](super::Profile) do
/// not form DAGs and should not implement `DagNode`.
///
/// The associated type `Id` is inherited from [`GraphNode`]
/// and preserves compile-time type safety: `Shape`'s ID is `ShapeId`,
/// `Constraint`'s is `ConstraintId`. This makes the trait not
/// object-safe, which is intentional — all usage is static dispatch.
///
/// See constraint 38 (DagNode Trait for DAG Operations) in `.shapes/`.
pub trait DagNode: GraphNode + DeserializeOwned + Clone {
    /// IDs of parent nodes in this DAG.
    fn parent_ids(&self) -> Vec<Self::Id>;

    /// IDs of child nodes in this DAG.
    fn child_ids(&self) -> Vec<Self::Id>;

    /// IDs of constraints directly attached to this node.
    ///
    /// Shapes return their `constraints` field; Constraints return
    /// the default empty vec (constraints reference other constraints
    /// through parent/child links, not through a separate constraints
    /// field).
    fn constraint_ids(&self) -> Vec<ConstraintId> {
        vec![]
    }

    /// Returns the optional governing profile ID.
    fn profile_id(&self) -> Option<ProfileId>;

    /// Returns the realization bindings.
    fn realization(&self) -> &[Realization];

    /// Returns the evidence bindings.
    fn evidence(&self) -> &[Evidence];

    /// Returns the provenance bindings.
    fn provenance(&self) -> &[Provenance];

    /// Returns the open metadata map.
    fn metadata(&self) -> &BTreeMap<String, serde_yaml_ng::Value>;

    /// Produces a display label for tree printing.
    ///
    /// Default: `"type:id name [status] kind=intent.kind"`.
    /// Constraint overrides to use its top-level `kind` field.
    fn tree_label(&self) -> String {
        format!(
            "{}:{} {} [{}] kind={}",
            Self::Id::NODE_TYPE,
            self.raw_id(),
            self.name(),
            self.status().name(),
            self.intent().kind,
        )
    }
}
