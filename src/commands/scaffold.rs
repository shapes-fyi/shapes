//! Hand-rolled YAML scaffold writers for `shapes create shape` and
//! `shapes create constraint`.
//!
//! These emit YAML strings that:
//!   1. Parse cleanly via `serde_yml` (so subsequent `shapes get`,
//!      `shapes validate`, etc. work without round-tripping issues), and
//!   2. Are maximally informative when read directly from disk — every
//!      expected field is present, either populated with `TODO: <hint>`
//!      or commented out as a `# TODO:` stub block.
//!
//! Comments do not survive serde round-trips. That's intentional: the
//! comments are first-fill hints. Once the agent fills in the TODOs
//! and deletes irrelevant stub blocks, any subsequent `save()` through
//! serde produces a clean structured file with no leftover scaffolding
//! noise.
//!
//! **Source of hints.** Both scaffold writers read field hints and
//! kind suggestions from a [`Profile`] — the active profile or a
//! per-call override. There is no parallel hardcoded template layer;
//! see constraint 34 (Profile is Sole Domain Schema).
//!
//! Profile scaffolding (used by `shapes create profile`) does **not**
//! live here — it is handled by
//! [`StarterKit::to_profile_yaml`](crate::templates::StarterKit::to_profile_yaml),
//! which builds a [`Profile`] struct directly and serializes it via
//! serde.

use crate::model::profile::FieldDef;
use crate::model::{Enforcement, Profile, ShapeId};

const TODO_DESCRIPTION: &str =
    "TODO: full paragraph — what this is, who uses it, what it does, why it exists";

/// Field bag passed to [`scaffold_shape`].
pub struct ShapeScaffold<'a> {
    /// The ID allocated by the store for this new shape.
    pub id: ShapeId,
    /// `--name`.
    pub name: &'a str,
    /// Resolved `--kind` (already defaulted from the profile if the
    /// caller omitted it).
    pub kind: &'a str,
    /// Optional `--summary`.
    pub summary: Option<&'a str>,
    /// `--source`.
    pub source: &'a str,
    /// Optional `--description`.
    pub description: Option<&'a str>,
    /// The profile governing this shape. Provides field hints, kind
    /// suggestions, and the `profile:` reference stamped onto the
    /// created node.
    pub profile: &'a Profile,
}

/// Scaffolds a new shape YAML from the profile's shape-section hints.
pub fn scaffold_shape(s: &ShapeScaffold<'_>) -> String {
    let mut out = String::new();
    out.push_str(&header_comment("shape", s.profile));

    out.push_str(&format!("id: {}\n", s.id.get()));
    out.push_str(&format!("name: {}\n", yaml_string(s.name)));
    out.push_str(&format!(
        "description: {}\n",
        yaml_block(s.description.unwrap_or(TODO_DESCRIPTION)),
    ));
    out.push_str(&format!("profile: {}\n", s.profile.id.get()));
    out.push_str("status: proposed\n");

    out.push_str("intent:\n");
    out.push_str(&format!("  kind: {}\n", yaml_string(s.kind)));
    out.push_str(&format!(
        "  summary: {}\n",
        yaml_string(s.summary.unwrap_or("TODO: one-line summary of this shape")),
    ));
    out.push_str(&format!("  source: {}\n", yaml_string(s.source)));
    write_intent_fields(&mut out, shape_intent_field_hints(s.profile), "  ");

    write_optional_kinds_comment(&mut out, "shape", shape_kind_hints(s.profile));
    write_parents_stub(&mut out, "shape");
    write_children_stub(&mut out, "shape");
    write_constraints_stub(&mut out);
    write_realization_stub(&mut out);

    out
}

