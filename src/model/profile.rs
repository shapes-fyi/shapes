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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Vec<Provenance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<ProfileFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versioning: Option<Versioning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amendment_rules: Option<AmendmentRules>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amendment_log: Option<Vec<AmendmentId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lifecycle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<StatusDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gates: Option<Vec<Gate>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preconditions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postconditions: Option<Vec<String>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<FieldDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<FieldDef>>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmendmentRules {
    pub application: String,
}
