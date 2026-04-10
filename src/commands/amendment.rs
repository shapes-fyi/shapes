//! `shapes amendment archive|unarchive` — toggles the display-only
//! `archived` field on an amendment. This is the **only** permitted
//! mutation of a canonical amendment: CI-003 (modified-amendment-
//! immutability) allows diffs whose sole field delta is `archived`.
//!
//! Archival is not deletion. The YAML file stays on disk, validation
//! and CI checks still see it, and `shapes list --archived` / `shapes
//! get <parent> --archived` bring it back into view.

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::shared::{open_store, output};
use crate::model::{Amendment, ArchivedDetail, NodeType};
use crate::store::NodeStore;

/// Sets `archived` on amendment `id` with a required reason and writes
/// it back.
pub fn archive(id: u64, reason: String, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    let mut amendment: Amendment = store.load(NodeType::Amendment, id)?;
    amendment.archived = Some(ArchivedDetail { reason });
    store.save(NodeType::Amendment, id, &amendment)?;
    output(&amendment, format)
}

/// Clears the archived field on amendment `id`, bringing the amendment
/// back into default listings.
pub fn unarchive(id: u64, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    let mut amendment: Amendment = store.load(NodeType::Amendment, id)?;
    amendment.archived = None;
    store.save(NodeType::Amendment, id, &amendment)?;
    output(&amendment, format)
}
