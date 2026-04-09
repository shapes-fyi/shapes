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

// ─── CI-002 / CI-003 fixtures ───────────────────────────────────────

/// Returns the absolute path of the single yaml file in
/// `.shapes/<subdir>/` matching `<id>-*.yaml`. Panics if not found —
/// these helpers are used during test setup where the file is known
/// to exist.
fn yaml_path(dir: &TempDir, subdir: &str, id: u64) -> std::path::PathBuf {
    let prefix = format!("{id}-");
    let dir = dir.path().join(".shapes").join(subdir);
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(&prefix) && name_str.ends_with(".yaml") {
            return entry.path();
        }
    }
    panic!(
        "no yaml file with id prefix '{prefix}' under {}",
        dir.display()
    );
}

/// Appends `archived: true` to an amendment yaml.
fn set_archived(path: &std::path::Path) {
    let text = fs::read_to_string(path).unwrap();
    let updated = format!("{text}archived: true\n");
    fs::write(path, updated).unwrap();
}

/// Replaces the `status: ...` line of a node yaml with `status: <new>`.
fn set_status(path: &std::path::Path, new_status: &str) {
    let text = fs::read_to_string(path).unwrap();
    let mut updated = String::new();
    for line in text.lines() {
        if line.starts_with("status:") {
            updated.push_str(&format!("status: {new_status}\n"));
        } else {
            updated.push_str(line);
            updated.push('\n');
        }
    }
    fs::write(path, updated).unwrap();
}

/// Replaces the top-level `name:` line of a node yaml with
/// `name: <new>`. Line-based replace that matches the first line
/// starting with `name:` (no indent), leaving nested `name:` fields
/// inside `fields` / `lifecycle` blocks alone. Panics if no top-
/// level `name:` line is found so a fixture-shape change never
/// silently no-ops.
fn set_top_level_name(path: &std::path::Path, new_name: &str) {
    let text = fs::read_to_string(path).unwrap();
    let mut updated = String::new();
    let mut replaced = false;
    for line in text.lines() {
        if !replaced && line.starts_with("name:") {
            updated.push_str(&format!("name: {new_name}\n"));
            replaced = true;
        } else {
            updated.push_str(line);
            updated.push('\n');
        }
    }
    assert!(
        replaced,
        "expected a top-level `name:` line in {}",
        path.display()
    );
    fs::write(path, updated).unwrap();
}

/// Replaces the `summary: ...` line under `intent:` with `summary:
/// <new>`. Naive but sufficient for these test fixtures because
/// scaffolds put intent.summary on its own line and metadata.summary
/// only appears nested under bindings.
fn set_intent_summary(path: &std::path::Path, new_summary: &str) {
    let text = fs::read_to_string(path).unwrap();
    let mut updated = String::new();
    let mut in_intent = false;
    for line in text.lines() {
        if line.starts_with("intent:") {
            in_intent = true;
            updated.push_str(line);
            updated.push('\n');
            continue;
        }
        // Leave the intent block on the next top-level key.
        if in_intent && !line.starts_with(' ') && !line.is_empty() && line.contains(':') {
            in_intent = false;
        }
        if in_intent && line.trim_start().starts_with("summary:") {
            let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            updated.push_str(&format!("{indent}summary: {new_summary}\n"));
        } else {
            updated.push_str(line);
            updated.push('\n');
        }
    }
    fs::write(path, updated).unwrap();
}

/// Sets up a base commit containing a single shape that has been
/// flipped to `promoted` status. Returns the shape's id and the base
/// commit sha. Subsequent test code mutates the working tree.
fn promoted_shape_on_base(dir: &TempDir, name: &str) -> (u64, String) {
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "create",
            "shape",
            "--name",
            name,
            "--kind",
            "feature",
            "--summary",
            "s",
        ])
        .assert()
        .success();
    let path = yaml_path(dir, "shapes", 1);
    set_status(&path, "promoted");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "--quiet", "-m", "promote shape 1"]);
    let base = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    (1, base)
}

