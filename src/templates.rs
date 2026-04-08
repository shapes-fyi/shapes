//! Starter kits for `shapes init` / `shapes create profile`.
//!
//! A [`StarterKit`] is an internal, compile-time data structure that
//! exists **only** to generate the first [`Profile`] for a new store
//! (via `shapes init`) or to scaffold a new profile on demand (via
//! `shapes create profile --kit <name>`).
//!
//! Kits are **not** a parallel governance layer. The runtime
//! governance source of truth is the active Profile recorded in
//! `meta.yaml`. Nothing in the `shapes create shape` or
//! `shapes create constraint` paths, and nothing in `shapes validate`,
//! consults a kit. This keeps intent-field hints, kind allow-lists,
//! default kinds, and required-field enforcement in exactly one place:
//! the Profile node.
//!
//! See constraint 34 (Profile is Sole Domain Schema) in `.shapes/`.

use std::collections::BTreeMap;

use clap::ValueEnum;

use crate::model::profile::{
    AmendmentModel, AmendmentRules, FieldDef, FieldGroup, FieldSection, ProfileFields,
};
use crate::model::{Intent, Profile, ProfileId, Status};

/// Internal starter-kit definition. Carries enough data to seed a
/// complete [`Profile`] for a new domain.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StarterKit {
    /// Kit identifier, matches the `--kit` flag value.
    pub name: &'static str,
    /// One-line human-readable description of the domain.
    pub description: &'static str,
    /// Shape intent field hints (seeded into
    /// `profile.fields.shape.intent.fields`).
    pub shape_intent_fields: &'static [FieldHint],
    /// Allowed shape kinds (seeded into
    /// `profile.fields.shape.intent.kinds`).
    pub shape_kinds: &'static [KindHint],
    /// Allowed shape `intent.source` values (seeded into
    /// `profile.fields.shape.intent.sources`). Empty = unrestricted.
    pub shape_sources: &'static [FieldHint],
    /// Constraint intent field hints.
    pub constraint_intent_fields: &'static [FieldHint],
    /// Allowed constraint kinds.
    pub constraint_kinds: &'static [KindHint],
    /// Allowed constraint `intent.source` values. Empty = unrestricted.
    pub constraint_sources: &'static [FieldHint],
    /// Default value for `shape.intent.kind` when `--kind` is omitted.
    pub default_shape_kind: &'static str,
    /// Default value for `constraint.intent.kind` when `--kind` is
    /// omitted.
    pub default_constraint_kind: &'static str,
}

/// A single intent-field hint embedded in a [`StarterKit`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldHint {
    /// Field name as it appears in the intent map.
    pub name: &'static str,
    /// Description shown to the author as a comment or doc.
    pub description: &'static str,
    /// `true` if the author must supply this field.
    pub required: bool,
}

/// A single kind hint embedded in a [`StarterKit`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct KindHint {
    /// Kind identifier.
    pub name: &'static str,
    /// Description shown to the author.
    pub description: &'static str,
}

/// Clap surface for `shapes init --kit <kind>` and
/// `shapes create profile --kit <kind>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum KitKind {
    /// Software engineering project — goals/rationale/non_goals,
    /// system/module/feature kinds.
    Software,
    /// Research project — hypotheses/methodology/success criteria,
    /// experiment/dataset kinds.
    Research,
    /// Editorial / writing project — themes/audience/tone,
    /// chapter/section/character kinds.
    Editorial,
    /// Bare minimum — just rationale, no kind enforcement.
    Minimal,
}

impl KitKind {
    /// Returns the static [`StarterKit`] backing this enum variant.
    #[must_use]
    pub(crate) fn kit(&self) -> &'static StarterKit {
        match self {
            KitKind::Software => &SOFTWARE,
            KitKind::Research => &RESEARCH,
            KitKind::Editorial => &EDITORIAL,
            KitKind::Minimal => &MINIMAL,
        }
    }
}

