//! Profile-related helpers for the `create` subcommands: loading the
//! active profile, resolving a per-call override, and validating
//! user-provided `kind` values against the profile's allow list.

use anyhow::{Context, Result};

use crate::error::CreateError;
use crate::model::profile::FieldSection;
use crate::model::{NodeType, Profile, ProfileId};
use crate::store::{FileStore, NodeStore};

/// Resolves the profile governing a `shapes create` call.
///
/// If the caller passed `--profile <id>`, that wins. Otherwise the
/// active profile is read from `meta.yaml::active_profile`. Returns a
/// hard error if the referenced profile does not exist on disk or
/// `meta.yaml` cannot be read — the store is expected to carry a
/// valid active profile at all times.
pub fn resolve_active_profile(store: &FileStore, override_id: Option<u64>) -> Result<Profile> {
    let profile_id = match override_id {
        Some(id) => ProfileId::new(id),
        None => {
            let meta = store
                .read_meta()
                .context("failed to read .shapes/meta.yaml")?;
            meta.active_profile
        }
    };
    store
        .load::<Profile>(NodeType::Profile, profile_id.get())
        .with_context(|| {
            format!(
                "failed to load active profile {} referenced by .shapes/meta.yaml",
                profile_id
            )
        })
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

/// Returns the shape's default `--kind` from its governing profile,
/// falling back to `None` if the profile declares no default.
#[must_use]
pub fn shape_default_kind(profile: &Profile) -> Option<&str> {
    profile
        .fields
        .as_ref()?
        .shape
        .as_ref()?
        .default_kind
        .as_deref()
}

/// Returns the constraint's default `--kind` from its governing
/// profile, falling back to `None` if the profile declares no default.
#[must_use]
pub fn constraint_default_kind(profile: &Profile) -> Option<&str> {
    profile
        .fields
        .as_ref()?
        .constraint
        .as_ref()?
        .default_kind
        .as_deref()
}
