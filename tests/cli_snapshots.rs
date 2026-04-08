//! Full-surface insta snapshot coverage for every CLI read command.
//!
//! Realizes constraint 30 (Snapshot Tests for CLI Output). Each test
//! runs the real `shapes` binary against a deterministic tempdir
//! fixture seeded via `shapes create --from -` and snapshots either
//! stdout or stderr (never both in the same snapshot, to keep diffs
//! scoped). The clean `seed_rich_fixture` is shared across all read
//! commands; the dirty fixture for `validate_snapshots` is a separate
//! helper that produces exactly three invariant violations.
//!
//! Stream choices are documented at each test and trace back to
//! `src/commands/validate.rs:20-36` and `src/commands/ci_check.rs:584-598`.

use assert_cmd::Command;
use tempfile::TempDir;

/// Spin up a fresh tempdir and run `shapes init --kit <kit>`.
fn fresh_store(kit: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--kit", kit])
        .assert()
        .success();
    dir
}

/// Build a `shapes` command rooted in `dir`.
fn shapes_in(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("shapes").unwrap();
    cmd.current_dir(dir.path());
    cmd
}

/// Pipe a yaml blob into `shapes create <type> --from -`. The caller's
/// `id:` field is required by the deserializer but is overwritten by
/// the store with the next auto-assigned id — so use `id: 0` as a
/// harmless placeholder.
fn create_from_stdin(dir: &TempDir, node_type: &str, yaml: &str) {
    shapes_in(dir)
        .args(["create", node_type, "--from", "-"])
        .write_stdin(yaml.to_string())
        .assert()
        .success();
}

/// Seed a tempdir with a deterministic clean graph:
///   shape:1 Parent  <-- constraint:1, constraint:2; child shape:2; amendment_log:[1]
///   shape:2 Child   <-- parent shape:1
///   constraint:1 ParentRule  <-- child constraint:2; amendment_log:[1]
///   constraint:2 ChildRule   <-- parent constraint:1
///   amendment:1 fixture-edit --> targets shape:1 + constraint:1
///
/// Final state passes `shapes validate` clean.
fn seed_rich_fixture() -> TempDir {
    let dir = fresh_store("software");

    create_from_stdin(
        &dir,
        "shape",
        r#"id: 0
name: Parent
description: Fixture parent shape used by CLI snapshot tests.
profile: 1
status: proposed
intent:
  kind: module
  summary: fixture parent
  source: ai
  goals: exist as a parent shape for the snapshot fixture
  rationale: gives the seed fixture a deterministic shape-DAG parent
constraints:
  - 1
  - 2
children:
  - shape: 2
    role: feature
amendment_log:
  - 1
"#,
    );

    create_from_stdin(
        &dir,
        "shape",
        r#"id: 0
name: Child
description: Fixture child shape used by CLI snapshot tests.
profile: 1
status: proposed
intent:
  kind: feature
  summary: fixture child
  source: ai
  goals: exist as a child shape for the snapshot fixture
  rationale: gives the seed fixture a deterministic shape-DAG child
parents:
  - id: 1
    role: component
"#,
    );

    create_from_stdin(
        &dir,
        "constraint",
        r#"id: 0
name: ParentRule
description: Fixture parent constraint used by CLI snapshot tests.
kind: invariant
rule: Fixture rule; not enforced by any code, exists solely to populate the constraint DAG for snapshot tests.
enforcement: manual
profile: 1
status: proposed
intent:
  kind: invariant
  summary: fixture parent rule
  source: ai
  rationale: gives the seed fixture a deterministic constraint-DAG parent
children:
  - constraint: 2
amendment_log:
  - 1
"#,
    );

    create_from_stdin(
        &dir,
        "constraint",
        r#"id: 0
name: ChildRule
description: Fixture child constraint used by CLI snapshot tests.
kind: invariant
rule: Fixture rule; not enforced by any code, exists solely to populate the constraint DAG for snapshot tests.
enforcement: manual
profile: 1
status: proposed
intent:
  kind: invariant
  summary: fixture child rule
  source: ai
  rationale: gives the seed fixture a deterministic constraint-DAG child
parents:
  - id: 1
"#,
    );

    create_from_stdin(
        &dir,
        "amendment",
        r#"id: 0
name: fixture-edit
description: Fixture amendment used by CLI snapshot tests.
targets:
  shape_ids:
    - 1
  constraint_ids:
    - 1
status: proposed
intent:
  kind: amendment
  summary: fixture amendment targeting shape 1 and constraint 1
  source: ai
initiated_by:
  type: ai
"#,
    );

    // Verify the fixture is clean before returning it so any drift in
    // fixture authoring surfaces as a loud failure rather than a
    // mysterious snapshot mismatch downstream.
    shapes_in(&dir).arg("validate").assert().success();

    dir
}

/// Seed a tempdir with exactly three invariant violations:
///   INV-003 — shape:2 references non-existent constraint 999
///   INV-009 — shape:1 lists shape 2 as parent, but shape 2 has no matching child
///   INV-019 — amendment:1 targets shape 1, but shape 1.amendment_log is empty
fn seed_dirty_fixture() -> TempDir {
    let dir = fresh_store("software");

    create_from_stdin(
        &dir,
        "shape",
        r#"id: 0
name: Alpha
description: Dirty fixture shape for INV-009 reverse direction.
profile: 1
status: proposed
intent:
  kind: module
  summary: alpha
  source: ai
  goals: exist
  rationale: dirty seed — INV-009 reverse direction
parents:
  - id: 2
    role: component
"#,
    );

    create_from_stdin(
        &dir,
        "shape",
        r#"id: 0
name: Beta
description: Dirty fixture shape for INV-003.
profile: 1
status: proposed
intent:
  kind: module
  summary: beta
  source: ai
  goals: exist
  rationale: dirty seed — INV-003 dangling constraint
constraints:
  - 999
"#,
    );

    create_from_stdin(
        &dir,
        "amendment",
        r#"id: 0
name: fixture-orphan
description: Dirty fixture amendment for INV-019.
targets:
  shape_ids:
    - 1
status: proposed
intent:
  kind: amendment
  summary: orphan amendment whose target does not list it in amendment_log
  source: ai
initiated_by:
  type: ai
"#,
    );

    dir
}

