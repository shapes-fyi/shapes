import type { ReactNode } from "react"

export type SchemaField = {
  key: string
  label: string
  description: ReactNode
  lines: string
}

export type SchemaDefinition = {
  name: string
  description: ReactNode
  fields: [SchemaField, ...Array<SchemaField>]
}

// ─── 1. Intent ──────────────────────────────────────────────────────────────

export const INTENT_SCHEMA: SchemaDefinition = {
  name: "Intent",
  description: (
    <p>
      Every Shape, Amendment, and Constraint carries an Intent. Intent captures
      the <em>why</em>: what the record aims to achieve, what it deliberately
      excludes from scope, and how success is measured. Intent makes each record
      reviewable independently of the deliverables that embody it.
    </p>
  ),
  fields: [
    {
      key: "kind",
      label: "kind",
      description: (
        <p>
          A free-form string that classifies the intent. Common values include{" "}
          <code className="text-foreground">feature</code>,{" "}
          <code className="text-foreground">bugfix</code>,{" "}
          <code className="text-foreground">governance</code>,{" "}
          <code className="text-foreground">experiment</code>. Profiles may
          constrain the allowed set of kinds.
        </p>
      ),
      lines: "  kind: string",
    },
    {
      key: "summary",
      label: "summary",
      description: (
        <p>
          A concise human-readable description of what this record aims to
          achieve. Should be self-contained enough to understand the intent
          without reading the full record.
        </p>
      ),
      lines: "  summary: string",
    },
    {
      key: "source",
      label: "source",
      description: (
        <p>
          Records the origin of this intent &mdash; for example{" "}
          <code className="text-foreground">human</code>,{" "}
          <code className="text-foreground">ai</code>, or{" "}
          <code className="text-foreground">system</code>. The value is
          free-form; Profiles MAY constrain the allowed set of sources.
        </p>
      ),
      lines: "  source: any",
    },
    {
      key: "uris",
      label: "uris",
      description: (
        <p>
          Optional list of external references such as issue trackers, design
          documents, discussion threads, or lab notebooks that provide
          additional context for this intent.
        </p>
      ),
      lines: "  uris: [string]?",
    },
    {
      key: "open_map",
      label: "<string>: any",
      description: (
        <p>
          Intent is an open map. Each domain extends it with its own vocabulary:
          a software team might add{" "}
          <code className="text-foreground">goals</code> and{" "}
          <code className="text-foreground">non_goals</code>, a research lab{" "}
          <code className="text-foreground">hypotheses</code>, an editorial team{" "}
          <code className="text-foreground">themes</code>.
        </p>
      ),
      lines: "  <string>: any",
    },
  ],
}

// ─── 2. Shape ───────────────────────────────────────────────────────────────

