//! Integration tests for the `shapes create` scaffold path.
//!
//! Each test runs the real `shapes` binary against a fresh tempdir,
//! creates nodes, and asserts the on-disk YAML contains the expected
//! `TODO:` markers and stub sections — and that the file still parses
//! cleanly via `shapes get` and `shapes validate`.

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

/// Build a `shapes` command rooted in a fresh tempdir with `shapes init`
/// already run for the given template.
fn fresh_store(template: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--template", template])
        .assert()
        .success();
    dir
}

fn shapes_in(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("shapes").unwrap();
    cmd.current_dir(dir.path());
    cmd
}

fn read_only_yaml_in(dir: &TempDir, subdir: &str) -> String {
    let dir = dir.path().join(".shapes").join(subdir);
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
        .collect();
    entries.sort();
    assert_eq!(
        entries.len(),
        1,
        "expected exactly one yaml file in {}, got {entries:?}",
        dir.display(),
    );
    fs::read_to_string(&entries[0]).expect("read yaml")
}

#[test]
fn software_shape_scaffold_has_required_fields_and_stubs() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args([
            "create",
            "shape",
            "--name",
            "AuthService",
            "--kind",
            "service",
        ])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "shapes");

    // Header comment with template name
    assert!(
        yaml.contains("template: software"),
        "missing template header: {yaml}"
    );

    // Description and required intent fields are TODO blocks
    assert!(
        yaml.contains("description: |\n    TODO:"),
        "missing description TODO"
    );
    assert!(yaml.contains("goals: |\n    TODO:"), "missing goals TODO");
    assert!(
        yaml.contains("rationale: |\n    TODO:"),
        "missing rationale TODO"
    );

    // Optional fields are commented out (not enforced)
    assert!(
        yaml.contains("# non_goals:"),
        "non_goals should be commented"
    );
    assert!(
        yaml.contains("# requirements:"),
        "requirements should be commented"
    );

    // Stub sections present as comments
    assert!(yaml.contains("# parents:"), "parents stub missing");
    assert!(yaml.contains("# children:"), "children stub missing");
    assert!(yaml.contains("# constraints:"), "constraints stub missing");
    assert!(yaml.contains("# realization:"), "realization stub missing");

    // Validate parses & passes
    shapes_in(&dir)
        .args(["get", "shape", "1"])
        .assert()
        .success();
    shapes_in(&dir).arg("validate").assert().success();
}

#[test]
fn software_constraint_scaffold_has_rule_and_evidence_stub() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args([
            "create",
            "constraint",
            "--name",
            "NoUnsafeBlocks",
            "--kind",
            "invariant",
        ])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "constraints");

    assert!(yaml.contains("template: software"));
    assert!(
        yaml.contains("rule: |\n    TODO:"),
        "rule should be a TODO block"
    );
    assert!(
        yaml.contains("rationale: |\n    TODO:"),
        "rationale required for software constraints"
    );
    assert!(
        yaml.contains("# impact_if_violated:"),
        "impact stub missing"
    );
    assert!(yaml.contains("# evidence:"), "evidence stub missing");
    assert!(
        yaml.contains("enforcement: manual"),
        "default enforcement should be manual"
    );

    shapes_in(&dir)
        .args(["get", "constraint", "1"])
        .assert()
        .success();
    shapes_in(&dir).arg("validate").assert().success();
}

#[test]
fn software_profile_scaffold_seeds_template_fields() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "profile", "--name", "Strict"])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "profiles");

    // Profile preamble explains optionality
    assert!(
        yaml.contains("Profiles are\n# OPTIONAL"),
        "profile preamble missing"
    );

    // Software template's required fields are seeded
    assert!(yaml.contains("name: \"goals\""));
    assert!(yaml.contains("name: \"rationale\""));
    // Software shape kinds are seeded
    assert!(yaml.contains("name: \"system\""));
    assert!(yaml.contains("name: \"feature\""));
    // Lifecycle gates seeded
    assert!(yaml.contains("from: proposed"));
    assert!(yaml.contains("to: promoted"));

    shapes_in(&dir)
        .args(["get", "profile", "1"])
        .assert()
        .success();
    shapes_in(&dir).arg("validate").assert().success();
}