/// Field bag passed to [`scaffold_constraint`].
pub struct ConstraintScaffold<'a> {
    /// The raw ID allocated by the store.
    pub id: u64,
    /// `--name`.
    pub name: &'a str,
    /// Resolved `--kind` (already defaulted from the profile).
    pub kind: &'a str,
    /// Optional `--rule` body.
    pub rule: Option<&'a str>,
    /// `--enforcement`.
    pub enforcement: Enforcement,
    /// Optional `--summary`.
    pub summary: Option<&'a str>,
    /// `--source`.
    pub source: &'a str,
    /// Optional `--description`.
    pub description: Option<&'a str>,
    /// Optional `--intent-kind` override (distinct from `--kind`).
    pub intent_kind: Option<&'a str>,
    /// The profile governing this constraint.
    pub profile: &'a Profile,
}

/// Scaffolds a new constraint YAML from the profile's constraint-section
/// hints.
pub fn scaffold_constraint(c: &ConstraintScaffold<'_>) -> String {
    let mut out = String::new();
    out.push_str(&header_comment("constraint", c.profile));

    out.push_str(&format!("id: {}\n", c.id));
    out.push_str(&format!("name: {}\n", yaml_string(c.name)));
    out.push_str(&format!(
        "description: {}\n",
        yaml_block(c.description.unwrap_or(TODO_DESCRIPTION)),
    ));
    out.push_str(&format!("kind: {}\n", yaml_string(c.kind)));
    out.push_str(&format!(
        "rule: {}\n",
        yaml_block(c.rule.unwrap_or(
            "TODO: specific, falsifiable rule — phrased so it can be checked by reading code",
        )),
    ));
    out.push_str(&format!(
        "enforcement: {}\n",
        match c.enforcement {
            Enforcement::Manual => "manual",
            Enforcement::Machine => "machine",
        }
    ));
    out.push_str(&format!("profile: {}\n", c.profile.id.get()));
    out.push_str("status: proposed\n");

    out.push_str("intent:\n");
    out.push_str(&format!(
        "  kind: {}\n",
        yaml_string(c.intent_kind.unwrap_or(c.kind)),
    ));
    out.push_str(&format!(
        "  summary: {}\n",
        yaml_string(
            c.summary
                .unwrap_or("TODO: one-line summary of this constraint")
        ),
    ));
    out.push_str(&format!("  source: {}\n", yaml_string(c.source)));
    write_intent_fields(&mut out, constraint_intent_field_hints(c.profile), "  ");

    write_optional_kinds_comment(&mut out, "constraint", constraint_kind_hints(c.profile));
    write_parents_stub(&mut out, "constraint");
    write_children_stub(&mut out, "constraint");
    write_realization_stub(&mut out);
    write_evidence_stub(&mut out);

    out
}

fn header_comment(node_kind: &str, profile: &Profile) -> String {
    format!(
        "# Generated by `shapes create {node_kind}` (profile: {pid} — {pname}).\n\
         # Replace each `TODO:` with real content. Uncomment and fill in the stub\n\
         # sections (parents/children/constraints/realization) you need; delete the\n\
         # ones you don't. Run `shapes validate` when ready.\n\n",
        pid = profile.id.get(),
        pname = profile.name,
    )
}

/// Returns the profile's shape intent-field hints, or an empty slice
/// if the profile does not declare any.
fn shape_intent_field_hints(profile: &Profile) -> &[FieldDef] {
    profile
        .fields
        .as_ref()
        .and_then(|f| f.shape.as_ref())
        .and_then(|s| s.intent.as_ref())
        .map(|g| g.fields.as_slice())
        .unwrap_or(&[])
}

/// Returns the profile's shape kind allow-list, or an empty slice.
fn shape_kind_hints(profile: &Profile) -> &[FieldDef] {
    profile
        .fields
        .as_ref()
        .and_then(|f| f.shape.as_ref())
        .and_then(|s| s.intent.as_ref())
        .map(|g| g.kinds.as_slice())
        .unwrap_or(&[])
}

/// Returns the profile's constraint intent-field hints.
fn constraint_intent_field_hints(profile: &Profile) -> &[FieldDef] {
    profile
        .fields
        .as_ref()
        .and_then(|f| f.constraint.as_ref())
        .and_then(|s| s.intent.as_ref())
        .map(|g| g.fields.as_slice())
        .unwrap_or(&[])
}

