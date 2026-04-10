//! `shapes amendment archive|unarchive` — toggles the display-only
//! `archived` flag on an amendment. This is the **only** permitted
//! mutation of a canonical amendment: CI-003 (modified-amendment-
//! immutability) allows diffs whose sole field delta is `archived`.
//!
//! Archival is not deletion. The YAML file stays on disk, validation
//! and CI checks still see it, and `shapes list --archived` / `shapes
//! get <parent> --archived` bring it back into view.

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::shared::{open_store, output};
use crate::model::{Amendment, Archived, NodeType};
use crate::store::NodeStore;

/// Sets `archived` on amendment `id` with an optional reason and writes
/// it back. A no-op on already-archived amendments (still rewrites the
/// file so the on-disk representation is canonical).
pub fn archive(id: u64, reason: Option<String>, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    let mut amendment: Amendment = store.load(NodeType::Amendment, id)?;
    amendment.archived = match reason {
        Some(r) => Archived::yes_with_reason(r),
        None => Archived::yes(),
    };
    store.save(NodeType::Amendment, id, &amendment)?;
    output(&amendment, format)
}

/// Clears the archived flag on amendment `id`, bringing the amendment
/// back into default listings.
pub fn unarchive(id: u64, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    let mut amendment: Amendment = store.load(NodeType::Amendment, id)?;
    amendment.archived = Archived::No;
    store.save(NodeType::Amendment, id, &amendment)?;
    output(&amendment, format)
}