/// Sets up a base commit containing a single constraint flipped to
/// `canonical`. Returns its id and the base sha.
fn canonical_constraint_on_base(dir: &TempDir, name: &str, rule: &str) -> (u64, String) {
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "create",
            "constraint",
            "--name",
            name,
            "--rule",
            rule,
            "--kind",
            "invariant",
            "--enforcement",
            "manual",
            "--summary",
            "s",
        ])
        .assert()
        .success();
    let path = yaml_path(dir, "constraints", 1);
    set_status(&path, "canonical");
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["commit", "--quiet", "-m", "canonicalize constraint 1"],
    );
    let base = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    (1, base)
}

/// Runs ci-check, expects exit 2, returns stderr. The
/// require-shapes-changes flag is left off so CI-001 doesn't pollute
/// the result of these tests, which only care about CI-002/CI-003.
fn ci_check_fails(dir: &TempDir, base: &str) -> String {
    let assert = shapes_in(dir)
        .args(["ci-check", "--base", base])
        .assert()
        .failure()
        .code(2);
    String::from_utf8_lossy(&assert.get_output().stderr).to_string()
}

/// CI-002 (control): a proposed shape modified in the working tree
/// does not fire CI-002 — direct edits are still allowed.
#[test]
fn ci_002_proposed_shape_modification_is_not_flagged() {
    let (dir, _base) = init_git_store("software");
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "create",
            "shape",
            "--name",
            "Drafty",
            "--kind",
            "feature",
            "--summary",
            "s",
        ])
        .assert()
        .success();
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["commit", "--quiet", "-m", "create proposed shape"],
    );
    let base = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    set_intent_summary(&yaml_path(&dir, "shapes", 1), "totally new summary");
    shapes_in(&dir)
        .args(["ci-check", "--base", &base])
        .assert()
        .success();
}

/// CI-002: a promoted shape with intent.summary changed and no
/// amendment fires CI-002.
#[test]
fn ci_002_promoted_shape_intent_change_without_amendment_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = promoted_shape_on_base(&dir, "Promoted");
    set_intent_summary(&yaml_path(&dir, "shapes", id), "rewritten");
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-002") && stderr.contains(&id.to_string()),
        "expected CI-002 for promoted shape: {stderr}"
    );
}

/// CI-002: a canonical constraint with `rule` changed and no
/// amendment fires CI-002.
#[test]
fn ci_002_canonical_constraint_rule_change_without_amendment_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = canonical_constraint_on_base(&dir, "RuleX", "old text");
    let path = yaml_path(&dir, "constraints", id);
    let text = fs::read_to_string(&path).unwrap();
    let updated = text.replacen("old text", "new text completely different", 1);
    fs::write(&path, updated).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-002") && stderr.contains(&id.to_string()),
        "expected CI-002 for canonical constraint: {stderr}"
    );
}

/// CI-002 satisfied: a promoted shape with intent.summary changed AND
/// a new amendment targeting the shape passes.
#[test]
fn ci_002_promoted_shape_change_with_satisfying_amendment_passes() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = promoted_shape_on_base(&dir, "Promoted");
    set_intent_summary(&yaml_path(&dir, "shapes", id), "rewritten");
    shapes_in(&dir)
        .args([
            "create",
            "amendment",
            "--name",
            "fix",
            "--target-shape",
            &id.to_string(),
            "--summary",
            "fixes the promoted shape",
        ])
        .assert()
        .success();
    shapes_in(&dir)
        .args(["ci-check", "--base", &base])
        .assert()
        .success();
}

/// CI-002 (strict): changing realization bindings on a promoted or
/// canonical node fires CI-002. Every edit under realization counts
/// — there is no opt-out flag. Projects that want the edit to land
/// must author an amendment targeting the node.
#[test]
fn ci_002_realization_only_change_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = promoted_shape_on_base(&dir, "Promoted");
    let path = yaml_path(&dir, "shapes", id);
    let text = fs::read_to_string(&path).unwrap();
    let appended = format!(
        "{text}realization:\n  - bindings:\n      - scheme: path\n        value: Cargo.toml\n        metadata:\n          summary: arbitrary realization\n    role: primary\n"
    );
    fs::write(&path, appended).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-002") && stderr.contains(&id.to_string()),
        "expected CI-002 for realization-only change: {stderr}"
    );
}

