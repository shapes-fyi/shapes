//! Integration tests for `shapes ci-check`.
//!
//! Each test spins up a tempdir, runs `git init`, calls `shapes init`,
//! commits the initial state, then exercises a specific scenario the
//! ci-check command must classify correctly. The base ref for the
//! check is the initial commit; the working tree carries the simulated
//! "PR" changes.

use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;

use assert_cmd::Command;
use tempfile::TempDir;

/// Spin up a fresh tempdir, run `git init`, `shapes init --kit <kit>`,
/// then commit everything as the initial state. Returns the tempdir
/// and the initial commit's full sha.
fn init_git_store(kit: &str) -> (TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    git(dir.path(), &["init", "--quiet", "--initial-branch=main"]);
    // Set a stable user identity so commits work in CI sandboxes.
    git(dir.path(), &["config", "user.email", "ci@test.invalid"]);
    git(dir.path(), &["config", "user.name", "ci-test"]);
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--kit", kit])
        .assert()
        .success();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "--quiet", "-m", "init"]);
    let head = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    (dir, head)
}

/// Build a `shapes` Command rooted in `dir`.
fn shapes_in(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("shapes").unwrap();
    cmd.current_dir(dir.path());
    cmd
}

fn git(dir: &Path, args: &[&str]) {
    let status = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap_or_else(|e| panic!("git {args:?} failed to spawn: {e}"));
    assert!(status.success(), "git {args:?} returned {status}");
}

fn git_capture(dir: &Path, args: &[&str]) -> String {
    let out = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?} failed to spawn: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} returned {} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// CI-001: when --require-shapes-changes is set and the working tree
/// makes no changes under .shapes/, ci-check fires CI-001.
#[test]
fn ci_001_no_shapes_changes_is_flagged() {
    let (dir, base) = init_git_store("software");
    // Touch an unrelated file outside .shapes/ so the diff is not
    // empty overall, only empty under .shapes/.
    fs::write(dir.path().join("README.md"), "noop").unwrap();
    let assert = shapes_in(&dir)
        .args(["ci-check", "--base", &base, "--require-shapes-changes"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("CI-001"),
        "expected CI-001 in stderr: {stderr}"
    );
}

/// CI-001 (negative control): when --require-shapes-changes is NOT
/// set, an empty .shapes/ diff passes.
#[test]
fn ci_001_off_by_default_passes() {
    let (dir, base) = init_git_store("software");
    shapes_in(&dir)
        .args(["ci-check", "--base", &base])
        .assert()
        .success();
}

/// CI-001 positive: when the working tree adds a new shape under
/// .shapes/, the require-shapes-changes check passes.
#[test]
fn ci_001_with_shapes_changes_passes() {
    let (dir, base) = init_git_store("software");
    shapes_in(&dir)
        .args([
            "create",
            "shape",
            "--name",
            "Touched",
            "--kind",
            "feature",
            "--summary",
            "x",
        ])
        .assert()
        .success();
    shapes_in(&dir)
        .args(["ci-check", "--base", &base, "--require-shapes-changes"])
        .assert()
        .success();
}