impl StarterKit {
    /// Builds a [`Profile`] struct seeded from this kit's hints.
    ///
    /// The resulting profile carries the kit's field hints, kind
    /// allow-lists, and default kinds in the canonical locations
    /// (`fields.{shape,constraint}.intent.{fields,kinds}` and
    /// `fields.{shape,constraint}.default_kind`). It is marked
    /// `canonical` and uses the `merge` amendment model.
    pub(crate) fn build_profile(self, id: ProfileId, name: &str) -> Profile {
        Profile {
            id,
            name: name.to_owned(),
            description: format!(
                "Starter profile seeded from the {} kit ({}). Edit freely.",
                self.name, self.description,
            ),
            version: None,
            status: Status::canonical(),
            intent: Intent {
                kind: "governance".to_owned(),
                summary: format!(
                    "Domain governance seeded from the {} starter kit",
                    self.name
                ),
                source: serde_yml::Value::String("system".to_owned()),
                uris: vec![],
                extra: BTreeMap::new(),
            },
            provenance: vec![],
            lifecycle: None,
            fields: Some(ProfileFields {
                shape: Some(node_section(
                    self.default_shape_kind,
                    self.shape_intent_fields,
                    self.shape_kinds,
                    self.shape_sources,
                )),
                constraint: Some(node_section(
                    self.default_constraint_kind,
                    self.constraint_intent_fields,
                    self.constraint_kinds,
                    self.constraint_sources,
                )),
            }),
            versioning: None,
            amendment_rules: Some(AmendmentRules {
                application: AmendmentModel::Merge,
            }),
            amendment_log: vec![],
            metadata: BTreeMap::new(),
        }
    }

    /// Serializes [`Self::build_profile`] as YAML. Used by
    /// `FileStore::init` to write the seeded profile to disk, and by
    /// `shapes create profile --kit <name>` to scaffold a new profile.
    pub(crate) fn to_profile_yaml(self, id: ProfileId, name: &str) -> String {
        serde_yml::to_string(&self.build_profile(id, name))
            // Serializing a hand-built Profile struct cannot fail:
            // every field is an owned, serde-ready value with no
            // dynamic key collisions.
            .expect("serializing a hand-built Profile struct never fails")
    }
}

/// Constructs a [`FieldSection`] for one node type (shape or
/// constraint) from the kit's field, kind, and source hints. Every
/// section also requires a `summary` metadata field on every
/// realization and evidence binding so that bindings explain
/// themselves to readers.
fn node_section(
    default_kind: &'static str,
    fields: &'static [FieldHint],
    kinds: &'static [KindHint],
    sources: &'static [FieldHint],
) -> FieldSection {
    FieldSection {
        default_kind: Some(default_kind.to_owned()),
        intent: Some(FieldGroup {
            fields: fields.iter().map(field_hint_to_def).collect(),
            kinds: kinds.iter().map(kind_hint_to_def).collect(),
            sources: sources.iter().map(field_hint_to_def).collect(),
        }),
        status: None,
        constraints: None,
        realization: Some(binding_summary_group()),
        evidence: Some(binding_summary_group()),
        provenance: None,
        metadata: None,
    }
}

/// Returns a `FieldGroup` requiring a `summary` metadata field on
/// every binding. Used for both realization and evidence sections.
fn binding_summary_group() -> FieldGroup {
    FieldGroup {
        fields: vec![FieldDef {
            name: "summary".to_owned(),
            description: "Brief description of what this binding points to".to_owned(),
            field_type: None,
            required: true,
        }],
        kinds: vec![],
        sources: vec![],
    }
}

fn field_hint_to_def(hint: &FieldHint) -> FieldDef {
    FieldDef {
        name: hint.name.to_owned(),
        description: hint.description.to_owned(),
        field_type: None,
        required: hint.required,
    }
}

fn kind_hint_to_def(hint: &KindHint) -> FieldDef {
    FieldDef {
        name: hint.name.to_owned(),
        description: hint.description.to_owned(),
        field_type: None,
        required: false,
    }
}

