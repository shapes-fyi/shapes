//! Integration tests for `shapes init --ci` and `shapes init --hooks`.
//!
//! Each test runs `shapes init` with scaffold flags in a fresh tempdir
//! and asserts the expected files are created (or skipped when they
//! already exist).

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

/// Run `shapes init` in a fresh tempdir with the given extra args.
fn init_with(args: &[&str]) -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut cmd = Command::cargo_bin("shapes").unwrap();
    cmd.current_dir(dir.path()).arg("init");
    for arg in args {
        cmd.arg(arg);
    }
    cmd.assert().success();
    dir
}

#[test]
fn init_without_flags_creates_no_extras() {
    let dir = init_with(&[]);
    assert!(dir.path().join(".shapes").is_dir());
    assert!(!dir.path().join(".github").exists());
    assert!(!dir.path().join("prek.toml").exists());
}

mod scaffold_snapshots {
    use super::*;

    #[test]
    fn ci_workflow_snapshot() {
        let dir = init_with(&["--ci"]);
        let workflow = dir.path().join(".github/workflows/shapes.yml");
        assert!(workflow.is_file(), "workflow file should exist");
        let content = fs::read_to_string(&workflow).unwrap();
        insta::assert_snapshot!(content);
    }

    #[test]
    fn prek_toml_snapshot() {
        let dir = init_with(&["--hooks"]);
        let prek = dir.path().join("prek.toml");
        assert!(prek.is_file(), "prek.toml should exist");
        let content = fs::read_to_string(&prek).unwrap();
        insta::assert_snapshot!(content);
    }
}

#[test]
fn init_with_both_flags() {
    let dir = init_with(&["--ci", "--hooks"]);
    assert!(dir.path().join(".github/workflows/shapes.yml").is_file());
    assert!(dir.path().join("prek.toml").is_file());
}

#[test]
fn init_ci_skips_existing_workflow() {
    let dir = tempfile::tempdir().expect("tempdir");
    let workflows = dir.path().join(".github/workflows");
    fs::create_dir_all(&workflows).unwrap();
    let target = workflows.join("shapes.yml");
    fs::write(&target, "# existing workflow\n").unwrap();

    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--ci"])
        .assert()
        .success()
        .stderr(predicates::str::contains("Skipped"));

    // File should be unchanged.
    let content = fs::read_to_string(&target).unwrap();
    assert_eq!(content, "# existing workflow\n");
}

#[test]
fn init_hooks_skips_existing_prek() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("prek.toml");
    fs::write(&target, "# existing config\n").unwrap();

    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--hooks"])
        .assert()
        .success()
        .stderr(predicates::str::contains("Skipped"));

    // File should be unchanged.
    let content = fs::read_to_string(&target).unwrap();
    assert_eq!(content, "# existing config\n");
}

#[test]
fn scaffold_hooks_on_existing_store() {
    // First init creates .shapes/.
    let dir = init_with(&[]);
    assert!(dir.path().join(".shapes").is_dir());
    assert!(!dir.path().join("prek.toml").exists());

    // Second init with --hooks succeeds and creates prek.toml.
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--hooks"])
        .assert()
        .success();
    assert!(dir.path().join("prek.toml").is_file());
}

#[test]
fn scaffold_ci_on_existing_store() {
    let dir = init_with(&[]);
    assert!(!dir.path().join(".github").exists());

    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--ci"])
        .assert()
        .success();
    assert!(dir.path().join(".github/workflows/shapes.yml").is_file());
}

#[test]
fn bare_init_on_existing_store_still_fails() {
    let dir = init_with(&[]);

    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
}

mod help_snapshots {
    use assert_cmd::Command;

    #[test]
    fn init_help_output() {
        let cmd = Command::cargo_bin("shapes")
            .unwrap()
            .args(["init", "--help"])
            .output()
            .unwrap();
        let stdout = String::from_utf8(cmd.stdout).unwrap();
        insta::assert_snapshot!(stdout);
    }
}