/// CI-002: changing the shape's `constraints` list (a monitored
/// field) on a promoted shape requires an amendment.
#[test]
fn ci_002_promoted_shape_constraint_list_change_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = promoted_shape_on_base(&dir, "Promoted");
    let path = yaml_path(&dir, "shapes", id);
    let text = fs::read_to_string(&path).unwrap();
    // Inject a `constraints: [99]` block. Constraint 99 doesn't have
    // to exist for the diff comparison — ci-check only inspects
    // structural differences, not constraint resolution.
    let appended = format!("{text}constraints:\n  - 99\n");
    fs::write(&path, appended).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-002") && stderr.contains(&id.to_string()),
        "expected CI-002 for constraint list change: {stderr}"
    );
}

/// CI-002: deleting a promoted shape from disk also fires CI-002 —
/// canonical lineage requires an amendment to record the removal.
#[test]
fn ci_002_promoted_shape_deletion_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = promoted_shape_on_base(&dir, "Promoted");
    let path = yaml_path(&dir, "shapes", id);
    fs::remove_file(&path).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-002") && stderr.contains(&id.to_string()) && stderr.contains("deleted"),
        "expected CI-002 deletion: {stderr}"
    );
}

/// CI-002 negative: a brand-new shape added in the PR with no base
/// version doesn't require an amendment.
#[test]
fn ci_002_new_shape_in_pr_does_not_require_amendment() {
    let (dir, base) = init_git_store("software");
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "create",
            "shape",
            "--name",
            "Brandnew",
            "--kind",
            "feature",
            "--summary",
            "s",
        ])
        .assert()
        .success();
    // Even with --require-shapes-changes set, this passes — CI-002
    // doesn't fire because the shape didn't exist on the base ref.
    shapes_in(&dir)
        .args(["ci-check", "--base", &base, "--require-shapes-changes"])
        .assert()
        .success();
}

/// CI-003: modifying an existing amendment file fires the
/// amendment-immutability check, and the modification does NOT
/// satisfy any pending CI-002.
#[test]
fn ci_003_modified_amendment_fires_and_does_not_satisfy_ci_002() {
    let (dir, _base0) = init_git_store("software");
    // Set up a promoted shape on base, plus an existing amendment that
    // would have satisfied CI-002.
    let (id, _) = promoted_shape_on_base(&dir, "Promoted");
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "create",
            "amendment",
            "--name",
            "PriorFix",
            "--target-shape",
            &id.to_string(),
            "--summary",
            "fixed yesterday",
        ])
        .assert()
        .success();
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["commit", "--quiet", "-m", "ship prior amendment"],
    );
    let base = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    // Now in the working tree: change the promoted shape AND mutate
    // the prior amendment.
    set_intent_summary(&yaml_path(&dir, "shapes", id), "rewritten");
    // Mutate a real field in the amendment yaml so the parsed
    // structure differs from base — comments would be stripped by
    // serde_yml and CI-003 would not fire.
    let amend_path = yaml_path(&dir, "amendments", 1);
    let amend = fs::read_to_string(&amend_path).unwrap();
    let tampered = amend.replacen("PriorFix", "TamperedName", 1);
    assert_ne!(
        amend, tampered,
        "name replacement must actually change the file"
    );
    fs::write(&amend_path, tampered).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(stderr.contains("CI-003"), "expected CI-003 fired: {stderr}");
    assert!(
        stderr.contains("CI-002"),
        "expected CI-002 to still fire because the modified amendment does not satisfy: {stderr}"
    );
}

/// CI-002: modifying the canonical profile seeded by `shapes init`
/// without an amendment fires CI-002. The software starter kit
/// seeds profile id 1 with `canonical` status already, so it is in
/// the amendment-required state on the base commit with no extra
/// setup. Exercises the `Profile` arm of
/// `check_required_amendments` that the shape and constraint
/// fixtures do not touch.
#[test]
fn ci_002_canonical_profile_name_change_without_amendment_fires() {
    let (dir, base) = init_git_store("software");
    set_top_level_name(&yaml_path(&dir, "profiles", 1), "renamed");
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-002") && stderr.contains("profile 1"),
        "expected CI-002 for canonical profile: {stderr}"
    );
}

