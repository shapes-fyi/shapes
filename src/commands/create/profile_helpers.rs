//! Profile-related helpers for the `create` subcommands: loading the
//! active profile, choosing the active scaffolding template, and
//! validating user-provided `kind` values against the profile's allow
//! list.

use anyhow::Result;

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
/// is read from `meta.yaml`; otherwise the resolver falls back to
/// `software`.
pub fn resolve_template(
    store: &FileStore,
    override_kind: Option<TemplateKind>,
) -> &'static Template {
    if let Some(k) = override_kind {
        return k.template();
    }
    let meta_template = store.read_meta().ok().and_then(|m| m.template);
    templates::resolve(meta_template.as_deref())
}
