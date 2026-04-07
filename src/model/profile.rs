//! The [`Profile`] node — governance configuration that declares which
//! intent fields are required, which kinds are allowed, and how
//! amendments apply to canonical nodes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{AmendmentId, Intent, ProfileId, Provenance, Status};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub status: Status,
    pub intent: Intent,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub provenance: Vec<Provenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<ProfileFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versioning: Option<Versioning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amendment_rules: Option<AmendmentRules>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub amendment_log: Vec<AmendmentId>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

/// Lifecycle definition: the named statuses a node may take and the
/// gates between them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lifecycle {
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub statuses: Vec<StatusDef>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub gates: Vec<Gate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub status_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gate {
    pub from: String,
    pub to: String,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub preconditions: Vec<String>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub postconditions: Vec<String>,
}

/// Declaration of a single profile-defined field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<FieldSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<FieldSection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<FieldGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<FieldGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<FieldGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realization: Option<FieldGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<FieldGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<FieldGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FieldGroup>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldGroup {
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub fields: Vec<FieldDef>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub kinds: Vec<FieldDef>,
    #[serde(
        default,
        deserialize_with = "crate::model::serde_helpers::null_to_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub sources: Vec<FieldDef>,
}

/// Versioning configuration for a profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Versioning {
    pub scheme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump_rules: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmendmentModel {
    Merge,
    Overlay,
    Edition,
    #[serde(rename = "append-only")]
    #[value(name = "append-only")]
    AppendOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmendmentRules {
    pub application: AmendmentModel,
}