/// CI-002 satisfied: a canonical profile modification plus a new
/// amendment created via `--target-profile` passes. Exercises the
/// `--target-profile` amendment-create plumbing end-to-end — the
/// scaffold round-trip test in cli_scaffold.rs only verifies that
/// the flag serializes to disk, not that the ci-check layer honors
/// the resulting profile_ids target.
#[test]
fn ci_002_canonical_profile_change_with_satisfying_amendment_passes() {
    let (dir, base) = init_git_store("software");
    set_top_level_name(&yaml_path(&dir, "profiles", 1), "renamed");
    shapes_in(&dir)
        .args([
            "create",
            "amendment",
            "--name",
            "fix",
            "--target-profile",
            "1",
            "--summary",
            "fixes the canonical profile",
        ])
        .assert()
        .success();
    shapes_in(&dir)
        .args(["ci-check", "--base", &base])
        .assert()
        .success();
}

/// CI-003: deleting an existing amendment file fires the
/// amendment-immutability check, just like a modification. An
/// amendment that shipped once cannot be removed without a new
/// amendment documenting the reversal. Pairs by amendment id so
/// renames-with-identical-content are still treated as unchanged.
#[test]
fn ci_003_deleted_amendment_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, _) = promoted_shape_on_base(&dir, "Promoted");
    // Create and commit an amendment targeting the promoted shape
    // so it exists on the base commit.
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "create",
            "amendment",
            "--name",
            "PriorFix",
            "--target-shape",
            &id.to_string(),
            "--summary",
            "fixed yesterday",
        ])
        .assert()
        .success();
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["commit", "--quiet", "-m", "ship prior amendment"],
    );
    let base = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    // Delete the amendment from the working tree without touching
    // the promoted shape. CI-003 should fire for the amendment
    // itself. CI-002 is silent because the shape is unchanged —
    // the satisfying-amendment logic only matters when the shape
    // was also modified, and here it was not.
    fs::remove_file(yaml_path(&dir, "amendments", 1)).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-003") && stderr.contains("deleted") && stderr.contains("amendment 1"),
        "expected CI-003 for deleted amendment: {stderr}"
    );
}

/// CI-003 carve-out: toggling ONLY the `archived` field on an existing
/// amendment is the sole permitted mutation. CI-003 must not fire.
#[test]
fn ci_003_archive_only_change_passes() {
    let (dir, _base0) = init_git_store("software");
    let (id, _) = promoted_shape_on_base(&dir, "Promoted");
    // Create and commit an amendment targeting the promoted shape.
    shapes_in(&dir)
        .args([
            "create",
            "amendment",
            "--name",
            "PriorFix",
            "--target-shape",
            &id.to_string(),
            "--summary",
            "fixed yesterday",
        ])
        .assert()
        .success();
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["commit", "--quiet", "-m", "ship prior amendment"],
    );
    let base = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    // Toggle only the archived flag — this is the sole permitted mutation.
    set_archived(&yaml_path(&dir, "amendments", 1));
    shapes_in(&dir)
        .args(["ci-check", "--base", &base])
        .assert()
        .success();
}

/// CI-003 carve-out boundary: toggling `archived` AND changing another
/// field (e.g. name) must still fire CI-003 — the carve-out only
/// applies when the archived field is the sole delta.
#[test]
fn ci_003_archive_plus_other_field_change_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, _) = promoted_shape_on_base(&dir, "Promoted");
    shapes_in(&dir)
        .args([
            "create",
            "amendment",
            "--name",
            "PriorFix",
            "--target-shape",
            &id.to_string(),
            "--summary",
            "fixed yesterday",
        ])
        .assert()
        .success();
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["commit", "--quiet", "-m", "ship prior amendment"],
    );
    let base = git_capture(dir.path(), &["rev-parse", "HEAD"]);
    // Toggle archived AND tamper with the name — should trip CI-003.
    let amend_path = yaml_path(&dir, "amendments", 1);
    set_archived(&amend_path);
    let text = fs::read_to_string(&amend_path).unwrap();
    let tampered = text.replacen("PriorFix", "TamperedName", 1);
    fs::write(&amend_path, tampered).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-003"),
        "expected CI-003 when archived + other field changed: {stderr}"
    );
}

