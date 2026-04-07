//! Hand-rolled YAML scaffold writers for `shapes create`.
//!
//! These emit YAML strings that:
//!   1. Parse cleanly via `serde_yml` (so subsequent `shapes get`,
//!      `shapes validate`, etc. work without round-tripping issues), and
//!   2. Are maximally informative when read directly from disk — every
//!      expected field is present, either populated with `TODO: <hint>`
//!      or commented out as a `# TODO:` stub block.
//!
//! Comments do not survive serde round-trips. That's intentional: the
//! comments are first-fill hints. Once the agent fills in the TODOs and
//! deletes irrelevant stub blocks, any subsequent `save()` through serde
//! produces a clean structured file with no leftover scaffolding noise.

use crate::model::{Enforcement, ShapeId};
use crate::templates::{FieldHint, KindHint, Template};

const TODO_DESCRIPTION: &str =
    "TODO: full paragraph — what this is, who uses it, what it does, why it exists";

// ---------------------------------------------------------------------------
// Shape scaffold
// ---------------------------------------------------------------------------

pub struct ShapeScaffold<'a> {
    pub id: ShapeId,
    pub name: &'a str,
    pub kind: &'a str,
    pub summary: Option<&'a str>,
    pub source: &'a str,
    pub description: Option<&'a str>,
    pub profile: Option<u64>,
    pub template: &'static Template,
}

pub fn scaffold_shape(s: &ShapeScaffold<'_>) -> String {
    let mut out = String::new();
    out.push_str(&header_comment("shape", s.template));

    out.push_str(&format!("id: {}\n", s.id.get()));
    out.push_str(&format!("name: {}\n", yaml_string(s.name)));
    out.push_str(&format!(
        "description: {}\n",
        yaml_block(s.description.unwrap_or(TODO_DESCRIPTION)),
    ));
    if let Some(pid) = s.profile {
        out.push_str(&format!("profile: {pid}\n"));
    }
    out.push_str("status: proposed\n");

    out.push_str("intent:\n");
    out.push_str(&format!("  kind: {}\n", yaml_string(s.kind)));
    out.push_str(&format!(
        "  summary: {}\n",
        yaml_string(s.summary.unwrap_or("TODO: one-line summary of this shape")),
    ));
    out.push_str(&format!("  source: {}\n", yaml_string(s.source)));
    write_intent_fields(&mut out, s.template.shape_intent_fields, "  ");

    write_optional_kinds_comment(&mut out, "shape", s.template.shape_kinds);
    write_parents_stub(&mut out, "shape");
    write_children_stub(&mut out, "shape");
    write_constraints_stub(&mut out);
    write_realization_stub(&mut out);

    out
}

// ---------------------------------------------------------------------------
// Constraint scaffold
// ---------------------------------------------------------------------------

pub struct ConstraintScaffold<'a> {
    pub id: u64,
    pub name: &'a str,
    pub kind: &'a str,
    pub rule: Option<&'a str>,
    pub enforcement: Enforcement,
    pub summary: Option<&'a str>,
    pub source: &'a str,
    pub description: Option<&'a str>,
    pub intent_kind: Option<&'a str>,
    pub profile: Option<u64>,
    pub template: &'static Template,
}

pub fn scaffold_constraint(c: &ConstraintScaffold<'_>) -> String {
    let mut out = String::new();
    out.push_str(&header_comment("constraint", c.template));

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
    if let Some(pid) = c.profile {
        out.push_str(&format!("profile: {pid}\n"));
    }
    out.push_str("status: proposed\n");

    out.push_str("intent:\n");
    out.push_str(&format!(
        "  kind: {}\n",
        yaml_string(c.intent_kind.unwrap_or(c.kind)),
    ));
    out.push_str(&format!(
        "  summary: {}\n",
        yaml_string(c.summary.unwrap_or("TODO: one-line summary of this constraint")),
    ));
    out.push_str(&format!("  source: {}\n", yaml_string(c.source)));
    write_intent_fields(&mut out, c.template.constraint_intent_fields, "  ");

    write_optional_kinds_comment(&mut out, "constraint", c.template.constraint_kinds);
    write_parents_stub(&mut out, "constraint");
    write_children_stub(&mut out, "constraint");
    write_realization_stub(&mut out);
    write_evidence_stub(&mut out);

    out
}

// ---------------------------------------------------------------------------
// Profile scaffold
// ---------------------------------------------------------------------------

pub struct ProfileScaffold<'a> {
    pub id: u64,
    pub name: &'a str,
    pub summary: Option<&'a str>,
    pub source: &'a str,
    pub description: Option<&'a str>,
    pub amendment_model: &'a str,
    pub template: &'static Template,
}