#[test]
fn research_template_uses_research_field_hints() {
    let dir = fresh_store("research");
    shapes_in(&dir)
        .args(["create", "shape", "--name", "DecayExperiment"])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "shapes");
    assert!(yaml.contains("template: research"));
    assert!(yaml.contains("hypotheses: |\n    TODO:"));
    assert!(yaml.contains("success_criteria: |\n    TODO:"));
    assert!(yaml.contains("methodology: |\n    TODO:"));
    // Default kind for research is "experiment"
    assert!(yaml.contains("kind: \"experiment\""));
    shapes_in(&dir).arg("validate").assert().success();
}

#[test]
fn editorial_template_uses_editorial_field_hints() {
    let dir = fresh_store("editorial");
    shapes_in(&dir)
        .args(["create", "shape", "--name", "Chapter1"])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "shapes");
    assert!(yaml.contains("template: editorial"));
    assert!(yaml.contains("themes: |\n    TODO:"));
    assert!(yaml.contains("target_audience: |\n    TODO:"));
    assert!(yaml.contains("tone: |\n    TODO:"));
    assert!(yaml.contains("kind: \"chapter\""));
    shapes_in(&dir).arg("validate").assert().success();
}

#[test]
fn minimal_template_only_requires_rationale() {
    let dir = fresh_store("minimal");
    shapes_in(&dir)
        .args(["create", "shape", "--name", "Bare", "--kind", "anything"])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "shapes");
    assert!(yaml.contains("template: minimal"));
    assert!(yaml.contains("rationale: |\n    TODO:"));
    // Minimal has no kind hints, so the suggested-kinds comment should be absent
    assert!(!yaml.contains("# Suggested shape kinds"));
    shapes_in(&dir).arg("validate").assert().success();
}

#[test]
fn from_stdin_path_still_works_without_name_flag() {
    let dir = fresh_store("software");
    let yaml_input = "id: 0\n\
        name: from-stdin\n\
        description: a real description\n\
        status: proposed\n\
        intent:\n  \
            kind: module\n  \
            summary: from stdin\n  \
            source: human\n";
    shapes_in(&dir)
        .args(["create", "shape", "--from", "-"])
        .write_stdin(yaml_input)
        .assert()
        .success();
    shapes_in(&dir)
        .args(["get", "shape", "1"])
        .assert()
        .success();
}

#[test]
fn per_call_template_override_does_not_modify_meta() {
    let dir = fresh_store("software");
    // Override to research for one call
    shapes_in(&dir)
        .args([
            "create",
            "shape",
            "--name",
            "OneOff",
            "--template",
            "research",
        ])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "shapes");
    assert!(
        yaml.contains("template: research"),
        "override should produce research scaffold"
    );

    // meta.yaml should still say software
    let meta = fs::read_to_string(dir.path().join(".shapes/meta.yaml")).unwrap();
    assert!(
        meta.contains("template: software"),
        "meta should be unchanged"
    );
}

#[test]
fn init_refuses_to_overwrite_existing_store() {
    let dir = fresh_store("software");
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .failure();
}

#[test]
fn init_rejects_unknown_template() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("shapes")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--template", "bogus"])
        .assert()
        .failure();
    assert!(
        !dir.path().join(".shapes").exists(),
        "store should not be created when template is invalid",
    );
}

#[test]
fn constraint_rejects_human_enforcement_with_helpful_error() {
    let dir = fresh_store("software");
    let assert = shapes_in(&dir)
        .args([
            "create",
            "constraint",
            "--name",
            "X",
            "--kind",
            "invariant",
            "--enforcement",
            "human",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("invalid value 'human'") && stderr.contains("manual"),
        "expected helpful enforcement error, got: {stderr}",
    );
}

#[test]
fn create_shape_without_name_or_from_fails_with_clap_error() {
    let dir = fresh_store("software");
    let assert = shapes_in(&dir).args(["create", "shape"]).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("required") && stderr.contains("name"),
        "expected required-name error, got: {stderr}",
    );
}

// previously panicked clap's debug_assert when --from was combined with
// --target-shape / --target-constraint / --version-impact.

#[test]
fn amendment_create_via_flags_succeeds() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "shape", "--name", "Target", "--kind", "feature"])
        .assert()
        .success();
    shapes_in(&dir)
        .args([
            "create",
            "amendment",
            "--name",
            "Test Amendment",
            "--target-shape",
            "1",
            "--summary",
            "what changed",
        ])
        .assert()
        .success();
}

#[test]
fn amendment_create_via_from_stdin_succeeds() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "shape", "--name", "Target", "--kind", "feature"])
        .assert()
        .success();
    let yaml = "id: 0\n\
        name: stdin amendment\n\
        description: d\n\
        targets:\n  \
          shape_ids: [1]\n\
        status: proposed\n\
        intent:\n  \
          kind: amendment\n  \
          summary: s\n  \
          source: ai\n\
        initiated_by:\n  \
          type: ai\n";
    shapes_in(&dir)
        .args(["create", "amendment", "--from", "-"])
        .write_stdin(yaml)
        .assert()
        .success();
}