/// Source allow-list shared by every node section in the software
/// kit. Constrains who may have authored a shape or constraint to
/// `human` or `ai`, matching the project's actual authorship pattern
/// and giving INV-013 something to enforce on this repo's own profile.
static SOFTWARE_SOURCES: [FieldHint; 2] = [
    FieldHint {
        name: "human",
        description: "Hand-authored by an engineer",
        required: false,
    },
    FieldHint {
        name: "ai",
        description: "Authored by an AI agent",
        required: false,
    },
];

pub(crate) static SOFTWARE: StarterKit = StarterKit {
    name: "software",
    description: "Software engineering project",
    shape_intent_fields: &[
        FieldHint {
            name: "goals",
            description: "What this shape must achieve — concrete, observable outcomes",
            required: true,
        },
        FieldHint {
            name: "rationale",
            description: "Why this approach was chosen over alternatives",
            required: true,
        },
        FieldHint {
            name: "non_goals",
            description: "What is explicitly out of scope",
            required: false,
        },
        FieldHint {
            name: "requirements",
            description: "Specific functional requirements this shape must meet",
            required: false,
        },
    ],
    shape_sources: &SOFTWARE_SOURCES,
    constraint_sources: &SOFTWARE_SOURCES,
    shape_kinds: &[
        KindHint {
            name: "system",
            description: "Top-level system — the project as a whole",
        },
        KindHint {
            name: "service",
            description: "A long-running service or daemon",
        },
        KindHint {
            name: "feature",
            description: "A user-facing capability",
        },
        KindHint {
            name: "module",
            description: "A cohesive code module",
        },
        KindHint {
            name: "interface",
            description: "An API, contract, or boundary",
        },
        KindHint {
            name: "data-flow",
            description: "How data moves through the system",
        },
        KindHint {
            name: "pattern",
            description: "A reusable design pattern",
        },
    ],
    constraint_intent_fields: &[
        FieldHint {
            name: "rationale",
            description: "Why this rule exists — the incident or decision that created it",
            required: true,
        },
        FieldHint {
            name: "impact_if_violated",
            description: "What breaks if this rule is violated, and how badly",
            required: false,
        },
        FieldHint {
            name: "exceptions",
            description: "Known cases where this rule does not apply",
            required: false,
        },
        FieldHint {
            name: "verification_method",
            description: "How compliance is checked (test, review, lint, audit)",
            required: false,
        },
    ],
    constraint_kinds: &[
        KindHint {
            name: "invariant",
            description: "Must always hold — violating it is a bug",
        },
        KindHint {
            name: "requirement",
            description: "Functional requirement the system must meet",
        },
        KindHint {
            name: "boundary",
            description: "Hard architectural boundary",
        },
        KindHint {
            name: "guideline",
            description: "Recommended practice; deviations need justification",
        },
        KindHint {
            name: "limit",
            description: "A quantitative cap (rate, size, count, latency)",
        },
        KindHint {
            name: "policy",
            description: "Organizational or compliance rule",
        },
    ],
    default_shape_kind: "feature",
    default_constraint_kind: "invariant",
};

