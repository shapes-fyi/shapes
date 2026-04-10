//! `shapes init` — bootstraps a new `.shapes/` directory and seeds the
//! active Profile from the chosen starter kit.
//!
//! Optional scaffolds:
//!
//! - `--ci` writes `.github/workflows/shapes.yml` (GitHub Actions).
//! - `--hooks` writes `prek.toml` and runs `prek install` if available.

use std::env;

use anyhow::Result;

use crate::scaffolds;
use crate::store::FileStore;
use crate::templates::KitKind;

/// Initializes a new `.shapes/` store, seeding profile id 1 from the
/// chosen starter kit and recording it as the active profile.
///
/// When `ci` is `true`, scaffolds a GitHub Actions workflow. When
/// `hooks` is `true`, scaffolds a prek pre-commit config and attempts
/// to run `prek install`.
pub fn init(kind: KitKind, ci: bool, hooks: bool) -> Result<()> {
    let dir = env::current_dir()?;
    let kit = kind.kit();
    FileStore::init(&dir, kit)?;
    eprintln!(
        "Initialized .shapes/ in {} (kit: {} — {}; active profile id 1 seeded)",
        dir.display(),
        kit.name,
        kit.description,
    );

    if ci {
        scaffolds::scaffold_github_actions(&dir)?;
    }
    if hooks {
        scaffolds::scaffold_prek_hooks(&dir)?;
    }

    Ok(())
}
