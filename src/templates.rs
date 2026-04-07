//! Domain templates for `shapes init` / `shapes create`.
//!
//! A template defines the field hints and kind hints used to scaffold new
//! shapes and constraints. The template is stored in `meta.yaml` at init
//! time and read on every `shapes create` (unless overridden via
//! `--template`).
//!
//! Templates only affect the *scaffold* — the YAML emitted by `shapes
//! create` when `--from` is not used. They do **not** enforce anything;
//! enforcement is opt-in via Profiles. The template's job is to seed the
//! YAML with `TODO:` placeholders so an editor (human or agent) can see
//! every field that matters in this domain on first read.

use clap::ValueEnum;

/// A template ships a name and the field/kind hints for shapes and
/// constraints in a particular domain.
#[derive(Debug, Clone, Copy)]
pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
    pub shape_intent_fields: &'static [FieldHint],
    pub shape_kinds: &'static [KindHint],
    pub constraint_intent_fields: &'static [FieldHint],
    pub constraint_kinds: &'static [KindHint],
    pub default_shape_kind: &'static str,
    pub default_constraint_kind: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldHint {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct KindHint {
    pub name: &'static str,
    pub description: &'static str,
}

// ---------------------------------------------------------------------------
// Clap enum surface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum TemplateKind {
    /// Software engineering project — goals/rationale/non_goals, system/module/feature kinds.
    Software,
    /// Research project — hypotheses/methodology/success criteria, experiment/dataset kinds.
    Research,
    /// Editorial / writing project — themes/audience/tone, chapter/section/character kinds.
    Editorial,
    /// Bare minimum — just rationale, no kind enforcement.
    Minimal,
}

impl TemplateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateKind::Software => "software",
            TemplateKind::Research => "research",
            TemplateKind::Editorial => "editorial",
            TemplateKind::Minimal => "minimal",
        }
    }

    pub fn template(&self) -> &'static Template {
        match self {
            TemplateKind::Software => &SOFTWARE,
            TemplateKind::Research => &RESEARCH,
            TemplateKind::Editorial => &EDITORIAL,
            TemplateKind::Minimal => &MINIMAL,
        }
    }
}

/// Look up a template by its stored string name. Returns the `software`
/// template as a fallback for unknown or missing names so existing stores
/// without a `template:` field in `meta.yaml` keep working.
pub fn resolve(name: Option<&str>) -> &'static Template {
    match name {
        Some("software") => &SOFTWARE,
        Some("research") => &RESEARCH,
        Some("editorial") => &EDITORIAL,
        Some("minimal") => &MINIMAL,
        _ => &SOFTWARE,
    }
}

// ---------------------------------------------------------------------------
// Software template
// ---------------------------------------------------------------------------

pub static SOFTWARE: Template = Template {
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

// ---------------------------------------------------------------------------
// Research template
// ---------------------------------------------------------------------------

pub static RESEARCH: Template = Template {
    name: "research",
    description: "Research project — experiments, datasets, findings",
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

// ---------------------------------------------------------------------------
// Editorial template
// ---------------------------------------------------------------------------

pub static EDITORIAL: Template = Template {
    name: "editorial",
    description: "Editorial / writing project — books, articles, narratives",
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

// ---------------------------------------------------------------------------
// Minimal template
// ---------------------------------------------------------------------------

pub static MINIMAL: Template = Template {
    name: "minimal",
    description: "Minimal — only `rationale` is required, no kind hints",
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
