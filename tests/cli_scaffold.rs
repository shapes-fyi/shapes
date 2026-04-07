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

// ---------------------------------------------------------------------------
// Software template
// ---------------------------------------------------------------------------

#[test]
fn software_shape_scaffold_has_required_fields_and_stubs() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "shape", "--name", "AuthService", "--kind", "service"])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "shapes");

    // Header comment with template name
    assert!(yaml.contains("template: software"), "missing template header: {yaml}");

    // Description and required intent fields are TODO blocks
    assert!(yaml.contains("description: |\n    TODO:"), "missing description TODO");
    assert!(yaml.contains("goals: |\n    TODO:"), "missing goals TODO");
    assert!(yaml.contains("rationale: |\n    TODO:"), "missing rationale TODO");

    // Optional fields are commented out (not enforced)
    assert!(yaml.contains("# non_goals:"), "non_goals should be commented");
    assert!(yaml.contains("# requirements:"), "requirements should be commented");

    // Stub sections present as comments
    assert!(yaml.contains("# parents:"), "parents stub missing");
    assert!(yaml.contains("# children:"), "children stub missing");
    assert!(yaml.contains("# constraints:"), "constraints stub missing");
    assert!(yaml.contains("# realization:"), "realization stub missing");

    // Validate parses & passes
    shapes_in(&dir).args(["get", "shape", "1"]).assert().success();
    shapes_in(&dir).arg("validate").assert().success();
}

#[test]
fn software_constraint_scaffold_has_rule_and_evidence_stub() {
    let dir = fresh_store("software");
    shapes_in(&dir)
        .args(["create", "constraint", "--name", "NoUnsafeBlocks", "--kind", "invariant"])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "constraints");

    assert!(yaml.contains("template: software"));
    assert!(yaml.contains("rule: |\n    TODO:"), "rule should be a TODO block");
    assert!(yaml.contains("rationale: |\n    TODO:"), "rationale required for software constraints");
    assert!(yaml.contains("# impact_if_violated:"), "impact stub missing");
    assert!(yaml.contains("# evidence:"), "evidence stub missing");
    assert!(yaml.contains("enforcement: manual"), "default enforcement should be manual");

    shapes_in(&dir).args(["get", "constraint", "1"]).assert().success();
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
    assert!(yaml.contains("Profiles are\n# OPTIONAL"), "profile preamble missing");

    // Software template's required fields are seeded
    assert!(yaml.contains("name: \"goals\""));
    assert!(yaml.contains("name: \"rationale\""));
    // Software shape kinds are seeded
    assert!(yaml.contains("name: \"system\""));
    assert!(yaml.contains("name: \"feature\""));
    // Lifecycle gates seeded
    assert!(yaml.contains("from: proposed"));
    assert!(yaml.contains("to: promoted"));

    shapes_in(&dir).args(["get", "profile", "1"]).assert().success();
    shapes_in(&dir).arg("validate").assert().success();
}

// ---------------------------------------------------------------------------
// Other templates
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// --from path still works (regression check)
// ---------------------------------------------------------------------------

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
    shapes_in(&dir).args(["get", "shape", "1"]).assert().success();
}

// ---------------------------------------------------------------------------
// Template override
// ---------------------------------------------------------------------------

#[test]
fn per_call_template_override_does_not_modify_meta() {
    let dir = fresh_store("software");
    // Override to research for one call
    shapes_in(&dir)
        .args(["create", "shape", "--name", "OneOff", "--template", "research"])
        .assert()
        .success();

    let yaml = read_only_yaml_in(&dir, "shapes");
    assert!(yaml.contains("template: research"), "override should produce research scaffold");

    // meta.yaml should still say software
    let meta = fs::read_to_string(dir.path().join(".shapes/meta.yaml")).unwrap();
    assert!(meta.contains("template: software"), "meta should be unchanged");
}
