//! The [`NodeType`] enum and its helpers.
//!
//! `NodeType` is the runtime tag that names which of the four node kinds
//! a generic operation is targeting (shape, constraint, amendment,
//! profile). It is used by the CLI for dispatch and by the file store to
//! find the right on-disk subdirectory.

use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Discriminator over the four node kinds in the shapes graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// A [`Shape`](super::Shape) node.
    Shape,
    /// A [`Constraint`](super::Constraint) node.
    Constraint,
    /// An [`Amendment`](super::Amendment) node.
    Amendment,
    /// A [`Profile`](super::Profile) node.
    Profile,
}

impl NodeType {
    /// Returns the on-disk directory name (under `.shapes/`) where nodes
    /// of this type are stored.
    #[must_use]
    pub fn dir_name(&self) -> &'static str {
        match self {
            NodeType::Shape => "shapes",
            NodeType::Constraint => "constraints",
            NodeType::Amendment => "amendments",
            NodeType::Profile => "profiles",
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            NodeType::Shape => "shape",
            NodeType::Constraint => "constraint",
            NodeType::Amendment => "amendment",
            NodeType::Profile => "profile",
        })
    }
}
