//! The lifecycle [`Status`] enum and its custom serde implementation.
//!
//! Every node in the shapes graph carries a `Status`. Statuses serialize
//! either as a bare YAML string (`canonical`) when no detail is attached,
//! or as a single-key map (`{canonical: {reason: "..."}}`) when the
//! author wants to record `reason`, `uris`, `successors`, or `metadata`.
//! Both forms round-trip through serde via the hand-rolled `Serialize`
//! and `Deserialize` impls in this file.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::serde_helpers::null_to_default;

/// Lifecycle state of a graph node.
///
/// Three progressive states (`proposed` → `promoted` → `canonical`) plus
/// four terminal states (`rejected`, `superseded`, `abandoned`,
/// `reverted`). Direct edits are allowed while `proposed`; promoted and
/// canonical changes require Amendments.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// Initial draft state — direct edits allowed.
    Proposed(StatusDetail),
    /// Reviewed but not yet ratified.
    Promoted(StatusDetail),
    /// Ratified state — changes require an Amendment.
    Canonical(StatusDetail),
    /// Terminally rejected.
    Rejected(TerminalDetail),
    /// Replaced by a successor node.
    Superseded(TerminalDetail),
    /// Abandoned without a replacement.
    Abandoned(TerminalDetail),
    /// Reverted after promotion or canonicalization.
    Reverted(TerminalDetail),
}

impl Status {
    /// Returns the lower-case status name as it appears on disk.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Status::Proposed(_) => "proposed",
            Status::Promoted(_) => "promoted",
            Status::Canonical(_) => "canonical",
            Status::Rejected(_) => "rejected",
            Status::Superseded(_) => "superseded",
            Status::Abandoned(_) => "abandoned",
            Status::Reverted(_) => "reverted",
        }
    }

    /// Constructs a fresh `proposed` status with default detail.
    #[must_use]
    pub fn proposed() -> Self {
        Status::Proposed(StatusDetail::default())
    }

    /// Constructs a fresh `canonical` status with default detail.
    #[must_use]
    pub fn canonical() -> Self {
        Status::Canonical(StatusDetail::default())
    }
}

/// Optional metadata attached to a progressive status.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StatusDetail {
    /// Free-form reason explaining the transition into this state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// External URIs documenting the transition.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub uris: Vec<String>,
    /// Open metadata bag for transition-specific facts.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

/// Optional metadata attached to a terminal status, plus successor IDs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TerminalDetail {
    /// Free-form reason explaining the terminal transition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// External URIs documenting the transition.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub uris: Vec<String>,
    /// Successor node IDs (same type as the owning node). For shapes
    /// these are `ShapeId` values; for constraints, `ConstraintId`
    /// values.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub successors: Vec<u64>,
    /// Open metadata bag for transition-specific facts.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

impl Serialize for Status {
    /// Emits a bare string when the detail is the default value, or a
    /// single-key map (`{name: detail}`) otherwise.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let name = self.name();
        match self {
            Status::Proposed(d) | Status::Promoted(d) | Status::Canonical(d) => {
                if *d == StatusDetail::default() {
                    serializer.serialize_str(name)
                } else {
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry(name, d)?;
                    map.end()
                }
            }
            Status::Rejected(d)
            | Status::Superseded(d)
            | Status::Abandoned(d)
            | Status::Reverted(d) => {
                if *d == TerminalDetail::default() {
                    serializer.serialize_str(name)
                } else {
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry(name, d)?;
                    map.end()
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for Status {
    /// Accepts either a bare string or a single-key map.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StatusVisitor)
    }
}

struct StatusVisitor;

impl StatusVisitor {
    fn from_str_progressive(name: &str) -> Option<Status> {
        match name {
            "proposed" => Some(Status::Proposed(StatusDetail::default())),
            "promoted" => Some(Status::Promoted(StatusDetail::default())),
            "canonical" => Some(Status::Canonical(StatusDetail::default())),
            _ => None,
        }
    }

    fn from_str_terminal(name: &str) -> Option<Status> {
        match name {
            "rejected" => Some(Status::Rejected(TerminalDetail::default())),
            "superseded" => Some(Status::Superseded(TerminalDetail::default())),
            "abandoned" => Some(Status::Abandoned(TerminalDetail::default())),
            "reverted" => Some(Status::Reverted(TerminalDetail::default())),
            _ => None,
        }
    }
}

impl<'de> Visitor<'de> for StatusVisitor {
    type Value = Status;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a status string or a single-key map like {canonical: {reason: ...}}")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Status, E> {
        Self::from_str_progressive(v)
            .or_else(|| Self::from_str_terminal(v))
            .ok_or_else(|| de::Error::unknown_variant(v, VALID_STATUSES))
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Status, M::Error> {
        let key: String = map
            .next_key()?
            .ok_or_else(|| de::Error::custom("expected a status key"))?;

        let status = match key.as_str() {
            "proposed" => Status::Proposed(map.next_value()?),
            "promoted" => Status::Promoted(map.next_value()?),
            "canonical" => Status::Canonical(map.next_value()?),
            "rejected" => Status::Rejected(map.next_value()?),
            "superseded" => Status::Superseded(map.next_value()?),
            "abandoned" => Status::Abandoned(map.next_value()?),
            "reverted" => Status::Reverted(map.next_value()?),
            other => return Err(de::Error::unknown_variant(other, VALID_STATUSES)),
        };

        Ok(status)
    }
}

const VALID_STATUSES: &[&str] = &[
    "proposed",
    "promoted",
    "canonical",
    "rejected",
    "superseded",
    "abandoned",
    "reverted",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip_bare_string() {
        let status = Status::proposed();
        let yaml = serde_yml::to_string(&status).unwrap();
        assert_eq!(yaml.trim(), "proposed");
        let parsed: Status = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(parsed, status);
    }

    #[test]
    fn status_roundtrip_with_detail() {
        let status = Status::Canonical(StatusDetail {
            reason: Some("passed review".into()),
            uris: vec![],
            metadata: BTreeMap::from([(
                "date".into(),
                serde_yml::Value::String("2025-12-01".into()),
            )]),
        });
        let yaml = serde_yml::to_string(&status).unwrap();
        assert!(yaml.contains("canonical"));
        assert!(yaml.contains("passed review"));
        let parsed: Status = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(parsed, status);
    }

    #[test]
    fn status_roundtrip_terminal() {
        let status = Status::Superseded(TerminalDetail {
            reason: Some("replaced".into()),
            successors: vec![5, 6],
            uris: vec![],
            metadata: BTreeMap::new(),
        });
        let yaml = serde_yml::to_string(&status).unwrap();
        assert!(yaml.contains("superseded"));
        let parsed: Status = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(parsed, status);
    }
}
