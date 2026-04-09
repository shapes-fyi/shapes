//! Domain types for the four shapes-graph node kinds plus their shared
//! value objects.
//!
//! This module is wiring only — every type lives in a focused sibling
//! file. The submodule layout is:
//!
//! - [`ids`] — newtype identifiers (`ShapeId`, `ConstraintId`, ...)
//! - [`node_type`] — the runtime [`NodeType`] discriminator
//! - [`status`] — lifecycle [`Status`] enum and its custom serde
//! - [`intent`] — the [`Intent`] open map
//! - [`refs`] — generic [`ParentRef`]
//! - [`bindings`] — [`bindings::Binding`], [`Realization`], [`Evidence`], [`Provenance`]
//! - [`serde_helpers`] — shared serde utilities
//! - [`shape`], [`constraint`], [`amendment`], [`profile`] — node types

pub mod amendment;
pub mod bindings;
pub mod constraint;
pub mod ids;
pub mod intent;
pub mod node_type;
pub mod profile;
pub mod refs;
pub mod serde_helpers;
pub mod shape;
pub mod status;
pub mod traits;

pub use amendment::{Amendment, AmendmentTargets, InitiatedBy, InitiatedType, VersionImpact};
pub use bindings::{Evidence, Provenance, Realization};
pub use constraint::{Constraint, Enforcement};
pub use ids::{AmendmentId, ConstraintId, NodeId, ProfileId, ShapeId};
pub use intent::Intent;
pub use node_type::NodeType;
pub use profile::{FieldGroup, Profile};
pub use refs::ParentRef;
pub use shape::Shape;
pub use status::Status;
pub use traits::{DagNode, GraphNode};
