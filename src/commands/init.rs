//! `shapes init` — bootstraps a new `.shapes/` directory and seeds the
//! active Profile from the chosen starter kit.

use std::env;

use anyhow::Result;

use crate::store::FileStore;
use crate::templates::KitKind;

/// Initializes a new `.shapes/` store, seeding profile id 1 from the
/// chosen starter kit and recording it as the active profile.
pub fn init(kind: KitKind) -> Result<()> {
    let dir = env::current_dir()?;
    let kit = kind.kit();
    FileStore::init(&dir, kit)?;
    eprintln!(
        "Initialized .shapes/ in {} (kit: {} — {}; active profile id 1 seeded)",
        dir.display(),
        kit.name,
        kit.description,
    );
    Ok(())
}
