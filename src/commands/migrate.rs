//! Handler for `shapes migrate` — upgrades a `.shapes/` store from an
//! older schema version to [`crate::store::CURRENT_STORE_VERSION`].
//!
//! This file is intentionally thin: it opens the store, delegates to
//! [`crate::migrate::run_migrations`] for the actual transformation,
//! and formats the result (changed files + follow-up action items)
//! to stderr. All migration logic lives in [`crate::migrate`].

use std::env;

use anyhow::Result;

use crate::migrate::run_migrations;
use crate::store::{CURRENT_STORE_VERSION, FileStore};

/// Runs all pending migrations on the `.shapes/` store in the current
/// working directory and prints a human-readable report to stderr.
///
/// Intentionally bypasses [`super::shared::open_store`] because the
/// whole point of this command is to handle stores whose version does
/// not match [`CURRENT_STORE_VERSION`].
pub fn migrate() -> Result<()> {
    let store = FileStore::open(&env::current_dir()?)?;
    let meta = store.read_meta()?;

    if meta.version == CURRENT_STORE_VERSION {
        eprintln!(
            "Store is already at version {CURRENT_STORE_VERSION} — nothing to migrate."
        );
        return Ok(());
    }

    eprintln!(
        "Migrating store from version {} to {}...",
        meta.version, CURRENT_STORE_VERSION
    );

    let result = run_migrations(&store)?;

    let cwd = env::current_dir().unwrap_or_default();
    if result.changed_files.is_empty() {
        eprintln!();
        eprintln!("No files were modified.");
    } else {
        eprintln!();
        eprintln!("Modified files:");
        for path in &result.changed_files {
            let display = path.strip_prefix(&cwd).unwrap_or(path);
            eprintln!("  {}", display.display());
        }
        eprintln!();
        eprintln!("{} file(s) migrated.", result.changed_files.len());
    }

    if !result.action_items.is_empty() {
        eprintln!();
        eprintln!("Action items:");
        for (i, item) in result.action_items.iter().enumerate() {
            eprintln!("  {}. {item}", i + 1);
        }
    }

    eprintln!();
    eprintln!("Migration complete. Store is now at version {CURRENT_STORE_VERSION}.");
    Ok(())
}
