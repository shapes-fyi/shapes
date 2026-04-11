//! Integration tests for `shapes migrate`.
//!
//! Covers the happy path (legacy boolean `archived: true` rewritten to
//! the structured object form), idempotency (re-running on an already
//! current store), the version gate in `shared::open_store` (regular
//! commands fail fast on outdated stores), and mixed-state stores
//! where some files are already migrated.

use std::fs;
use std::path::{Path, PathBuf};

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

fn shapes_in(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("shapes").unwrap();
    cmd.current_dir(dir.path());
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

/// Writes a legacy-format amendment YAML (with `archived: true`) into
/// the store at the given ID and returns the resulting file path.
fn write_legacy_archived_amendment(dir: &Path, id: u64) -> PathBuf {
    let path = dir
        .join(".shapes")
        .join("amendments")
        .join(format!("{id}-legacy.yaml"));
    let yaml = format!(
        r#"id: {id}
name: legacy amendment
description: fixture — legacy archived boolean
targets:
  shape_ids:
  - 1
status: proposed
intent:
  kind: amendment
  summary: fixture
  source: ai
initiated_by:
  type: ai
archived: true
"#
    );
    fs::write(&path, yaml).expect("write legacy amendment");
    path
}

/// Writes an already-current amendment (structured `archived.reason`)
/// at the given ID and returns the file path.
fn write_current_archived_amendment(dir: &Path, id: u64) -> PathBuf {
    let path = dir
        .join(".shapes")
        .join("amendments")
        .join(format!("{id}-current.yaml"));
    let yaml = format!(
        r#"id: {id}
name: current amendment
description: fixture — already migrated
targets:
  shape_ids:
  - 1
status: proposed
intent:
  kind: amendment
  summary: fixture
  source: ai
initiated_by:
  type: ai
archived:
  reason: already current
"#
    );
    fs::write(&path, yaml).expect("write current amendment");
    path
}

/// Writes an amendment with no `archived` field at all.
fn write_unarchived_amendment(dir: &Path, id: u64) -> PathBuf {
    let path = dir
        .join(".shapes")
        .join("amendments")
        .join(format!("{id}-unarchived.yaml"));
    let yaml = format!(
        r#"id: {id}
name: unarchived amendment
description: fixture — no archived field
targets:
  shape_ids:
  - 1
status: proposed
intent:
  kind: amendment
  summary: fixture
  source: ai
initiated_by:
  type: ai
"#
    );
    fs::write(&path, yaml).expect("write unarchived amendment");
    path
}

#[test]
fn migrate_rewrites_legacy_boolean_archived_to_structured_form() {
    let dir = fresh_store();
    downgrade_meta_to_0_1(dir.path());
    let amendment_path = write_legacy_archived_amendment(dir.path(), 100);

    let assert = shapes_in(&dir).arg("migrate").assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();

    assert!(
        stderr.contains("Migrating store from version 0.1.0 to 0.2.0"),
        "stderr missing migration header: {stderr}"
    );
    assert!(
        stderr.contains("Modified files"),
        "stderr missing modified-files section: {stderr}"
    );
    assert!(
        stderr.contains("100-legacy.yaml"),
        "stderr should list the migrated file path: {stderr}"
    );
    assert!(
        stderr.contains("Action items"),
        "stderr missing action-items section: {stderr}"
    );
    assert!(
        stderr.contains("archived.reason"),
        "action item should mention the `archived.reason` follow-up: {stderr}"
    );
    assert!(
        stderr.contains("Migration complete. Store is now at version 0.2.0"),
        "stderr missing completion summary: {stderr}"
    );

    let rewritten = fs::read_to_string(&amendment_path).expect("read migrated amendment");
    assert!(
        !rewritten.contains("archived: true"),
        "legacy boolean still present: {rewritten}"
    );
    assert!(
        rewritten.contains("archived:\n  reason:"),
        "expected structured archived form: {rewritten}"
    );
    assert!(
        rewritten.contains("Migrated from legacy boolean field"),
        "placeholder reason missing: {rewritten}"
    );

    let meta = fs::read_to_string(dir.path().join(".shapes").join("meta.yaml")).unwrap();
    assert!(
        meta.contains("version: 0.2.0"),
        "meta version not bumped: {meta}"
    );
}

#[test]
fn migrate_is_idempotent_on_already_current_store() {
    let dir = fresh_store();

    let assert = shapes_in(&dir).arg("migrate").assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();

    assert!(
        stderr.contains("Store is already at version 0.2.0"),
        "stderr should report no-op: {stderr}"
    );
    assert!(
        stderr.contains("nothing to migrate"),
        "stderr should say nothing to migrate: {stderr}"
    );
}

#[test]
fn version_gate_blocks_regular_commands_on_outdated_store() {
    let dir = fresh_store();
    downgrade_meta_to_0_1(dir.path());

    let assert = shapes_in(&dir).arg("list").assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();

    assert!(
        stderr.contains("store is at version 0.1.0"),
        "error should name the on-disk version: {stderr}"
    );
    assert!(
        stderr.contains("expects 0.2.0"),
        "error should name the expected version: {stderr}"
    );
    assert!(
        stderr.contains("shapes migrate"),
        "error must point at the migrate command: {stderr}"
    );
}

#[test]
fn migrate_handles_mixed_state_stores() {
    let dir = fresh_store();
    downgrade_meta_to_0_1(dir.path());

    let legacy = write_legacy_archived_amendment(dir.path(), 200);
    let current = write_current_archived_amendment(dir.path(), 201);
    let unarchived = write_unarchived_amendment(dir.path(), 202);

    let current_before = fs::read_to_string(&current).unwrap();
    let unarchived_before = fs::read_to_string(&unarchived).unwrap();

    let assert = shapes_in(&dir).arg("migrate").assert().success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();

    assert!(
        stderr.contains("200-legacy.yaml"),
        "legacy file should be listed as modified: {stderr}"
    );
    assert!(
        !stderr.contains("201-current.yaml"),
        "already-current file should NOT be modified: {stderr}"
    );
    assert!(
        !stderr.contains("202-unarchived.yaml"),
        "unarchived file should NOT be modified: {stderr}"
    );

    let legacy_after = fs::read_to_string(&legacy).unwrap();
    assert!(legacy_after.contains("reason:"));
    assert!(!legacy_after.contains("archived: true"));

    assert_eq!(
        fs::read_to_string(&current).unwrap(),
        current_before,
        "already-current file must be left byte-identical"
    );
    assert_eq!(
        fs::read_to_string(&unarchived).unwrap(),
        unarchived_before,
        "unarchived file must be left byte-identical"
    );
}

#[test]
fn migrate_followed_by_list_works_end_to_end() {
    let dir = fresh_store();
    downgrade_meta_to_0_1(dir.path());
    write_legacy_archived_amendment(dir.path(), 300);

    shapes_in(&dir).arg("migrate").assert().success();
    shapes_in(&dir).arg("list").assert().success();
}

/// Insta snapshot coverage for `shapes migrate` stderr per constraint
/// 30. Locks the exact wording of the happy-path migration report and
/// the idempotent no-op path. Any copy drift in `src/commands/migrate.rs`
/// surfaces as a reviewable `.snap` diff. Update with `cargo insta
/// review` after an intentional change.
mod migrate_snapshots {
    use super::*;

    /// Strips both the canonical and non-canonical tempdir prefixes
    /// from `stderr` so snapshots are stable across platforms (macOS
    /// symlinks `/var` to `/private/var`, which sometimes defeats
    /// `strip_prefix` in the command handler and leaves an absolute
    /// path in the "Modified files" block).
    fn normalize(stderr: &str, dir: &Path) -> String {
        let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        stderr
            .replace(canonical.to_str().unwrap_or_default(), "[TMPDIR]")
            .replace(dir.to_str().unwrap_or_default(), "[TMPDIR]")
    }

    /// Happy path: legacy `archived: true` rewritten to the structured
    /// form. Locks the full stderr layout — header, modified-files
    /// block, action-items block, and completion summary.
    #[test]
    fn migrate_rewrites_legacy_archived_stderr() {
        let dir = fresh_store();
        downgrade_meta_to_0_1(dir.path());
        write_legacy_archived_amendment(dir.path(), 100);

        let assert = shapes_in(&dir).arg("migrate").assert().success();
        let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
        let stderr = normalize(&stderr, dir.path());
        insta::assert_snapshot!(stderr);
    }

    /// No-op path: store is already at the current version. Locks the
    /// "Store is already at version … — nothing to migrate." line.
    #[test]
    fn migrate_noop_stderr() {
        let dir = fresh_store();

        let assert = shapes_in(&dir).arg("migrate").assert().success();
        let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
        let stderr = normalize(&stderr, dir.path());
        insta::assert_snapshot!(stderr);
    }
}
