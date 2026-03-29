use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    AmendmentId, ProfileId,
    common::{Intent, Provenance, Status},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub status: Status,
    pub intent: Intent,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<Provenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<ProfileFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versioning: Option<Versioning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amendment_rules: Option<AmendmentRules>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amendment_log: Vec<AmendmentId>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_yml::Value>,
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lifecycle {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<StatusDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postconditions: Vec<String>,
}

// ---------------------------------------------------------------------------
// Field declarations
// ---------------------------------------------------------------------------

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<FieldDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<FieldDef>,
}

// ---------------------------------------------------------------------------
// Versioning & amendment rules
// ---------------------------------------------------------------------------

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