/// CI-002: deleting the entire `.shapes/` directory fires CI-002
/// for every promoted/canonical node that was on base. The check
/// does not silently skip when the shapes directory is missing on
/// HEAD — that would be an escape hatch for full-graph removal,
/// exactly the kind of edit that should require an amendment.
#[test]
fn ci_002_full_shapes_dir_deletion_fires() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = promoted_shape_on_base(&dir, "Promoted");
    fs::remove_dir_all(dir.path().join(".shapes")).unwrap();
    let stderr = ci_check_fails(&dir, &base);
    assert!(
        stderr.contains("CI-002")
            && stderr.contains(&format!("shape {id}"))
            && stderr.contains("deleted"),
        "expected CI-002 deletion for full .shapes/ removal: {stderr}"
    );
}

/// Insta snapshot coverage for ci-check stderr output per constraint 30.
///
/// These lock the exact message text of the CI-001 / CI-002 invariants
/// plus the clean-case summary line emitted at
/// src/commands/ci_check.rs:584-598. Any wording drift surfaces as a
/// reviewable `.snap` diff. Update with `cargo insta review` after an
/// intentional copy change.
mod ci_check_snapshots {
    use super::*;

    /// Clean case: base commit has a promoted shape; HEAD modifies
    /// that shape's intent AND adds an amendment targeting it, so
    /// CI-002 is satisfied and ci-check passes. Stderr should be
    /// exactly "No issues found.\n" (ci_check.rs:584).
    #[test]
    fn ci_check_clean_stderr() {
        let (dir, _base0) = init_git_store("software");
        let (id, base) = promoted_shape_on_base(&dir, "Promoted");
        // Mutate the promoted shape's intent summary.
        set_intent_summary(&yaml_path(&dir, "shapes", id), "snapshot fixture rewrite");
        // Add an amendment targeting the shape to satisfy CI-002.
        Command::cargo_bin("shapes")
            .unwrap()
            .current_dir(dir.path())
            .args([
                "create",
                "amendment",
                "--name",
                "satisfy",
                "--target-shape",
                "1",
                "--summary",
                "rewrite promoted intent",
                "--source",
                "ai",
            ])
            .assert()
            .success();
        let out = shapes_in(&dir)
            .args(["ci-check", "--base", &base])
            .assert()
            .success()
            .get_output()
            .stderr
            .clone();
        let stderr = String::from_utf8(out).expect("stderr is utf-8");
        insta::assert_snapshot!(stderr);
    }

    /// CI-001: `--require-shapes-changes` is set but the PR has no
    /// diff under .shapes/. Only an unrelated README change. Locks
    /// the "[error] [CI-001] pull-request:- — PR has no changes …"
    /// line plus the summary "{} ci-check issue(s) found" at
    /// ci_check.rs:598.
    #[test]
    fn ci_check_ci001_stderr() {
        let (dir, base) = init_git_store("software");
        fs::write(dir.path().join("README.md"), "noop").unwrap();
        let stderr = shapes_in(&dir)
            .args(["ci-check", "--base", &base, "--require-shapes-changes"])
            .assert()
            .failure()
            .code(2)
            .get_output()
            .stderr
            .clone();
        let stderr = String::from_utf8(stderr).expect("stderr is utf-8");
        insta::assert_snapshot!(stderr);
    }

    /// CI-002: modify a promoted shape's intent without creating any
    /// amendment targeting it. Locks the full CI-002 message text
    /// including the node id, the cited field, and the summary count.
    #[test]
    fn ci_check_ci002_stderr() {
        let (dir, _base0) = init_git_store("software");
        let (id, base) = promoted_shape_on_base(&dir, "Promoted");
        set_intent_summary(&yaml_path(&dir, "shapes", id), "snapshot fixture rewrite");
        let stderr = ci_check_fails(&dir, &base);
        insta::assert_snapshot!(stderr);
    }
}
