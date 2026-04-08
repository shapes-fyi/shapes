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

/// CI-002 exemption: changing only realization metadata on a promoted
/// shape does NOT require an amendment (mechanical hygiene rule).
#[test]
fn ci_002_realization_only_change_does_not_require_amendment() {
    let (dir, _base0) = init_git_store("software");
    let (id, base) = promoted_shape_on_base(&dir, "Promoted");
    let path = yaml_path(&dir, "shapes", id);
    let text = fs::read_to_string(&path).unwrap();
    // Append a realization binding rather than touching anything
    // semantic. The realization block scaffolds out empty by default
    // when no flags are passed, so build it from scratch.
    let appended = format!(
        "{text}realization:\n  - bindings:\n      - scheme: path\n        value: Cargo.toml\n        metadata:\n          summary: arbitrary realization\n    role: primary\n"
    );
    fs::write(&path, appended).unwrap();
    shapes_in(&dir)
        .args(["ci-check", "--base", &base])
        .assert()
        .success();
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
