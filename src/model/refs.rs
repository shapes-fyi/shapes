//! Generic [`ParentRef`] used by every node type to record parent links
//! in the shape and constraint DAGs.

use serde::{Deserialize, Serialize};

/// A typed reference to a parent node in a DAG.
///
/// `Id` is generic so each node type re-uses this struct with its own
/// newtype identifier (`ShapeId`, `ConstraintId`, etc.) without losing
/// compile-time type safety.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentRef<Id> {
    /// The parent's identifier.
    pub id: Id,
    /// Optional role label describing how this parent relates to the
    /// child (e.g. `component`, `pattern`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Optional reason explaining the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
