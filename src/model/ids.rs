//! Newtype wrappers around the raw `u64` identifiers used for each node
//! type in the shapes graph.
//!
//! Each `*Id` type is a transparent wrapper that encodes — at the type
//! level — *which* kind of node an identifier belongs to. This makes it
//! impossible to accidentally pass a [`ShapeId`] where a [`ConstraintId`]
//! is expected, while still serializing as a bare integer in YAML and
//! JSON. See constraint 13 (Newtype ID Safety) in `.shapes/`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Identifier for a [`Shape`](super::Shape) node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShapeId(u64);

impl ShapeId {
    /// Wraps a raw `u64` as a `ShapeId`.
    #[must_use]
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw `u64`.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ShapeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ShapeId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u64>().map(Self::new)
    }
}

/// Identifier for a [`Constraint`](super::Constraint) node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConstraintId(u64);

impl ConstraintId {
    /// Wraps a raw `u64` as a `ConstraintId`.
    #[must_use]
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw `u64`.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ConstraintId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ConstraintId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u64>().map(Self::new)
    }
}

/// Identifier for an [`Amendment`](super::Amendment) node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AmendmentId(u64);

impl AmendmentId {
    /// Wraps a raw `u64` as an `AmendmentId`.
    #[must_use]
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw `u64`.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for AmendmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for AmendmentId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u64>().map(Self::new)
    }
}

/// Identifier for a [`Profile`](super::Profile) node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileId(u64);

impl ProfileId {
    /// Wraps a raw `u64` as a `ProfileId`.
    #[must_use]
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw `u64`.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ProfileId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u64>().map(Self::new)
    }
}