/// Capture stdout from `cmd` and return it as a `String`, asserting
/// exit success.
fn stdout_of(cmd: &mut Command) -> String {
    let out = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(out).expect("stdout is utf-8")
}

/// Capture stderr from `cmd` and return it as a `String`, asserting
/// exit success.
fn stderr_of(cmd: &mut Command) -> String {
    let out = cmd.assert().success().get_output().stderr.clone();
    String::from_utf8(out).expect("stderr is utf-8")
}

/// Capture stderr from `cmd` expecting a failure exit (validation).
/// Used for dirty validate / ci-check snapshots.
fn stderr_of_failure(cmd: &mut Command, code: i32) -> String {
    let out = cmd
        .assert()
        .failure()
        .code(code)
        .get_output()
        .stderr
        .clone();
    String::from_utf8(out).expect("stderr is utf-8")
}

/// Capture stdout from `cmd` expecting a failure exit.
fn stdout_of_failure(cmd: &mut Command, code: i32) -> String {
    let out = cmd
        .assert()
        .failure()
        .code(code)
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).expect("stdout is utf-8")
}

mod get_snapshots {
    use super::*;

    #[test]
    fn get_shape_yaml() {
        let dir = seed_rich_fixture();
        let yaml = stdout_of(shapes_in(&dir).args(["get", "shape", "1"]));
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn get_shape_json() {
        let dir = seed_rich_fixture();
        let json = stdout_of(shapes_in(&dir).args(["get", "shape", "1", "--format", "json"]));
        insta::assert_snapshot!(json);
    }

    #[test]
    fn get_constraint_yaml() {
        let dir = seed_rich_fixture();
        let yaml = stdout_of(shapes_in(&dir).args(["get", "constraint", "1"]));
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn get_amendment_yaml() {
        let dir = seed_rich_fixture();
        let yaml = stdout_of(shapes_in(&dir).args(["get", "amendment", "1"]));
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn get_profile_yaml() {
        let dir = seed_rich_fixture();
        let yaml = stdout_of(shapes_in(&dir).args(["get", "profile", "1"]));
        insta::assert_snapshot!(yaml);
    }
}

mod list_snapshots {
    use super::*;

    #[test]
    fn list_all() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).arg("list"));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn list_all_json() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["list", "--format", "json"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn list_shapes_only() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["list", "shape"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn list_by_kind() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["list", "shape", "--kind", "feature"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn list_by_status() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["list", "--status", "proposed"]));
        insta::assert_snapshot!(out);
    }
}

mod tree_snapshots {
    use super::*;

    #[test]
    fn tree_shape() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["tree", "shape"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn tree_constraint() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["tree", "constraint"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn tree_shape_rooted() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["tree", "shape", "--root", "1"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn tree_shape_depth_1() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["tree", "shape", "--root", "1", "--depth", "1"]));
        insta::assert_snapshot!(out);
    }
}

mod query_snapshots {
    use super::*;

    #[test]
    fn query_ancestors_yaml() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["query", "ancestors", "shape", "2"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn query_ancestors_json() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args([
            "query",
            "ancestors",
            "shape",
            "2",
            "--format",
            "json",
        ]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn query_descendants_yaml() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["query", "descendants", "shape", "1"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn query_constraints_yaml() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["query", "constraints", "2"]));
        insta::assert_snapshot!(out);
    }

    #[test]
    fn query_shapes_for_constraint_yaml() {
        let dir = seed_rich_fixture();
        let out = stdout_of(shapes_in(&dir).args(["query", "shapes-for-constraint", "2"]));
        insta::assert_snapshot!(out);
    }
}

mod validate_snapshots {
    use super::*;

    /// Clean yaml output: `eprintln!("No issues found.")` on stderr;
    /// stdout stays empty.
    #[test]
    fn validate_clean_yaml() {
        let dir = seed_rich_fixture();
        let stderr = stderr_of(shapes_in(&dir).arg("validate"));
        insta::assert_snapshot!(stderr);
    }

    /// Clean json output: `println!("[]")` on stdout; stderr stays empty.
    #[test]
    fn validate_clean_json() {
        let dir = seed_rich_fixture();
        let stdout = stdout_of(shapes_in(&dir).args(["validate", "--format", "json"]));
        insta::assert_snapshot!(stdout);
    }

    /// Dirty yaml: three invariant rows + summary line on stderr.
    #[test]
    fn validate_dirty_yaml() {
        let dir = seed_dirty_fixture();
        let stderr = stderr_of_failure(shapes_in(&dir).arg("validate"), 2);
        insta::assert_snapshot!(stderr);
    }

    /// Dirty json: `ValidationIssue[]` array on stdout; summary line on
    /// stderr is covered by `validate_dirty_yaml`.
    #[test]
    fn validate_dirty_json() {
        let dir = seed_dirty_fixture();
        let stdout = stdout_of_failure(shapes_in(&dir).args(["validate", "--format", "json"]), 2);
        insta::assert_snapshot!(stdout);
    }
}