export const SHAPE_SCHEMA: SchemaDefinition = {
  name: "Shape",
  description: (
    <p>
      A Shape is a record that captures what is being built, why, and what rules
      govern it. Shapes compose through inline children and cross-references by
      ID, forming a directed acyclic graph (DAG). The composition graph enables
      descriptions that range from a single unit of work to a full system
      architecture.
    </p>
  ),
  fields: [
    {
      key: "id",
      label: "id",
      description: (
        <p>
          An opaque identifier for this Shape. The specification prescribes no
          format or generation strategy &mdash; implementations may use UUIDs,
          integers, or any other scheme.
        </p>
      ),
      lines: "  id: ShapeId",
    },
    {
      key: "name",
      label: "name",
      description: (
        <p>
          A human-readable name for this Shape. Should be concise and
          descriptive enough to identify the Shape in listings and references.
        </p>
      ),
      lines: "  name: string",
    },
    {
      key: "description",
      label: "description",
      description: (
        <p>
          A longer description providing context about what this Shape
          represents, its purpose, and any relevant background information.
        </p>
      ),
      lines: "  description: string",
    },
    {
      key: "profile",
      label: "profile",
      description: (
        <p>
          Optional reference to the Profile that governs this Shape's lifecycle
          and validation rules: what each gate requires, what canonical means,
          and how amendments are applied.
        </p>
      ),
      lines: "  profile: ProfileId?",
    },
    {
      key: "version",
      label: "version",
      description: (
        <p>
          An opaque string whose interpretation is profile-defined. When an
          Amendment reaches Canonical, the target's version is updated according
          to profile rules.
        </p>
      ),
      lines: "  version: string?",
    },
    {
      key: "predecessors",
      label: "predecessors",
      description: (
        <p>
          Optional list of Shape IDs that this Shape replaces or continues from.
          Predecessors MUST be in a terminal state. Used together with{" "}
          <code className="text-foreground">status.successors</code> on terminal
          states to express lineage: one-to-one is a replacement, one-to-many is
          a decomposition, many-to-one is a consolidation.
        </p>
      ),
      lines: "  predecessors: [ShapeId]?",
    },
    {
      key: "status",
      label: "status",
      description: (
        <p>
          A tagged union representing the lifecycle state. The state name is the
          key, and each state carries only its valid fields. All terminal states
          may carry an optional{" "}
          <code className="text-foreground">successors</code> field pointing to
          replacement records; for{" "}
          <code className="text-foreground">superseded</code> it is required.
          State transitions are governed by preconditions and postconditions
          defined by the profile.
        </p>
      ),
      lines: `  status:
    proposed | promoted | canonical:
      reason: string?
      uris: [string]?
      metadata: <string>: any
    rejected | superseded | abandoned | reverted:
      reason: string?
      uris: [string]?
      successors: [ShapeId]?
      metadata: <string>: any`,
    },
    {
      key: "intent",
      label: "intent",
      description: (
        <p>
          The Intent block captures the <em>why</em>: what the Shape aims to
          achieve, what it deliberately excludes from scope, and how success is
          measured. See the Intent schema for its structure.
        </p>
      ),
      lines: "  intent: Intent",
    },
    {
      key: "constraints",
      label: "constraints",
      description: (
        <p>
          Optional list of Constraint IDs referencing standalone Constraints
          that apply to this Shape.
        </p>
      ),
      lines: "  constraints: [ConstraintId]?",
    },
    {
      key: "realization",
      label: "realization",
      description: (
        <p>
          Optional list of Realization records connecting this Shape to the
          concrete deliverables that embody it. A binding may also reference
          other Shapes, enabling cross-project composition.
        </p>
      ),
      lines: "  realization: [Realization]?",
    },
    {
      key: "evidence",
      label: "evidence",
      description: (
        <p>
          Optional list of Evidence records establishing whether deliverables
          satisfy this Shape's requirements. May include verification reports,
          benchmarks, reviews, or attestations.
        </p>
      ),
      lines: "  evidence: [Evidence]?",
    },
    {
      key: "provenance",
      label: "provenance",
      description: (
        <p>
          Optional list of Provenance records linking this Shape to its origin
          process. Provenance records operational history while Shapes record
          semantic state.
        </p>
      ),
      lines: "  provenance: [Provenance]?",
    },
    {
      key: "amendment_log",
      label: "amendment_log",
      description: (
        <p>
          Append-only list of Amendment IDs that have been applied to this
          Shape. Entries appear in the order they were applied. Each Amendment
          modifies the Shape's fields and is tracked here for auditability.
        </p>
      ),
      lines: "  amendment_log: [AmendmentId]?",
    },
    {
      key: "parents",
      label: "parents",
      description: (
        <p>
          The inverse link for bidirectional traversal. Each entry references a
          parent Shape and carries an optional{" "}
          <code className="text-foreground">role</code> (e.g.{" "}
          <code className="text-foreground">component</code>,{" "}
          <code className="text-foreground">chapter</code>) and{" "}
          <code className="text-foreground">reason</code>.
        </p>
      ),
      lines: `  parents:
    - id: ShapeId
      role: string?
      reason: string?`,
    },
    {
      key: "children",
      label: "children",
      description: (
        <p>
          Inline children owned by this Shape, or cross-references by ID to
          Shapes that may live in a different project or organization. When the
          value is an object it is an inline Shape definition; when it is a
          scalar it is a reference to a standalone Shape by ID. Each child
          carries an optional <code className="text-foreground">role</code> and{" "}
          <code className="text-foreground">reason</code>. A Shape may appear as
          a child of multiple parents; the composition graph is a DAG.
        </p>
      ),
      lines: `  children:
    - shape: Shape | ShapeId
      role: string?
      reason: string?`,
    },
    {
      key: "metadata",
      label: "metadata",
      description: (
        <p>
          Optional open map for domain-specific metadata. Can hold any key-value
          pairs that the profile or project requires but are not covered by the
          standard fields.
        </p>
      ),
      lines: `  metadata:
    <string>: any`,
    },
  ],
}

