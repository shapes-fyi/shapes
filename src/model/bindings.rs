//! [`Binding`], [`Realization`], [`Evidence`], and [`Provenance`] —
//! the value objects that connect a node to external artifacts.
//!
//! A `Binding` is a (`scheme`, `value`) pair pointing at something
//! outside the graph (typically a repo path). `Realization`, `Evidence`,
//! and `Provenance` group bindings together with type and role metadata.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::serde_helpers::null_to_default;

/// A typed pointer to an artifact outside the graph.
///
/// The `scheme` names the addressing system (e.g. `path`, `url`,
/// `git`); `value` is the address itself; `metadata` carries
/// scheme-specific extra fields like a `summary` describing what the
/// binding points at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    /// Addressing scheme (`path`, `url`, ...).
    pub scheme: String,
    /// Scheme-specific address.
    pub value: String,
    /// Open metadata bag for binding-specific facts.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yaml_ng::Value>,
}

/// A group of bindings that realize a shape (or constraint) as a
/// concrete deliverable, plus a `role` label distinguishing primary
/// realizations from supporting and test ones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Realization {
    /// One or more bindings that together realize the parent node.
    pub bindings: Vec<Binding>,
    /// Role label (`primary`, `supporting`, `test`, ...).
    pub role: String,
}

/// A piece of evidence demonstrating that a constraint is satisfied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// Unique identifier for this evidence record within its parent
    /// node's evidence array. This is a freeform string (e.g.
    /// `test-suite-pass`), NOT a node ID.
    pub id: String,
    /// Evidence kind (`test`, `review`, `metric`, ...).
    #[serde(rename = "type")]
    pub evidence_type: String,
    /// Bindings to the underlying artifacts.
    pub bindings: Vec<Binding>,
    /// Whether this evidence is from a trusted source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted: Option<bool>,
    /// Open metadata bag for evidence-specific facts.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yaml_ng::Value>,
}

/// A provenance record linking a node to a decision-history artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Provenance kind (`discussion`, `review`, `session`, ...).
    #[serde(rename = "type")]
    pub provenance_type: String,
    /// Bindings to the underlying artifacts.
    pub bindings: Vec<Binding>,
    /// Open metadata bag for provenance-specific facts.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yaml_ng::Value>,
}
