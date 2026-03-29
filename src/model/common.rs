use std::collections::BTreeMap;
use std::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Proposed(StatusDetail),
    Promoted(StatusDetail),
    Canonical(StatusDetail),
    Rejected(TerminalDetail),
    Superseded(TerminalDetail),
    Abandoned(TerminalDetail),
    Reverted(TerminalDetail),
}

impl Status {
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

    pub fn proposed() -> Self {
        Status::Proposed(StatusDetail::default())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StatusDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uris: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TerminalDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uris: Vec<String>,
    /// Successor node IDs (same type as the owning node).
    /// For shapes, these are ShapeId values; for constraints, ConstraintId values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successors: Vec<u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

// Custom Serialize: if detail is default, emit bare string; otherwise tagged map.
impl Serialize for Status {
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

// Custom Deserialize: accept bare string OR single-key map.
impl<'de> Deserialize<'de> for Status {
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

// ---------------------------------------------------------------------------
// Intent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub kind: String,
    pub summary: String,
    pub source: serde_yml::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uris: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yml::Value>,
}

// ---------------------------------------------------------------------------
// ParentRef — generic over ID type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentRef<Id> {
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Bindings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub scheme: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Realization {
    pub bindings: Vec<Binding>,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// Unique identifier for this evidence record within its parent node's
    /// evidence array. This is a freeform string (e.g., "test-suite-pass"),
    /// NOT a node ID.
    pub id: String,
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub bindings: Vec<Binding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted: Option<bool>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(rename = "type")]
    pub provenance_type: String,
    pub bindings: Vec<Binding>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

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

    #[test]
    fn intent_roundtrip_with_extra_fields() {
        let intent = Intent {
            kind: "feature".into(),
            summary: "Add auth".into(),
            source: serde_yml::Value::String("human".into()),
            uris: vec![],
            extra: BTreeMap::from([(
                "goals".into(),
                serde_yml::Value::Sequence(vec![serde_yml::Value::String(
                    "SSO support".into(),
                )]),
            )]),
        };
        let yaml = serde_yml::to_string(&intent).unwrap();
        assert!(yaml.contains("goals"));
        let parsed: Intent = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(parsed, intent);
    }
}
