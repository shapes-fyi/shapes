//! Integration tests for `shapes preflight`.
//!
//! Covers the two schema-drift branches (outdated store → MIGRATION
//! NEEDED; newer store → STORE AHEAD OF CLI), the current-store happy
//! path (tree prints, no warning), and the no-`.shapes/` case (init
//! guidance prints). Every invocation sets `SHAPES_SKIP_UPDATE_CHECK=1`
//! so the `UPDATE AVAILABLE` line never reaches captured output — see
//! `src/commands/preflight.rs::check_latest_version`.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

/// Creates a fresh `.shapes/` store via `shapes init --kit software`.
fn fresh_store() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--kit", "software"])
        .assert()
        .success();
    dir
}

/// Returns a `shapes` [`Command`] rooted in `dir` with the
/// `SHAPES_SKIP_UPDATE_CHECK` env var set, so preflight's optional
/// `UPDATE AVAILABLE` line never appears in captured output.
fn shapes_in(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("shapes").unwrap();
    cmd.current_dir(dir.path());
    cmd.env("SHAPES_SKIP_UPDATE_CHECK", "1");
    cmd
}

/// Rewrites `.shapes/meta.yaml` in-place so its `version` field reads
/// `0.1.0`, simulating a store written by an older CLI release.
fn downgrade_meta_to_0_1(dir: &Path) {
    let path = dir.join(".shapes").join("meta.yaml");
    let content = fs::read_to_string(&path).expect("read meta.yaml");
    let rewritten = content.replace("version: 0.2.0", "version: 0.1.0");
    assert_ne!(
        content, rewritten,
        "expected fresh store to start at version 0.2.0"
    );
    fs::write(&path, rewritten).expect("write meta.yaml");
}

/// Rewrites `.shapes/meta.yaml` in-place so its `version` field reads
/// `999.0.0`, simulating a store written by a future CLI release that
/// the current binary does not understand.
fn bump_meta_to_future(dir: &Path) {
    let path = dir.join(".shapes").join("meta.yaml");
    let content = fs::read_to_string(&path).expect("read meta.yaml");
    let rewritten = content.replace("version: 0.2.0", "version: 999.0.0");
    assert_ne!(
        content, rewritten,
        "expected fresh store to start at version 0.2.0"
    );
    fs::write(&path, rewritten).expect("write meta.yaml");
}

#[test]
fn preflight_on_current_store_prints_tree() {
    let dir = fresh_store();
    let assert = shapes_in(&dir).arg("preflight").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();

    assert!(
        stdout.contains("Shapes CLI v"),
        "stdout should include the version line: {stdout}"
    );
    assert!(
        !stdout.contains("MIGRATION NEEDED"),
        "current store should NOT emit a drift warning: {stdout}"
    );
    assert!(
        !stdout.contains("STORE AHEAD OF CLI"),
        "current store should NOT emit a drift warning: {stdout}"
    );
    // A freshly initialized software-kit store has no seeded shapes,
    // so `tree()` in `src/commands/dag/tree.rs` emits its empty-DAG
    // notice on *stderr* via `eprintln!`. Asserting on stderr here
    // proves preflight reached the tree branch at all — i.e. it was
    // not short-circuited by a drift warning.
    assert!(
        stderr.contains("No shape nodes found."),
        "expected empty-tree notice on stderr: {stderr}"
    );
}

#[test]
fn preflight_without_shapes_dir_prints_init_guidance() {
    let dir = tempfile::tempdir().expect("tempdir");
    let assert = Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .env("SHAPES_SKIP_UPDATE_CHECK", "1")
        .arg("preflight")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();

    assert!(
        stdout.contains("No .shapes/ directory found"),
        "stdout should include the no-shapes-dir guidance: {stdout}"
    );
    assert!(
        stdout.contains("shapes-init"),
        "stdout should point at shapes-init: {stdout}"
    );
    assert!(
        !stdout.contains("MIGRATION NEEDED"),
        "no-shapes-dir path should NOT warn about drift: {stdout}"
    );
}

/// Insta snapshot coverage for the two schema-drift branches per
/// constraint 30. Locks the exact wording of the `MIGRATION NEEDED`
/// and `STORE AHEAD OF CLI` warnings so any copy drift in
/// `src/commands/preflight.rs` surfaces as a reviewable `.snap` diff.
/// Update with `cargo insta review` after an intentional change.
///
/// The `Shapes CLI vX.Y.Z` header line is filtered out so release
/// version bumps in `Cargo.toml` do not invalidate the snapshots —
/// only the drift-warning body is load-bearing for these tests.
mod preflight_snapshots {
    use super::*;

    /// Replaces the `Shapes CLI vX.Y.Z` header line with a placeholder
    /// so release version bumps in `Cargo.toml` do not invalidate the
    /// snapshot — only the drift-warning body is load-bearing for
    /// these tests. Done with manual string rewriting to avoid needing
    /// the `filters` feature on the `insta` crate, matching the
    /// `cli_migrate::migrate_snapshots::normalize` pattern.
    fn normalize(stdout: &str) -> String {
        let mut lines: Vec<String> = stdout
            .lines()
            .map(|line| {
                if line.starts_with("Shapes CLI v") {
                    "Shapes CLI v[X.Y.Z]".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect();
        if stdout.ends_with('\n') {
            lines.push(String::new());
        }
        lines.join("\n")
    }

    /// Outdated store: on-disk 0.1.0, CLI expects 0.2.0 → migration
    /// prompt. Locks the exact `MIGRATION NEEDED:` line so the
    /// imperative-first phrasing can't silently drift.
    #[test]
    fn preflight_outdated_stdout() {
        let dir = fresh_store();
        downgrade_meta_to_0_1(dir.path());
        let assert = shapes_in(&dir).arg("preflight").assert().success();
        let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
        insta::assert_snapshot!(normalize(&stdout));
    }

    /// Newer store: on-disk 999.0.0, CLI only supports 0.2.0 →
    /// upgrade-CLI prompt. Only reachable when the user has downgraded
    /// their installed `shapes` binary or opened a store written by a
    /// future release.
    #[test]
    fn preflight_newer_stdout() {
        let dir = fresh_store();
        bump_meta_to_future(dir.path());
        let assert = shapes_in(&dir).arg("preflight").assert().success();
        let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
        insta::assert_snapshot!(normalize(&stdout));
    }
}
