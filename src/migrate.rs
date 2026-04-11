//! Schema migration engine for `.shapes/` stores.
//!
//! A migration is a self-contained function that transforms YAML files
//! in place, reports the files it changed, and records follow-up
//! action items for the user. [`run_migrations`] chains migrations
//! sequentially from the store's current `meta.yaml` version to
//! [`crate::store::CURRENT_STORE_VERSION`], bumping `meta.yaml` after
//! each successful step so a crashed run can be resumed.
//!
//! Adding a new version bump:
//!
//! 1. Write a `fn migrate_0_N_to_0_N_plus_1(store: &FileStore) -> Result<MigrationResult>`
//!    that reads the relevant YAML files as [`serde_yaml_ng::Value`]
//!    (not typed structs — typed structs often reject old schemas),
//!    applies the transformation, writes back, and returns the list of
//!    changed files plus any action items.
//! 2. Append one entry to [`registry`].
//!
//! The engine takes care of chaining, version bumping, crash recovery,
//! and aggregating results across steps.

use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use semver::Version;

use crate::model::NodeType;
use crate::store::{CURRENT_STORE_VERSION, FileStore};

/// Aggregated outcome of one or more migration steps.
///
/// `changed_files` lists every YAML file the migration touched, and
/// `action_items` surfaces follow-up tasks the user should address
/// after the migration completes (e.g. filling in placeholder
/// archival reasons).
#[derive(Debug, Default)]
pub struct MigrationResult {
    /// Files that were modified by the migration, in traversal order.
    pub changed_files: Vec<PathBuf>,
    /// Follow-up actions the user should take after migration.
    pub action_items: Vec<String>,
}

/// A single version-to-version migration step.
struct Migration {
    /// Source version this step upgrades from.
    from: Version,
    /// Target version this step upgrades to.
    to: Version,
    /// Transformation function invoked by [`run_migrations`].
    run: fn(&FileStore) -> Result<MigrationResult>,
}

/// Returns the registered migration steps, in version order.
///
/// To add a new migration: write a function matching the
/// `fn(&FileStore) -> Result<MigrationResult>` signature and append a
/// new [`Migration`] entry. The runner finds the matching step by
/// `from` version, so the order here is for readability only.
fn registry() -> Vec<Migration> {
    vec![Migration {
        from: Version::new(0, 1, 0),
        to: Version::new(0, 2, 0),
        run: migrate_0_1_to_0_2,
    }]
}

/// Runs every applicable migration step against `store`, chaining from
/// the current `meta.yaml` version up to [`CURRENT_STORE_VERSION`].
///
/// After each step succeeds, `meta.yaml` is bumped to that step's
/// target version so an interrupted migration can be resumed by
/// re-invoking the command. If the on-disk version is strictly newer
/// than what the CLI knows about, the migration aborts rather than
/// attempting to downgrade.
pub fn run_migrations(store: &FileStore) -> Result<MigrationResult> {
    let mut meta = store.read_meta()?;
    let mut aggregate = MigrationResult::default();

    while meta.version != CURRENT_STORE_VERSION {
        if meta.version > CURRENT_STORE_VERSION {
            bail!(
                "store version {} is newer than this CLI supports ({}). \
                 Upgrade the `shapes` binary instead of downgrading the store.",
                meta.version,
                CURRENT_STORE_VERSION,
            );
        }

        let step = registry()
            .into_iter()
            .find(|m| m.from == meta.version)
            .ok_or_else(|| {
                anyhow!(
                    "unknown store version {}. This CLI knows how to migrate to {} \
                     but has no registered step from {}.",
                    meta.version,
                    CURRENT_STORE_VERSION,
                    meta.version,
                )
            })?;

        let result = (step.run)(store)?;

        meta.version = step.to;
        store.write_meta(&meta)?;

        aggregate.changed_files.extend(result.changed_files);
        aggregate.action_items.extend(result.action_items);
    }

    Ok(aggregate)
}

/// Migrates amendment files from the 0.1.0 boolean `archived: true`
/// format to the 0.2.0 structured `archived: { reason: "..." }` format.
///
/// Inserts a placeholder reason string for each converted file and
/// records an action item asking the user to review and replace
/// those placeholders.
fn migrate_0_1_to_0_2(store: &FileStore) -> Result<MigrationResult> {
    const PLACEHOLDER_REASON: &str = "Migrated from legacy boolean field — please update";

    let mut result = MigrationResult::default();
    let files = store.yaml_files(NodeType::Amendment)?;

    for path in files {
        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow!("failed to read {}: {}", path.display(), e))?;

        let mut doc: serde_yaml_ng::Value = serde_yaml_ng::from_str(&content)
            .map_err(|e| anyhow!("failed to parse {}: {}", path.display(), e))?;

        let Some(mapping) = doc.as_mapping_mut() else {
            continue;
        };

        let archived_key = serde_yaml_ng::Value::String("archived".into());
        let Some(archived_val) = mapping.get(&archived_key).cloned() else {
            continue;
        };

        let changed = match archived_val {
            serde_yaml_ng::Value::Bool(true) => {
                let mut detail = serde_yaml_ng::Mapping::new();
                detail.insert(
                    serde_yaml_ng::Value::String("reason".into()),
                    serde_yaml_ng::Value::String(PLACEHOLDER_REASON.into()),
                );
                mapping.insert(archived_key, serde_yaml_ng::Value::Mapping(detail));
                true
            }
            serde_yaml_ng::Value::Bool(false) => {
                mapping.remove(&archived_key);
                true
            }
            serde_yaml_ng::Value::Mapping(_) => false,
            other => bail!(
                "unexpected `archived` value in {}: {:?} — expected bool or mapping",
                path.display(),
                other
            ),
        };

        if !changed {
            continue;
        }

        let yaml = serde_yaml_ng::to_string(&doc)
            .map_err(|e| anyhow!("failed to serialize {}: {}", path.display(), e))?;
        fs::write(&path, yaml)
            .map_err(|e| anyhow!("failed to write {}: {}", path.display(), e))?;

        result.changed_files.push(path);
    }

    if !result.changed_files.is_empty() {
        result.action_items.push(
            "Review and update the `archived.reason` field on each migrated amendment — \
             a placeholder string was inserted and should be replaced with a real \
             explanation of why the amendment was archived."
                .into(),
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_contiguous_and_terminates_at_current_version() {
        let steps = registry();
        assert!(!steps.is_empty(), "registry must have at least one step");

        for window in steps.windows(2) {
            assert_eq!(
                window[0].to, window[1].from,
                "migration chain must be contiguous: {} -> {} but next step starts at {}",
                window[0].from, window[0].to, window[1].from
            );
        }

        let last = steps.last().expect("registry is non-empty");
        assert_eq!(
            last.to, CURRENT_STORE_VERSION,
            "last migration step must terminate at CURRENT_STORE_VERSION"
        );
    }

    #[test]
    fn registry_steps_only_upgrade_forward() {
        for step in registry() {
            assert!(
                step.from < step.to,
                "migration step {} -> {} must move forward",
                step.from,
                step.to,
            );
        }
    }
}