/// Returns the profile's constraint kind allow-list.
fn constraint_kind_hints(profile: &Profile) -> &[FieldDef] {
    profile
        .fields
        .as_ref()
        .and_then(|f| f.constraint.as_ref())
        .and_then(|s| s.intent.as_ref())
        .map(|g| g.kinds.as_slice())
        .unwrap_or(&[])
}

fn write_intent_fields(out: &mut String, fields: &[FieldDef], indent: &str) {
    for f in fields {
        if f.required {
            out.push_str(&format!(
                "{indent}{name}: {value}\n",
                name = f.name,
                value = yaml_block(&format!("TODO: {}", f.description)),
            ));
        } else {
            out.push_str(&format!(
                "{indent}# {name}: {value}\n",
                name = f.name,
                value = yaml_string(&format!("TODO: {}", f.description)),
            ));
        }
    }
}

fn write_optional_kinds_comment(out: &mut String, node_kind: &str, kinds: &[FieldDef]) {
    if kinds.is_empty() {
        return;
    }
    out.push_str(&format!("\n# Suggested {node_kind} kinds from profile:\n"));
    for k in kinds {
        out.push_str(&format!("#   {} — {}\n", k.name, k.description));
    }
}

fn write_parents_stub(out: &mut String, node_kind: &str) {
    out.push_str(&format!(
        "\n# Parent {node_kind}s (this {node_kind} is a child of):\n\
         # parents:\n\
         #   - id: TODO  # parent {node_kind} ID\n\
         #     role: component\n",
    ));
}

fn write_children_stub(out: &mut String, node_kind: &str) {
    let key = match node_kind {
        "shape" => "shape",
        "constraint" => "constraint",
        _ => "node",
    };
    out.push_str(&format!(
        "\n# Child {node_kind}s (this {node_kind} contains):\n\
         # children:\n\
         #   - {key}: TODO  # child {node_kind} ID\n\
         #     role: component\n",
    ));
}

fn write_constraints_stub(out: &mut String) {
    out.push_str(
        "\n# Constraints that apply to this shape (by ID):\n\
         # constraints:\n\
         #   - TODO  # constraint ID\n",
    );
}

fn write_realization_stub(out: &mut String) {
    out.push_str(
        "\n# Where this is realized in the codebase:\n\
         # realization:\n\
         #   - bindings:\n\
         #       - scheme: path\n\
         #         value: TODO  # path/to/file.rs\n\
         #         metadata:\n\
         #           summary: TODO — what this file does for this node\n\
         #     role: primary\n",
    );
}

fn write_evidence_stub(out: &mut String) {
    out.push_str(
        "\n# Evidence that this constraint holds (tests, reviews, audits):\n\
         # evidence:\n\
         #   - id: TODO  # short identifier, e.g. unit-test-suite\n\
         #     type: test\n\
         #     bindings:\n\
         #       - scheme: path\n\
         #         value: TODO  # tests/path.rs\n\
         #         metadata: {}\n",
    );
}

/// Quote a value as a YAML string. Uses double quotes and escapes inner
/// quotes and backslashes. Safe for any single-line content.
fn yaml_string(s: &str) -> String {
    let escaped: String = s
        .chars()
        .map(|c| match c {
            '\\' => "\\\\".to_owned(),
            '"' => "\\\"".to_owned(),
            '\n' => "\\n".to_owned(),
            other => other.to_string(),
        })
        .collect();
    format!("\"{escaped}\"")
}

/// Format a longer value as a YAML literal block (`|`). Used for
/// description and intent body fields where the content may eventually
/// be multi-paragraph.
fn yaml_block(s: &str) -> String {
    let mut out = String::from("|\n");
    for line in s.lines() {
        out.push_str("    ");
        out.push_str(line);
        out.push('\n');
    }
    // Trim the trailing newline that literal blocks add.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}