// ─── 3. Amendment ───────────────────────────────────────────────────────────

export const AMENDMENT_SCHEMA: SchemaDefinition = {
  name: "Amendment",
  description: (
    <p>
      Once a Shape or Constraint reaches Promoted or Canonical, all further
      changes must be recorded as Amendments &mdash; immutable records that
      follow a five-state lifecycle (proposed, promoted, canonical, rejected,
      abandoned). Amendments carry the same semantic fields as Shapes (intent,
      constraints, realizations, evidence) because a profile may require any of
      these before granting Canonical status.
    </p>
  ),
  fields: [
    {
      key: "id",
      label: "id",
      description: (
        <p>
          An opaque identifier for this Amendment. Like Shape IDs, the
          specification prescribes no format.
        </p>
      ),
      lines: "  id: AmendmentId",
    },
    {
      key: "name",
      label: "name",
      description: (
        <p>
          A human-readable name for this Amendment, describing the change being
          proposed.
        </p>
      ),
      lines: "  name: string",
    },
    {
      key: "description",
      label: "description",
      description: (
        <p>
          A longer description providing context about what this Amendment
          changes and why.
        </p>
      ),
      lines: "  description: string",
    },
    {
      key: "targets",
      label: "targets",
      description: (
        <p>
          The Shapes, Constraints, and/or Profiles this Amendment modifies. An
          Amendment MUST target at least one node and may target any combination
          of these node types.
        </p>
      ),
      lines: `  targets:
    shape_ids: [ShapeId]?
    constraint_ids: [ConstraintId]?
    profile_ids: [ProfileId]?`,
    },
    {
      key: "status",
      label: "status",
      description: (
        <p>
          Lifecycle state of this Amendment. Amendments use five states &mdash;{" "}
          <code className="text-foreground">superseded</code> and{" "}
          <code className="text-foreground">reverted</code> do not apply to
          amendments. Terminal states (rejected, abandoned) may optionally carry
          a <code className="text-foreground">successors</code> field.
        </p>
      ),
      lines: `  status:
    proposed | promoted | canonical:
      reason: string?
      uris: [string]?
      metadata: <string>: any
    rejected | abandoned:
      reason: string?
      uris: [string]?
      successors: [AmendmentId]?
      metadata: <string>: any`,
    },
    {
      key: "version_impact",
      label: "version_impact",
      description: (
        <p>
          Indicates the magnitude of the change under the profile's versioning
          scheme (e.g. <code className="text-foreground">major</code>,{" "}
          <code className="text-foreground">minor</code>,{" "}
          <code className="text-foreground">patch</code>). When this Amendment
          reaches Canonical, the target's version is updated per the profile's
          rules.
        </p>
      ),
      lines: "  version_impact: string?",
    },
    {
      key: "intent",
      label: "intent",
      description: (
        <p>
          The Intent block for this Amendment, capturing why the change is being
          made. Follows the same Intent schema as Shapes.
        </p>
      ),
      lines: "  intent: Intent",
    },
    {
      key: "constraints",
      label: "constraints",
      description: (
        <p>Optional Constraint IDs introduced or modified by this Amendment.</p>
      ),
      lines: "  constraints: [ConstraintId]?",
    },
    {
      key: "realization",
      label: "realization",
      description: (
        <p>
          Optional realization records linking this Amendment to the concrete
          deliverables that implement the change.
        </p>
      ),
      lines: "  realization: [Realization]?",
    },
    {
      key: "evidence",
      label: "evidence",
      description: (
        <p>
          Optional evidence records verifying that the Amendment's changes
          satisfy its requirements.
        </p>
      ),
      lines: "  evidence: [Evidence]?",
    },
    {
      key: "provenance",
      label: "provenance",
      description: (
        <p>
          Optional list of Provenance records linking this Amendment to its
          origin process.
        </p>
      ),
      lines: "  provenance: [Provenance]?",
    },
    {
      key: "initiated_by",
      label: "initiated_by",
      description: (
        <p>
          Records the actor who created this Amendment. The{" "}
          <code className="text-foreground">type</code> distinguishes human from
          machine actors, and <code className="text-foreground">identity</code>{" "}
          provides the specific identifier.
        </p>
      ),
      lines: `  initiated_by:
    type: string
    identity: string?
    provenance: string?`,
    },
    {
      key: "metadata",
      label: "metadata",
      description: (
        <p>
          Optional open map for domain-specific metadata about this Amendment.
        </p>
      ),
      lines: `  metadata:
    <string>: any`,
    },
  ],
}