pub(crate) static RESEARCH: StarterKit = StarterKit {
    name: "research",
    description: "Research project — experiments, datasets, findings",
    shape_sources: &[],
    constraint_sources: &[],
    shape_intent_fields: &[
        FieldHint {
            name: "hypotheses",
            description: "What you believe is true and want to test",
            required: true,
        },
        FieldHint {
            name: "success_criteria",
            description: "Measurable conditions that confirm or refute the hypothesis",
            required: true,
        },
        FieldHint {
            name: "methodology",
            description: "How the experiment or analysis is performed",
            required: true,
        },
        FieldHint {
            name: "variables",
            description: "Independent, dependent, and controlled variables",
            required: false,
        },
        FieldHint {
            name: "prior_work",
            description: "Related research and how this builds on or differs from it",
            required: false,
        },
    ],
    shape_kinds: &[
        KindHint {
            name: "experiment",
            description: "A single experimental run or protocol",
        },
        KindHint {
            name: "dataset",
            description: "A corpus of data used as input or evidence",
        },
        KindHint {
            name: "analysis",
            description: "A processing or statistical pipeline",
        },
        KindHint {
            name: "finding",
            description: "A confirmed or refuted result",
        },
        KindHint {
            name: "hypothesis",
            description: "A testable claim",
        },
    ],
    constraint_intent_fields: &[
        FieldHint {
            name: "rationale",
            description: "Why this rule exists — methodology, ethics, or reproducibility",
            required: true,
        },
        FieldHint {
            name: "impact_if_violated",
            description: "What conclusions become unreliable if this rule is broken",
            required: false,
        },
    ],
    constraint_kinds: &[
        KindHint {
            name: "invariant",
            description: "Must always hold across runs",
        },
        KindHint {
            name: "methodology",
            description: "How experiments must be conducted",
        },
        KindHint {
            name: "ethics",
            description: "Ethical or regulatory requirement",
        },
        KindHint {
            name: "reproducibility",
            description: "Conditions needed to reproduce results",
        },
    ],
    default_shape_kind: "experiment",
    default_constraint_kind: "methodology",
};

pub(crate) static EDITORIAL: StarterKit = StarterKit {
    name: "editorial",
    description: "Editorial / writing project — books, articles, narratives",
    shape_sources: &[],
    constraint_sources: &[],
    shape_intent_fields: &[
        FieldHint {
            name: "themes",
            description: "Central themes or ideas this piece explores",
            required: true,
        },
        FieldHint {
            name: "target_audience",
            description: "Who this is written for",
            required: true,
        },
        FieldHint {
            name: "tone",
            description: "Voice and emotional register",
            required: true,
        },
        FieldHint {
            name: "narrative_arc",
            description: "How the piece moves from beginning to end",
            required: false,
        },
    ],
    shape_kinds: &[
        KindHint {
            name: "work",
            description: "A complete book, article, or series",
        },
        KindHint {
            name: "chapter",
            description: "A chapter or major section",
        },
        KindHint {
            name: "section",
            description: "A subsection or scene",
        },
        KindHint {
            name: "character",
            description: "A character or persona",
        },
        KindHint {
            name: "arc",
            description: "A narrative or thematic arc",
        },
        KindHint {
            name: "theme",
            description: "A recurring idea",
        },
    ],
    constraint_intent_fields: &[
        FieldHint {
            name: "rationale",
            description: "Why this rule exists — voice, continuity, or audience expectations",
            required: true,
        },
        FieldHint {
            name: "impact_if_violated",
            description: "What the reader will feel or notice if this is broken",
            required: false,
        },
    ],
    constraint_kinds: &[
        KindHint {
            name: "voice",
            description: "How the work sounds on the page",
        },
        KindHint {
            name: "continuity",
            description: "Internal consistency of facts, characters, and timeline",
        },
        KindHint {
            name: "style",
            description: "Stylistic conventions (formatting, capitalization, terminology)",
        },
        KindHint {
            name: "plot",
            description: "Structural rules about story shape",
        },
    ],
    default_shape_kind: "chapter",
    default_constraint_kind: "voice",
};

pub(crate) static MINIMAL: StarterKit = StarterKit {
    name: "minimal",
    description: "Minimal — only `rationale` is required, no kind hints",
    shape_sources: &[],
    constraint_sources: &[],
    shape_intent_fields: &[FieldHint {
        name: "rationale",
        description: "Why this shape exists",
        required: true,
    }],
    shape_kinds: &[],
    constraint_intent_fields: &[FieldHint {
        name: "rationale",
        description: "Why this rule exists",
        required: true,
    }],
    constraint_kinds: &[],
    default_shape_kind: "shape",
    default_constraint_kind: "constraint",
};
