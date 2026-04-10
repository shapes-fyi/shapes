//! The [`Intent`] open map carried by every shape, constraint, and
//! amendment.
//!
//! `Intent` has three required fields (`kind`, `summary`, `source`) plus
//! an open `extra` map of profile-defined fields. The `extra` map is
//! flattened on serialization so on-disk YAML stays flat.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::serde_helpers::null_to_default;

/// Per-node intent payload — the "why" behind a shape, constraint, or
/// amendment.
///
/// `kind`, `summary`, and `source` are always present. Profiles may
/// declare additional fields (e.g. `goals`, `rationale`, `non_goals`)
/// which round-trip through `extra`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    /// Domain label such as `feature`, `module`, or `invariant`.
    pub kind: String,
    /// One-line human-readable description.
    pub summary: String,
    /// Origin of the intent — typically `human`, `ai`, or `system`.
    pub source: serde_yaml_ng::Value,
    /// External URIs documenting the intent.
    #[serde(
        default,
        deserialize_with = "null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub uris: Vec<String>,
    /// Profile-defined extra fields, flattened into the parent map.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml_ng::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_roundtrip_with_extra_fields() {
        let intent = Intent {
            kind: "feature".into(),
            summary: "Add auth".into(),
            source: serde_yaml_ng::Value::String("human".into()),
            uris: vec![],
            extra: BTreeMap::from([(
                "goals".into(),
                serde_yaml_ng::Value::Sequence(vec![serde_yaml_ng::Value::String(
                    "SSO support".into(),
                )]),
            )]),
        };
        let yaml = serde_yaml_ng::to_string(&intent).unwrap();
        assert!(yaml.contains("goals"));
        let parsed: Intent = serde_yaml_ng::from_str(&yaml).unwrap();
        assert_eq!(parsed, intent);
    }
}