// ─── 4. Constraint ──────────────────────────────────────────────────────────

export const CONSTRAINT_SCHEMA: SchemaDefinition = {
  name: "Constraint",
  description: (
    <p>
      Constraints are standalone records with their own lifecycle, referenced by
      ID from any number of Shapes and Amendments. Constraints do not
      participate in the Shape composition graph but form their own directed
      acyclic graph through <code className="text-foreground">parents</code> and{" "}
      <code className="text-foreground">children</code> fields, enabling
      decomposition of complex policies into sub-constraints.
    </p>
  ),
  fields: [
    {
      key: "id",
      label: "id",
      description: (
        <p>
          An opaque identifier for this Constraint. Like Shape IDs, the
          specification prescribes no format.
        </p>
      ),
      lines: "  id: ConstraintId",
    },
    {
      key: "name",
      label: "name",
      description: (
        <p>
          A human-readable name for this Constraint. Should be concise and
          descriptive enough to identify the Constraint in listings and
          references.
        </p>
      ),
      lines: "  name: string",
    },
    {
      key: "description",
      label: "description",
      description: (
        <p>
          A longer description providing context about what this Constraint
          enforces and why.
        </p>
      ),
      lines: "  description: string",
    },
    {
      key: "kind",
      label: "kind",
      description: (
        <p>
          Classifies the constraint. Common values:{" "}
          <code className="text-foreground">invariant</code>,{" "}
          <code className="text-foreground">requirement</code>,{" "}
          <code className="text-foreground">boundary</code>,{" "}
          <code className="text-foreground">guideline</code>,{" "}
          <code className="text-foreground">limit</code>,{" "}
          <code className="text-foreground">policy</code>. This is an open
          string; profiles may define additional values.
        </p>
      ),
      lines: "  kind: string",
    },
    {
      key: "rule",
      label: "rule",
      description: (
        <p>
          A human-readable statement of the constraint in natural language. This
          is the requirement that must be satisfied &mdash; what the constraint
          enforces.
        </p>
      ),
      lines: "  rule: string",
    },
    {
      key: "enforcement",
      label: "enforcement",
      description: (
        <p>
          How the constraint is enforced. Common values:{" "}
          <code className="text-foreground">machine</code> (automated checks),{" "}
          <code className="text-foreground">human</code> (manual review),{" "}
          <code className="text-foreground">hybrid</code> (both). This is an
          open string; profiles may define additional values.
        </p>
      ),
      lines: "  enforcement: string",
    },
    {
      key: "profile",
      label: "profile",
      description: (
        <p>
          Optional reference to the Profile that governs this Constraint's
          lifecycle and validation rules.
        </p>
      ),
      lines: "  profile: ProfileId?",
    },
    {
      key: "version",
      label: "version",
      description: (
        <p>
          An opaque string whose interpretation is profile-defined. When an
          Amendment reaches Canonical, the target's version is updated according
          to profile rules.
        </p>
      ),
      lines: "  version: string?",
    },
    {
      key: "status",
      label: "status",
      description: (
        <p>
          A tagged union representing the lifecycle state. Follows the same
          structure as Shape status, with optional{" "}
          <code className="text-foreground">successors</code> on terminal
          states; required for{" "}
          <code className="text-foreground">superseded</code>.
        </p>
      ),
      lines: `  status:
    proposed | promoted | canonical:
      reason: string?
      uris: [string]?
      metadata: <string>: any
    rejected | superseded | abandoned | reverted:
      reason: string?
      uris: [string]?
      successors: [ConstraintId]?
      metadata: <string>: any`,
    },
    {
      key: "intent",
      label: "intent",
      description: (
        <p>
          The Intent block captures the <em>why</em>: what the Constraint aims
          to enforce and the reasoning behind it.
        </p>
      ),
      lines: "  intent: Intent",
    },
    {
      key: "realization",
      label: "realization",
      description: (
        <p>
          Optional list of Realization records connecting this Constraint to the
          concrete implementations that enforce it.
        </p>
      ),
      lines: "  realization: [Realization]?",
    },
    {
      key: "evidence",
      label: "evidence",
      description: (
        <p>
          Optional list of Evidence records establishing whether the Constraint
          is being satisfied.
        </p>
      ),
      lines: "  evidence: [Evidence]?",
    },
    {
      key: "provenance",
      label: "provenance",
      description: (
        <p>
          Optional list of Provenance records linking this Constraint to its
          origin process.
        </p>
      ),
      lines: "  provenance: [Provenance]?",
    },
    {
      key: "amendment_log",
      label: "amendment_log",
      description: (
        <p>
          Append-only list of Amendment IDs that have been applied to this
          Constraint. Entries appear in the order they were applied.
        </p>
      ),
      lines: "  amendment_log: [AmendmentId]?",
    },
    {
      key: "parents",
      label: "parents",
      description: (
        <p>
          The inverse link for bidirectional traversal. Each entry references a
          parent Constraint and carries an optional{" "}
          <code className="text-foreground">role</code> and{" "}
          <code className="text-foreground">reason</code>.
        </p>
      ),
      lines: `  parents:
    - id: ConstraintId
      role: string?
      reason: string?`,
    },
    {
      key: "children",
      label: "children",
      description: (
        <p>
          Inline children owned by this Constraint, or cross-references by ID to
          Constraints that may live in a different project or organization. When
          the value is an object it is an inline Constraint definition; when it
          is a scalar it is a reference to a standalone Constraint by ID. A
          Constraint may appear as a child of multiple parents; the composition
          graph is a DAG.
        </p>
      ),
      lines: `  children:
    - constraint: Constraint | ConstraintId
      role: string?
      reason: string?`,
    },
    {
      key: "metadata",
      label: "metadata",
      description: (
        <p>
          Optional open map for domain-specific metadata about this Constraint.
        </p>
      ),
      lines: `  metadata:
    <string>: any`,
    },
  ],
}

