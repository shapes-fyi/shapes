pub mod amendment;
pub mod common;
pub mod constraint;
pub mod profile;
pub mod shape;

pub type ShapeId = u64;
pub type ConstraintId = u64;
pub type AmendmentId = u64;
pub type ProfileId = u64;

pub use amendment::{Amendment, AmendmentTargets, InitiatedBy};
pub use common::{Intent, Status};
pub use constraint::Constraint;
pub use profile::{AmendmentRules, FieldGroup, Profile};
pub use shape::Shape;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Shape,
    Constraint,
    Amendment,
    Profile,
}

impl NodeType {
    pub fn dir_name(&self) -> &'static str {
        match self {
            NodeType::Shape => "shapes",
            NodeType::Constraint => "constraints",
            NodeType::Amendment => "amendments",
            NodeType::Profile => "profiles",
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NodeType::Shape => "shape",
            NodeType::Constraint => "constraint",
            NodeType::Amendment => "amendment",
            NodeType::Profile => "profile",
        })
    }
}