#[test]
fn profile_kind_validation_rejects_disallowed_kind() {
    let dir = fresh_store("software");
    // Software profile allows system/service/feature/module/interface/data-flow/pattern
    shapes_in(&dir)
        .args(["create", "profile", "--name", "Strict"])
        .assert()
        .success();
    let assert = shapes_in(&dir)
        .args([
            "create",
            "shape",
            "--name",
            "BadKind",
            "--kind",
            "nonsense",
            "--profile",
            "1",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("not allowed") && stderr.contains("nonsense"),
        "expected kind rejection, got: {stderr}",
    );
}

#[test]
fn profile_kind_validation_accepts_allowed_kind() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "profile", "--name", "Strict"])
        .assert()
        .success();
    shapes_in(&dir)
        .args([
            "create",
            "shape",
            "--name",
            "GoodKind",
            "--kind",
            "feature",
            "--profile",
            "1",
        ])
        .assert()
        .success();
}

#[test]
fn list_and_tree_and_inheritance_work_after_linking_parent_and_child() {
    let dir = fresh_store("software");
    // Create a constraint and two shapes linked parent → child
    let parent_yaml = "id: 0\n\
        name: parent-shape\n\
        description: d\n\
        status: proposed\n\
        intent:\n  \
          kind: service\n  \
          summary: s\n  \
          source: human\n  \
          goals: g\n  \
          rationale: r\n\
        children:\n  \
          - shape: 2\n    \
              role: component\n\
        constraints:\n  \
          - 1\n";
    let child_yaml = "id: 0\n\
        name: child-shape\n\
        description: d\n\
        status: proposed\n\
        intent:\n  \
          kind: feature\n  \
          summary: s\n  \
          source: human\n  \
          goals: g\n  \
          rationale: r\n\
        parents:\n  \
          - id: 1\n    \
              role: component\n";
    let constraint_yaml = "id: 0\n\
        name: TheRule\n\
        description: d\n\
        kind: invariant\n\
        rule: r\n\
        enforcement: machine\n\
        status: proposed\n\
        intent:\n  \
          kind: invariant\n  \
          summary: s\n  \
          source: human\n  \
          rationale: r\n  \
          impact_if_violated: i\n";
    shapes_in(&dir)
        .args(["create", "constraint", "--from", "-"])
        .write_stdin(constraint_yaml)
        .assert()
        .success();
    shapes_in(&dir)
        .args(["create", "shape", "--from", "-"])
        .write_stdin(parent_yaml)
        .assert()
        .success();
    shapes_in(&dir)
        .args(["create", "shape", "--from", "-"])
        .write_stdin(child_yaml)
        .assert()
        .success();
    shapes_in(&dir).arg("validate").assert().success();

    // list shape includes both
    let list_out = shapes_in(&dir).args(["list", "shape"]).assert().success();
    let list_stdout = String::from_utf8_lossy(&list_out.get_output().stdout).to_string();
    assert!(list_stdout.contains("parent-shape"));
    assert!(list_stdout.contains("child-shape"));

    // list --kind filter
    let kind_filter = shapes_in(&dir)
        .args(["list", "shape", "--kind", "service"])
        .assert()
        .success();
    let kind_stdout = String::from_utf8_lossy(&kind_filter.get_output().stdout).to_string();
    assert!(kind_stdout.contains("parent-shape"));
    assert!(!kind_stdout.contains("child-shape"));

    // tree shape
    let tree_out = shapes_in(&dir).args(["tree", "shape"]).assert().success();
    let tree_stdout = String::from_utf8_lossy(&tree_out.get_output().stdout).to_string();
    assert!(tree_stdout.contains("parent-shape"));
    assert!(tree_stdout.contains("child-shape"));

    // ancestors / descendants
    let ancestors = shapes_in(&dir)
        .args(["query", "ancestors", "shape", "2"])
        .assert()
        .success();
    let ancestors_stdout = String::from_utf8_lossy(&ancestors.get_output().stdout).to_string();
    assert!(ancestors_stdout.contains("1"), "ancestors should include 1");

    let descendants = shapes_in(&dir)
        .args(["query", "descendants", "shape", "1"])
        .assert()
        .success();
    let descendants_stdout = String::from_utf8_lossy(&descendants.get_output().stdout).to_string();
    assert!(
        descendants_stdout.contains("2"),
        "descendants should include 2"
    );

    // Constraint inheritance: child should see TheRule even though it's
    // declared on the parent.
    let constraints = shapes_in(&dir)
        .args(["query", "constraints", "2"])
        .assert()
        .success();
    let constraints_stdout = String::from_utf8_lossy(&constraints.get_output().stdout).to_string();
    assert!(
        constraints_stdout.contains("TheRule"),
        "child should inherit TheRule from parent: {constraints_stdout}",
    );
}

