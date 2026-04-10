//! `shapes init` — bootstraps a new `.shapes/` directory and seeds the
//! active Profile from the chosen starter kit.
//!
//! Optional scaffolds (each writes a single file with sensible defaults;
//! if the target already exists it is skipped with a warning):
//!
//! - `--ci` writes `.github/workflows/shapes.yml` (GitHub Actions).
//! - `--hooks` writes `prek.toml` and runs `prek install` if prek is on
//!   PATH, activating the git hooks immediately.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;

use anyhow::{Context, Result};

use crate::store::FileStore;
use crate::templates::KitKind;

/// GitHub Actions workflow that runs `shapes validate`, `shapes fmt --check`,
/// and `shapes ci-check` on every PR via the reusable composite action.
const GITHUB_ACTIONS_WORKFLOW: &str = r#"name: Shapes

on:
  pull_request:
  push:
    branches: [main]

jobs:
  shapes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      # Pins to main until shapes-cli cuts v1; will move to a tagged release.
      - uses: shapes-fyi/shapes/.github/actions/shapes-validate@main
"#;

/// prek hook configuration that runs `shapes validate` and
/// `shapes fmt --check` as local system hooks before each commit.
const PREK_CONFIG: &str = r#"[[repos]]
repo = "local"

[[repos.hooks]]
id = "shapes-validate"
name = "Shapes validate"
language = "system"
entry = "shapes validate"
pass_filenames = false
always_run = true

[[repos.hooks]]
id = "shapes-fmt-check"
name = "Shapes format check"
language = "system"
entry = "shapes fmt --check"
pass_filenames = false
always_run = true
"#;

/// Initializes a new `.shapes/` store, seeding profile id 1 from the
/// chosen starter kit and recording it as the active profile.
///
/// When `ci` is `true`, scaffolds a GitHub Actions workflow. When
/// `hooks` is `true`, scaffolds a prek pre-commit config and attempts
/// to run `prek install`.
///
/// Scaffold flags work on already-initialized projects: if `.shapes/`
/// exists and `--ci` or `--hooks` is passed, the store init is skipped
/// and only the requested scaffolds are written.
pub fn init(kind: KitKind, ci: bool, hooks: bool) -> Result<()> {
    let dir = env::current_dir()?;
    let has_scaffolds = ci || hooks;
    let shapes_exists = dir.join(".shapes").is_dir();

    if shapes_exists && !has_scaffolds {
        anyhow::bail!(".shapes/ directory already exists.");
    }

    if !shapes_exists {
        let kit = kind.kit();
        FileStore::init(&dir, kit)?;
        eprintln!(
            "Initialized .shapes/ in {} (kit: {} — {}; active profile id 1 seeded)",
            dir.display(),
            kit.name,
            kit.description,
        );
    }

    if ci {
        scaffold_github_actions(&dir)?;
    }
    if hooks {
        scaffold_prek_hooks(&dir)?;
    }

    Ok(())
}

/// Scaffold `.github/workflows/shapes.yml` in the given directory.
///
/// Creates `.github/workflows/` if it does not exist. Skips with a
/// warning if the workflow file is already present.
fn scaffold_github_actions(dir: &Path) -> Result<()> {
    let workflows_dir = dir.join(".github").join("workflows");
    let target = workflows_dir.join("shapes.yml");

    if target.exists() {
        eprintln!(
            "  Skipped: {} already exists",
            target.strip_prefix(dir).unwrap_or(&target).display()
        );
        return Ok(());
    }

    fs::create_dir_all(&workflows_dir)
        .with_context(|| format!("failed to create {}", workflows_dir.display()))?;
    fs::write(&target, GITHUB_ACTIONS_WORKFLOW)
        .with_context(|| format!("failed to write {}", target.display()))?;

    eprintln!(
        "  Created {}",
        target.strip_prefix(dir).unwrap_or(&target).display()
    );
    Ok(())
}

/// Scaffold `prek.toml` in the given directory and attempt to run
/// `prek install` if prek is on PATH.
///
/// Skips with a warning if `prek.toml` already exists. Detection and
/// execution of `prek install` are best-effort — failures produce
/// warnings, never errors.
fn scaffold_prek_hooks(dir: &Path) -> Result<()> {
    let target = dir.join("prek.toml");

    if target.exists() {
        eprintln!(
            "  Skipped: {} already exists",
            target.strip_prefix(dir).unwrap_or(&target).display()
        );
        return Ok(());
    }

    fs::write(&target, PREK_CONFIG)
        .with_context(|| format!("failed to write {}", target.display()))?;

    eprintln!(
        "  Created {}",
        target.strip_prefix(dir).unwrap_or(&target).display()
    );

    // Best-effort: run `prek install` if available.
    if which::which("prek").is_ok() {
        let status = StdCommand::new("prek")
            .args(["install"])
            .current_dir(dir)
            .status();
        match status {
            Ok(s) if s.success() => eprintln!("  Ran `prek install` — git hooks are active."),
            Ok(s) => eprintln!("  Warning: `prek install` exited with {s}"),
            Err(e) => eprintln!("  Warning: failed to run `prek install`: {e}"),
        }
    } else {
        eprintln!("  prek not found on PATH. Install it to activate hooks:");
        eprintln!("    pip install prek");
        eprintln!("    cargo install --locked prek");
        eprintln!("    brew install prek");
        eprintln!("  Then run: prek install");
    }

    Ok(())
}
