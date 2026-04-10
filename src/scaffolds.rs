//! Scaffold templates for `shapes init --ci` and `shapes init --hooks`.
//!
//! Each scaffold writes a single file with sensible defaults. If the
//! target file already exists, a warning is printed to stderr and the
//! file is left untouched — no error, no overwrite.

use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;

use anyhow::{Context, Result};

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
      - uses: shapes-fyi/shapes/.github/actions/shapes-validate@main
        with:
          require-shapes-changes: 'false'
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

/// Scaffold `.github/workflows/shapes.yml` in the given directory.
///
/// Creates `.github/workflows/` if it does not exist. Skips with a
/// warning if the workflow file is already present.
pub(crate) fn scaffold_github_actions(dir: &Path) -> Result<()> {
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
pub(crate) fn scaffold_prek_hooks(dir: &Path) -> Result<()> {
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