#[test]
fn validate_detects_dangling_child_reference() {
    let dir = fresh_store("software");
    let bad_yaml = "id: 0\n\
        name: lonely\n\
        description: d\n\
        status: proposed\n\
        intent:\n  \
          kind: feature\n  \
          summary: s\n  \
          source: human\n  \
          goals: g\n  \
          rationale: r\n\
        children:\n  \
          - shape: 999\n    \
              role: component\n";
    shapes_in(&dir)
        .args(["create", "shape", "--from", "-"])
        .write_stdin(bad_yaml)
        .assert()
        .success();
    // Validate should fail with exit code 2 (validation issues found).
    let assert = shapes_in(&dir).arg("validate").assert().failure();
    assert.code(2);
}

#[test]
fn get_missing_id_returns_not_found_error() {
    let dir = fresh_store("software");
    let assert = shapes_in(&dir)
        .args(["get", "shape", "999"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(stderr.to_lowercase().contains("not found"), "got: {stderr}");
}

#[test]
fn create_from_stdin_rejects_malformed_yaml() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "shape", "--from", "-"])
        .write_stdin("not yaml :::\n")
        .assert()
        .failure();
}

#[test]
fn list_supports_json_format() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "shape", "--name", "X", "--kind", "feature"])
        .assert()
        .success();
    let assert = shapes_in(&dir)
        .args(["list", "shape", "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(
        stdout.trim_start().starts_with('['),
        "expected JSON array, got: {stdout}"
    );
    assert!(stdout.contains("\"id\""));
    assert!(stdout.contains("\"name\""));
}

#[test]
fn validate_supports_json_format() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["validate", "--format", "json"])
        .assert()
        .success();
}

/// Snapshot coverage for scaffold output per constraint 30.
///
/// These capture the full scaffold YAML verbatim so any drift in
/// whitespace, ordering, or comment text shows up as a reviewable
/// `.snap` diff. Update with `cargo insta review` after intentional
/// changes.
mod scaffold_snapshots {
    use super::{fresh_store, read_only_yaml_in, shapes_in};

    #[test]
    fn software_shape_scaffold_snapshot() {
        let dir = fresh_store("software");
        shapes_in(&dir)
            .args([
                "create",
                "shape",
                "--name",
                "AuthService",
                "--kind",
                "service",
            ])
            .assert()
            .success();
        let yaml = read_only_yaml_in(&dir, "shapes");
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn software_constraint_scaffold_snapshot() {
        let dir = fresh_store("software");
        shapes_in(&dir)
            .args([
                "create",
                "constraint",
                "--name",
                "NoUnsafeBlocks",
                "--kind",
                "invariant",
            ])
            .assert()
            .success();
        let yaml = read_only_yaml_in(&dir, "constraints");
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn software_profile_scaffold_snapshot() {
        let dir = fresh_store("software");
        shapes_in(&dir)
            .args(["create", "profile", "--name", "Strict"])
            .assert()
            .success();
        let yaml = read_only_yaml_in(&dir, "profiles");
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn research_shape_scaffold_snapshot() {
        let dir = fresh_store("research");
        shapes_in(&dir)
            .args(["create", "shape", "--name", "DecayExperiment"])
            .assert()
            .success();
        let yaml = read_only_yaml_in(&dir, "shapes");
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn editorial_shape_scaffold_snapshot() {
        let dir = fresh_store("editorial");
        shapes_in(&dir)
            .args(["create", "shape", "--name", "Chapter1"])
            .assert()
            .success();
        let yaml = read_only_yaml_in(&dir, "shapes");
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn minimal_shape_scaffold_snapshot() {
        let dir = fresh_store("minimal");
        shapes_in(&dir)
            .args(["create", "shape", "--name", "Bare", "--kind", "anything"])
            .assert()
            .success();
        let yaml = read_only_yaml_in(&dir, "shapes");
        insta::assert_snapshot!(yaml);
    }
}
