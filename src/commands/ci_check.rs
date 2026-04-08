//! `shapes ci-check` — PR-level enforcement on top of `shapes validate`.
//!
//! Designed to run inside a CI job (or locally before committing) and
//! enforce two rules that depend on the **diff** between a base ref and
//! the working tree:
//!
//! - **CI-001**: when invoked with `--require-shapes-changes`, the PR
//!   must touch at least one file under the shapes directory.
//! - **CI-002** (added in a follow-up commit): when a shape, constraint,
//!   or profile that was already in `promoted` or `canonical` state on
//!   the base ref is semantically modified on HEAD, the PR must contain
//!   a new amendment YAML targeting that node.
//! - **CI-003** (added in a follow-up commit): existing amendment files
//!   in the diff must not be modified — amendments are immutable per
//!   constraint:10.
//!
//! Comparison semantics: `base` ref vs the **working tree**, so the same
//! command yields the same answer locally (before `git commit`) and in
//! CI (after `actions/checkout@v4`, where the working tree equals
//! HEAD).

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::OutputFormat;
use crate::commands::dag::{Severity, ValidationIssue};
use crate::error::{CiCheckError, CliError};

/// Runs the PR-level checks against the working tree, comparing each
/// monitored file against its state on `base`.
///
/// `shapes_dir` is the path to the `.shapes/` directory, relative to
/// the current working directory. Defaults are wired up by `main`.
pub fn ci_check(
    base: &str,
    shapes_dir: &Path,
    require_shapes_changes: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    // Fail fast with a friendly message if `git` isn't on PATH —
    // otherwise the first subprocess explodes with a kernel-level
    // ENOENT and the user has to figure out the cause.
    which::which("git").map_err(|_| {
        CliError::Other(anyhow::anyhow!(
            "shapes ci-check requires `git` on PATH (compares the working tree against {base})",
        ))
    })?;

    let mut issues: Vec<ValidationIssue> = Vec::new();

    let changed = changed_paths_under(base, shapes_dir).map_err(CliError::Other)?;
    if require_shapes_changes && changed.is_empty() {
        issues.push(ValidationIssue {
            invariant: "CI-001".into(),
            severity: Severity::Error,
            node_type: "pull-request".into(),
            node_id: "-".into(),
            message: format!(
                "PR has no changes under {} (required by --require-shapes-changes)",
                shapes_dir.display()
            ),
        });
    }

    report(&issues, format)
}

/// Returns the set of paths under `shapes_dir` that differ between
/// `base` and the current working tree. The result includes:
///
/// - tracked files modified, added, deleted, or renamed since `base`
///   (`git diff --name-only <base> -- <shapes_dir>`)
/// - untracked files inside `shapes_dir` (`git ls-files --others
///   --exclude-standard -- <shapes_dir>`)
///
/// Combining both is necessary so the command works locally before a
/// commit (where new files are still untracked) AND in CI after
/// `actions/checkout@v4` (where every PR file is already tracked).
fn changed_paths_under(base: &str, shapes_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();

    // Tracked changes (modified / added / deleted / renamed) vs base.
    let diff = Command::new("git")
        .args(["diff", "--name-only", base, "--"])
        .arg(shapes_dir)
        .output()
        .with_context(|| format!("failed to spawn `git diff` against {base}"))?;
    if !diff.status.success() {
        let stderr = String::from_utf8_lossy(&diff.stderr);
        bail!("git diff failed: {stderr}");
    }
    paths.extend(
        String::from_utf8_lossy(&diff.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from),
    );

    // Untracked files inside shapes_dir, respecting .gitignore.
    let untracked = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard", "--"])
        .arg(shapes_dir)
        .output()
        .with_context(|| "failed to spawn `git ls-files --others`")?;
    if !untracked.status.success() {
        let stderr = String::from_utf8_lossy(&untracked.stderr);
        bail!("git ls-files failed: {stderr}");
    }
    paths.extend(
        String::from_utf8_lossy(&untracked.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from),
    );

    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Emits the issue list in the requested format and converts a
/// non-empty list into the dedicated [`CiCheckError::IssuesFound`]
/// error so the CLI returns exit code 2.
fn report(issues: &[ValidationIssue], format: OutputFormat) -> Result<(), CliError> {
    if issues.is_empty() {
        match format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Yaml => eprintln!("No issues found."),
        }
        return Ok(());
    }
    match format {
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(issues).map_err(|e| CliError::Other(e.into()))?;
            println!("{json}");
        }
        OutputFormat::Yaml => {
            for issue in issues {
                eprintln!("{issue}");
            }
            eprintln!("{} ci-check issue(s) found", issues.len());
        }
    }
    Err(CiCheckError::IssuesFound {
        count: issues.len(),
    }
    .into())
}