// ─── 5. Binding ─────────────────────────────────────────────────────────────

export const BINDING_SCHEMA: SchemaDefinition = {
  name: "Binding",
  description: (
    <p>
      Bindings connect Shapes to concrete deliverables, validation, and origin
      records. All external references use this generic model. The{" "}
      <code className="text-foreground">scheme</code> identifies the reference
      type; profiles may constrain which schemes are accepted.
    </p>
  ),
  fields: [
    {
      key: "scheme",
      label: "scheme",
      description: (
        <p>
          Identifies the reference type. Common schemes:{" "}
          <code className="text-foreground">uri</code>,{" "}
          <code className="text-foreground">doi</code>,{" "}
          <code className="text-foreground">shape</code>,{" "}
          <code className="text-foreground">trace</code>, or any domain-specific
          scheme.
        </p>
      ),
      lines: "  scheme: string",
    },
    {
      key: "value",
      label: "value",
      description: (
        <p>
          The reference value interpreted according to the scheme. For a{" "}
          <code className="text-foreground">uri</code> scheme this would be a
          URL; for <code className="text-foreground">doi</code> a DOI string.
        </p>
      ),
      lines: "  value: string",
    },
    {
      key: "metadata",
      label: "metadata",
      description: (
        <p>
          Optional open map for additional context about this binding that
          doesn't fit into the standard fields.
        </p>
      ),
      lines: `  metadata:
    <string>: any`,
    },
  ],
}

