//! Profile-related helpers for the `create` subcommands: loading the
//! active profile, choosing the active scaffolding template, and
//! validating user-provided `kind` values against the profile's allow
//! list.

use anyhow::{Context, Result, anyhow};

use crate::error::CreateError;
use crate::model::profile::FieldSection;
use crate::model::{NodeType, Profile};
use crate::store::{FileStore, NodeStore};
use crate::templates::{self, Template, TemplateKind};

/// Loads the profile with the given `id`, mapping a missing profile
/// to [`CreateError::ProfileNotFound`].
pub fn load_profile(store: &impl NodeStore, profile_id: u64) -> Result<Profile, CreateError> {
    store
        .load::<Profile>(NodeType::Profile, profile_id)
        .map_err(|_| CreateError::ProfileNotFound { id: profile_id })
}

/// Validates that `kind` is in the profile's allowed-kinds list for
/// the given `node_type_str` (`"shape"` or `"constraint"`). Profiles
/// without a kinds list always accept the kind.
pub fn validate_kind_against_profile(
    profile: &Profile,
    node_type_str: &str,
    kind: &str,
) -> Result<(), CreateError> {
    let fields = match &profile.fields {
        Some(f) => f,
        None => return Ok(()),
    };
    let section: &FieldSection = match node_type_str {
        "shape" => match &fields.shape {
            Some(s) => s,
            None => return Ok(()),
        },
        "constraint" => match &fields.constraint {
            Some(s) => s,
            None => return Ok(()),
        },
        _ => return Ok(()),
    };
    if let Some(ref group) = section.intent
        && !group.kinds.is_empty()
        && !group.kinds.iter().any(|k| k.name == kind)
    {
        let allowed: Vec<&str> = group.kinds.iter().map(|k| k.name.as_str()).collect();
        return Err(CreateError::InvalidKind {
            kind: kind.to_owned(),
            profile_id: profile.id.get(),
            allowed: allowed.join(", "),
        });
    }
    Ok(())
}

/// Resolves which template to use for a scaffold call.
///
/// A per-call `--template` always wins; otherwise the active template
/// is read from the store's `meta.yaml`. Both a missing `meta.yaml`
/// and an unknown template name surface as hard errors — the store
/// is expected to carry a valid `template:` field at all times.
pub fn resolve_template(
    store: &FileStore,
    override_kind: Option<TemplateKind>,
) -> Result<&'static Template> {
    if let Some(k) = override_kind {
        return Ok(k.template());
    }
    let meta = store
        .read_meta()
        .context("failed to read .shapes/meta.yaml")?;
    templates::resolve(&meta.template)
        .ok_or_else(|| anyhow!("unknown template '{}' in .shapes/meta.yaml", meta.template))
}