pub fn scaffold_profile(p: &ProfileScaffold<'_>) -> String {
    let mut out = String::new();
    out.push_str(
        "# Generated by `shapes create profile`. A Profile declares which intent fields\n\
         # and kinds are required for shapes and constraints in this project. Profiles are\n\
         # OPTIONAL — shapes and constraints work without one. Attach a profile by passing\n\
         # `--profile <id>` to `shapes create`, which will pre-populate required fields and\n\
         # validate kinds. The fields below are seeded from the active template; edit them\n\
         # to match this project's needs and run `shapes validate` when ready.\n\n",
    );

    out.push_str(&format!("id: {}\n", p.id));
    out.push_str(&format!("name: {}\n", yaml_string(p.name)));
    out.push_str(&format!(
        "description: {}\n",
        yaml_block(p.description.unwrap_or(
            "TODO: paragraph describing what this Profile governs and why",
        )),
    ));
    out.push_str("status: proposed\n");

    out.push_str("intent:\n");
    out.push_str("  kind: governance\n");
    out.push_str(&format!(
        "  summary: {}\n",
        yaml_string(p.summary.unwrap_or("TODO: one-line summary of what this profile governs")),
    ));
    out.push_str(&format!("  source: {}\n", yaml_string(p.source)));

    out.push_str("\nfields:\n");
    out.push_str("  shape:\n");
    out.push_str("    intent:\n");
    out.push_str("      fields:\n");
    for f in p.template.shape_intent_fields {
        write_field_def(&mut out, f, "        ");
    }
    out.push_str("      kinds:\n");
    if p.template.shape_kinds.is_empty() {
        out.push_str("        # No kind constraints — any kind is allowed.\n");
    } else {
        for k in p.template.shape_kinds {
            out.push_str(&format!(
                "        - name: {}\n          description: {}\n",
                yaml_string(k.name),
                yaml_string(k.description),
            ));
        }
    }
    out.push_str("  constraint:\n");
    out.push_str("    intent:\n");
    out.push_str("      fields:\n");
    for f in p.template.constraint_intent_fields {
        write_field_def(&mut out, f, "        ");
    }
    out.push_str("      kinds:\n");
    if p.template.constraint_kinds.is_empty() {
        out.push_str("        # No kind constraints — any kind is allowed.\n");
    } else {
        for k in p.template.constraint_kinds {
            out.push_str(&format!(
                "        - name: {}\n          description: {}\n",
                yaml_string(k.name),
                yaml_string(k.description),
            ));
        }
    }

    out.push_str("\nlifecycle:\n");
    out.push_str("  gates:\n");
    out.push_str("    - from: proposed\n");
    out.push_str("      to: promoted\n");
    out.push_str("      preconditions:\n");
    out.push_str("        - All required intent fields populated\n");
    out.push_str("    - from: promoted\n");
    out.push_str("      to: canonical\n");
    out.push_str("      preconditions:\n");
    out.push_str("        - Realization bindings present\n");

    out.push_str("\namendment_rules:\n");
    out.push_str(&format!("  application: {}\n", p.amendment_model));

    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn header_comment(node_kind: &str, template: &Template) -> String {
    format!(
        "# Generated by `shapes create {node_kind}` (template: {tname}).\n\
         # Replace each `TODO:` with real content. Uncomment and fill in the stub\n\
         # sections (parents/children/constraints/realization) you need; delete the\n\
         # ones you don't. Run `shapes validate` when ready.\n\n",
        tname = template.name,
    )
}

fn write_intent_fields(out: &mut String, fields: &[FieldHint], indent: &str) {
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

fn write_field_def(out: &mut String, f: &FieldHint, indent: &str) {
    out.push_str(&format!(
        "{indent}- name: {name}\n\
         {indent}  description: {desc}\n\
         {indent}  required: {req}\n",
        name = yaml_string(f.name),
        desc = yaml_string(f.description),
        req = f.required,
    ));
}

fn write_optional_kinds_comment(out: &mut String, node_kind: &str, kinds: &[KindHint]) {
    if kinds.is_empty() {
        return;
    }
    out.push_str(&format!("\n# Suggested {node_kind} kinds for this template:\n"));
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
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            '\n' => "\\n".to_string(),
            other => other.to_string(),
        })
        .collect();
    format!("\"{escaped}\"")
}

/// Format a longer value as a YAML literal block (`|`). Used for
/// description and intent body fields where the content may eventually be
/// multi-paragraph.
fn yaml_block(s: &str) -> String {
    let mut out = String::from("|\n");
    for line in s.lines() {
        out.push_str("    ");
        out.push_str(line);
        out.push('\n');
    }
    if s.is_empty() {
        out.push_str("    \n");
    }
    // Trim the trailing newline so the parent writer's `\n` lands cleanly.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