// ─── 6. Realization ─────────────────────────────────────────────────────────

export const REALIZATION_SCHEMA: SchemaDefinition = {
  name: "Realization",
  description: (
    <p>
      A Realization connects a Shape to the deliverables that embody it. A
      binding may also reference other Shapes, enabling cross-project
      composition.
    </p>
  ),
  fields: [
    {
      key: "bindings",
      label: "bindings",
      description: (
        <p>
          List of Binding records pointing to the concrete artifacts that
          realize this Shape &mdash; source files, services, documents, or other
          Shapes.
        </p>
      ),
      lines: "  bindings: [Binding]",
    },
    {
      key: "role",
      label: "role",
      description: (
        <p>
          Classifies the realization. Common roles:{" "}
          <code className="text-foreground">primary</code>,{" "}
          <code className="text-foreground">supporting</code>,{" "}
          <code className="text-foreground">interface</code>,{" "}
          <code className="text-foreground">verification</code>,{" "}
          <code className="text-foreground">migration</code>,{" "}
          <code className="text-foreground">docs</code>. Profiles may define
          additional roles.
        </p>
      ),
      lines: "  role: string",
    },
  ],
}

// ─── 7. Evidence ────────────────────────────────────────────────────────────

export const EVIDENCE_SCHEMA: SchemaDefinition = {
  name: "Evidence",
  description: (
    <p>
      Evidence is distinct from realization. Realization identifies{" "}
      <em>the deliverables that embody a Shape</em>; Evidence establishes{" "}
      <em>whether those deliverables satisfy its requirements</em>. Evidence
      records may include verification reports, benchmarks, reviews, or
      attestations.
    </p>
  ),
  fields: [
    {
      key: "id",
      label: "id",
      description: (
        <p>
          A unique identifier for this Evidence record, allowing it to be
          referenced from other records.
        </p>
      ),
      lines: "  id: string",
    },
    {
      key: "type",
      label: "type",
      description: (
        <p>
          Classifies the evidence. Common types:{" "}
          <code className="text-foreground">test_report</code>,{" "}
          <code className="text-foreground">review</code>,{" "}
          <code className="text-foreground">dataset</code>,{" "}
          <code className="text-foreground">attestation</code>.
        </p>
      ),
      lines: "  type: string",
    },
    {
      key: "bindings",
      label: "bindings",
      description: (
        <p>
          List of Binding records pointing to the evidence artifacts &mdash;
          test reports, review documents, benchmark results, etc.
        </p>
      ),
      lines: "  bindings: [Binding]",
    },
    {
      key: "trusted",
      label: "trusted",
      description: (
        <p>
          Optional flag indicating whether the evidence source is considered
          authoritative. Useful for distinguishing first-party verification from
          third-party attestations.
        </p>
      ),
      lines: "  trusted: boolean?",
    },
    {
      key: "metadata",
      label: "metadata",
      description: (
        <p>
          Optional open map for additional context about this evidence record.
        </p>
      ),
      lines: `  metadata:
    <string>: any`,
    },
  ],
}

// ─── 8. Provenance ──────────────────────────────────────────────────────────

export const PROVENANCE_SCHEMA: SchemaDefinition = {
  name: "Provenance",
  description: (
    <p>
      Provenance links a record to its origin process. Provenance records
      operational history; Shapes record semantic state. This separation keeps
      the specification focused on intent and structure while delegating
      contributor attribution and revision-scoped details to external systems.
    </p>
  ),
  fields: [
    {
      key: "type",
      label: "type",
      description: (
        <p>
          Classifies the provenance record. Identifies the kind of origin
          process (e.g. a code review, an automated pipeline, a manual
          operation).
        </p>
      ),
      lines: "  type: string",
    },
    {
      key: "bindings",
      label: "bindings",
      description: (
        <p>
          List of Binding records pointing to the origin artifacts &mdash;
          commits, traces, pipeline runs, or external system records.
        </p>
      ),
      lines: "  bindings: [Binding]",
    },
    {
      key: "metadata",
      label: "metadata",
      description: (
        <p>Optional open map for additional context about the provenance.</p>
      ),
      lines: `  metadata:
    <string>: any`,
    },
  ],
}

// ─── 9. Profile ─────────────────────────────────────────────────────────────

export const PROFILE_SCHEMA: SchemaDefinition = {
  name: "Profile",
  description: (
    <p>
      A Profile is a first-class lifecycle governance node that follows the same
      lifecycle as Shapes and Amendments. It defines what each gate requires,
      what canonical means, how amendments are applied, and which custom fields
      and kinds are recognized. A single Profile MAY govern both Shapes and
      Constraints through separate field declaration sections. Every custom
      field and kind carries a description so that agents and users can
      understand what each value means without external documentation.
    </p>
  ),
  fields: [
    {
      key: "id",
      label: "id",
      description: <p>An opaque identifier for this Profile.</p>,
      lines: "  id: ProfileId",
    },
    {
      key: "name",
      label: "name",
      description: <p>A human-readable name for this Profile.</p>,
      lines: "  name: string",
    },
    {
      key: "description",
      label: "description",
      description: (
        <p>
          A longer description of the Profile's purpose and the domain it
          governs.
        </p>
      ),
      lines: "  description: string",
    },
    {
      key: "version",
      label: "version",
      description: (
        <p>
          Optional version of this Profile, following whatever scheme the
          Profile itself defines.
        </p>
      ),
      lines: "  version: string?",
    },
    {
      key: "status",
      label: "status",
      description: (
        <p>
          Lifecycle state of this Profile. Follows the same tagged union
          structure as Shape status, including optional{" "}
          <code className="text-foreground">successors</code> on terminal states
          and required <code className="text-foreground">successors</code> on{" "}
          <code className="text-foreground">superseded</code>.
        </p>
      ),
      lines: `  status:
    proposed | promoted | canonical:
      reason: string?
      uris: [string]?
      metadata: <string>: any
    rejected | superseded | abandoned | reverted:
      reason: string?
      uris: [string]?
      successors: [ProfileId]?
      metadata: <string>: any`,
    },
    {
      key: "intent",
      label: "intent",
      description: (
        <p>
          The Intent block for this Profile, capturing why this governance
          configuration exists and what it aims to achieve.
        </p>
      ),
      lines: "  intent: Intent",
    },
    {
      key: "provenance",
      label: "provenance",
      description: (
        <p>
          Optional list of Provenance records linking this Profile to its origin
          process and decision history.
        </p>
      ),
      lines: "  provenance: [Provenance]?",
    },
    {
      key: "lifecycle",
      label: "lifecycle",
      description: (
        <p>
          Defines the allowed state transitions and their gate conditions for
          records governed by this Profile. Profiles MAY define custom statuses
          beyond the base set (proposed, promoted, canonical, rejected,
          superseded, abandoned, reverted) via the{" "}
          <code className="text-foreground">statuses</code> list. Each gate
          specifies a <code className="text-foreground">from</code> and{" "}
          <code className="text-foreground">to</code> state, with optional
          preconditions and postconditions that must be satisfied.
        </p>
      ),
      lines: `  lifecycle:
    statuses:
      - name: string
        description: string
        type: progressive | terminal
    gates:
      - from: status
        to: status
        preconditions: [string]?
        postconditions: [string]?`,
    },
    {
      key: "fields",
      label: "fields",
      description: (
        <p>
          Declares which sub-fields are required or recognized within core
          fields. The <code className="text-foreground">shape</code> section
          governs Shapes that reference this Profile; the{" "}
          <code className="text-foreground">constraint</code> section governs
          Constraints. Either or both MAY be defined. For example, a software
          profile might require that Shape Intent includes{" "}
          <code className="text-foreground">requirements</code> and{" "}
          <code className="text-foreground">acceptance_criteria</code>, while a
          research profile might require{" "}
          <code className="text-foreground">hypotheses</code>.
        </p>
      ),
      lines: `  fields:
    shape:
      intent:
        fields: [FieldDef]?
        kinds: [FieldDef]?
        sources: [FieldDef]?
      status:
        fields: [FieldDef]?
      constraints:
        fields: [FieldDef]?
        kinds: [FieldDef]?
      realization:
        fields: [FieldDef]?
        kinds: [FieldDef]?
      evidence:
        fields: [FieldDef]?
        kinds: [FieldDef]?
      provenance:
        fields: [FieldDef]?
        kinds: [FieldDef]?
      metadata:
        fields: [FieldDef]?
    constraint:
      intent:
        fields: [FieldDef]?
        kinds: [FieldDef]?
        sources: [FieldDef]?
      status:
        fields: [FieldDef]?
      realization:
        fields: [FieldDef]?
        kinds: [FieldDef]?
      evidence:
        fields: [FieldDef]?
        kinds: [FieldDef]?
      provenance:
        fields: [FieldDef]?
        kinds: [FieldDef]?
      metadata:
        fields: [FieldDef]?`,
    },
    {
      key: "versioning",
      label: "versioning",
      description: (
        <p>
          Defines the versioning scheme used by records under this Profile (e.g.
          semver) and optional rules for how Amendments bump the version.
        </p>
      ),
      lines: `  versioning:
    scheme: string
    bump_rules: string?`,
    },
    {
      key: "amendment_rules",
      label: "amendment_rules",
      description: (
        <p>
          Defines how canonical Amendments are applied to their targets. The{" "}
          <code className="text-foreground">application</code> field specifies
          the model: merge, overlay, edition, or append-only.
        </p>
      ),
      lines: `  amendment_rules:
    application: string`,
    },
    {
      key: "amendment_log",
      label: "amendment_log",
      description: (
        <p>
          Append-only list of Amendment IDs that have been applied to this
          Profile. Entries appear in the order they were applied.
        </p>
      ),
      lines: "  amendment_log: [AmendmentId]?",
    },
    {
      key: "metadata",
      label: "metadata",
      description: (
        <p>
          Optional open map for domain-specific metadata about this Profile.
        </p>
      ),
      lines: `  metadata:
    <string>: any`,
    },
  ],
}

// ─── 10. FieldDef ───────────────────────────────────────────────────────────

export const FIELDDEF_SCHEMA: SchemaDefinition = {
  name: "FieldDef",
  description: (
    <p>
      Defines a custom field within a Profile's field declarations. Each
      FieldDef describes a field recognized within intent, status, constraints,
      realization, evidence, provenance, or metadata sections, and indicates
      whether the field is required via a boolean flag.
    </p>
  ),
  fields: [
    {
      key: "name",
      label: "name",
      description: <p>The name of the custom field being defined.</p>,
      lines: "  name: string",
    },
    {
      key: "description",
      label: "description",
      description: (
        <p>
          A human-readable explanation of what this field means, so that agents
          and users can understand it without external documentation.
        </p>
      ),
      lines: "  description: string",
    },
    {
      key: "type",
      label: "type",
      description: (
        <p>
          Optional type annotation for this field. When omitted, the field
          accepts any value.
        </p>
      ),
      lines: "  type: string?",
    },
    {
      key: "required",
      label: "required",
      description: (
        <p>
          Whether this field must be present on records governed by the Profile.
          Defaults to <code className="text-foreground">false</code> when
          omitted.
        </p>
      ),
      lines: "  required: boolean?",
    },
  ],
}
