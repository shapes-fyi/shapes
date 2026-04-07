import type { ReactNode } from "react"
import { CodeBlock } from "@/components/spec/code-block"
import { ExampleToggle } from "@/components/spec/example-toggle"
import {
  AMENDMENT_SCHEMA,
  BINDING_SCHEMA,
  CONSTRAINT_SCHEMA,
  EVIDENCE_SCHEMA,
  FIELDDEF_SCHEMA,
  INTENT_SCHEMA,
  PROFILE_SCHEMA,
  PROVENANCE_SCHEMA,
  REALIZATION_SCHEMA,
  SHAPE_SCHEMA,
} from "@/components/spec/schema-data"
import { SchemaExplorer } from "@/components/spec/schema-explorer"

export type SpecSubsection = {
  id: string
  title: string
  content: ReactNode
}

export type SpecSection = {
  id: string
  title: string
  content: ReactNode
  subsections?: Array<SpecSubsection>
}

export const SPEC_META = [
  { label: "Version", value: "0.1.0" },
  { label: "Status", value: "Working Draft" },
  { label: "Date", value: "March 2026" },
] as const

const linkClass =
  "text-foreground underline decoration-muted-foreground/60 underline-offset-4 transition hover:text-primary hover:decoration-primary"

export const sections: Array<SpecSection> = [
  // ── §1 The Intent Layer ───────────────────────────────────────────────────
  {
    id: "the-intent-layer",
    title: "The Intent Layer",
    content: (
      <div className="space-y-6">
        <p>
          Shapes is an open specification that defines a semantic layer for
          collaboration between agents and humans. This layer — the intent layer
          — captures <em>what</em> is to be built and <em>why</em>, while lower
          layers address <em>how</em> it is done.
        </p>

        <p>The specification captures the following explicitly:</p>

        <ul className="space-y-2 pl-5 marker:text-primary">
          <li>
            <strong className="text-foreground">Intent</strong> — the purpose
            and motivation behind a unit of work.
          </li>
          <li>
            <strong className="text-foreground">Constraints</strong> — strict
            invariants and enforcement rules.
          </li>
          <li>
            <strong className="text-foreground">Bindings</strong> — connections
            to external artifacts, verification, and history.
            <ul className="mt-1.5 space-y-1 pl-5 marker:text-primary/60">
              <li>Realization — deliverables that fulfill a Shape.</li>
              <li>Evidence — proof that requirements are satisfied.</li>
              <li>Provenance — decision history and origin records.</li>
            </ul>
          </li>
          <li>
            <strong className="text-foreground">Amendments</strong> — immutable
            change records applied to canonical nodes.
          </li>
          <li>
            <strong className="text-foreground">Boundaries</strong> — scope
            derived from the graph structure.
          </li>
        </ul>

        <p>
          Shapes sits between agents and the work that must be done. It provides
          a structured representation of the entire history and current state of
          a project, enabling agents to operate with full context without
          reconstructing it from raw artifacts alone.
        </p>

        <p>
          The specification does not require datetime values at the base level.
          Profiles MAY declare date fields as required within status metadata or
          other sections.
        </p>

        <figure className="full-bleed flex justify-center py-2">
          <svg
            viewBox="0 0 420 300"
            className="w-full max-w-md"
            role="img"
            aria-label="Shapes sits as a layer between Agents and Work, containing Shapes, Constraints, Amendments, and Profiles"
          >
            {/* Human */}
            <rect
              x="16"
              y="8"
              width="388"
              height="42"
              rx="8"
              fill="none"
              stroke="var(--foreground)"
              strokeWidth="1.5"
              opacity="0.25"
            />
            <text
              x="210"
              y="34"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="15"
              fontFamily="var(--font-serif)"
              opacity="0.6"
            >
              Human
            </text>

            {/* Agents */}
            <rect
              x="16"
              y="62"
              width="388"
              height="42"
              rx="8"
              fill="none"
              stroke="var(--foreground)"
              strokeWidth="1.5"
              opacity="0.25"
            />
            <text
              x="210"
              y="88"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="15"
              fontFamily="var(--font-serif)"
              opacity="0.6"
            >
              Agents
            </text>

            {/* Shapes container */}
            <rect
              x="16"
              y="116"
              width="388"
              height="120"
              rx="10"
              fill="var(--primary)"
              opacity="0.12"
            />
            <rect
              x="16"
              y="116"
              width="388"
              height="120"
              rx="10"
              fill="none"
              stroke="var(--primary)"
              strokeWidth="1.5"
              opacity="0.45"
            />
            <text
              x="210"
              y="140"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="15"
              fontWeight="600"
              fontFamily="var(--font-serif)"
            >
              Shapes
            </text>

            {/* Inner: Shapes */}
            <rect
              x="32"
              y="150"
              width="176"
              height="34"
              rx="7"
              fill="var(--primary)"
              opacity="0.13"
            />
            <rect
              x="32"
              y="150"
              width="176"
              height="34"
              rx="7"
              fill="none"
              stroke="var(--primary)"
              strokeWidth="1"
              opacity="0.35"
            />
            <text
              x="120"
              y="172"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="13"
              fontFamily="var(--font-serif)"
              opacity="0.7"
            >
              Shapes
            </text>

            {/* Inner: Constraints */}
            <rect
              x="216"
              y="150"
              width="176"
              height="34"
              rx="7"
              fill="var(--primary)"
              opacity="0.13"
            />
            <rect
              x="216"
              y="150"
              width="176"
              height="34"
              rx="7"
              fill="none"
              stroke="var(--primary)"
              strokeWidth="1"
              opacity="0.35"
            />
            <text
              x="304"
              y="172"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="13"
              fontFamily="var(--font-serif)"
              opacity="0.7"
            >
              Constraints
            </text>

            {/* Inner: Amendments */}
            <rect
              x="32"
              y="192"
              width="176"
              height="34"
              rx="7"
              fill="var(--primary)"
              opacity="0.13"
            />
            <rect
              x="32"
              y="192"
              width="176"
              height="34"
              rx="7"
              fill="none"
              stroke="var(--primary)"
              strokeWidth="1"
              opacity="0.35"
            />
            <text
              x="120"
              y="214"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="13"
              fontFamily="var(--font-serif)"
              opacity="0.7"
            >
              Amendments
            </text>

            {/* Inner: Profiles */}
            <rect
              x="216"
              y="192"
              width="176"
              height="34"
              rx="7"
              fill="var(--primary)"
              opacity="0.13"
            />
            <rect
              x="216"
              y="192"
              width="176"
              height="34"
              rx="7"
              fill="none"
              stroke="var(--primary)"
              strokeWidth="1"
              opacity="0.35"
            />
            <text
              x="304"
              y="214"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="13"
              fontFamily="var(--font-serif)"
              opacity="0.7"
            >
              Profiles
            </text>

            {/* Work */}
            <rect
              x="16"
              y="248"
              width="388"
              height="42"
              rx="8"
              fill="none"
              stroke="var(--foreground)"
              strokeWidth="1.5"
              opacity="0.25"
            />
            <text
              x="210"
              y="274"
              textAnchor="middle"
              fill="var(--foreground)"
              fontSize="15"
              fontFamily="var(--font-serif)"
              opacity="0.6"
            >
              Work
            </text>
          </svg>
        </figure>
      </div>
    ),
  },
  // ── §2 Motivation ─────────────────────────────────────────────────────────
  {
    id: "motivation",
    title: "Motivation",
    content: (
      <div className="space-y-6">
        <p>
          Existing tools — version control, task tracking, review workflows,
          editorial pipelines — record <em>what changed</em> but not{" "}
          <em>why it matters</em>. Provenance-only systems capture session
          history and decision traces, yet they do not transform that history
          into a queryable, semantically structured context layer. The gap
          between raw historical records and structured intent remains.
        </p>

        <p>
          As projects grow in complexity, agents require a shared semantic
          surface to plan, build, and review work without reconstructing context
          from code and artifacts alone. Human review does not scale to the
          volume of concurrent changes that parallelized agent workflows
          produce. Local AI reviewers operate at the granular level without
          visibility into the broader intent and constraints of the project as a
          whole.
        </p>

        <p>
          Shapes addresses this gap. The specification captures intent,
          constraints, bindings, and amendment history as first-class structured
          records, forming a queryable graph that any agent — regardless of
          architecture — can read, evaluate, and act on.
        </p>
      </div>
    ),
  },
  // ── §3 The DAG ────────────────────────────────────────────────────────────
  {
    id: "the-dag",
    title: "The DAG",
    content: (
      <div className="space-y-6">
        <p>
          The specification is format-agnostic and storage-agnostic. Examples in
          this specification use YAML for readability; implementations MAY use
          any serialization format.
        </p>

        <p>
          The Shapes Specification is architected as a Directed Acyclic Graph
          (DAG). The specification maintains two composition DAGs: the Shape
          composition graph and the Constraint composition graph. These
          structures express systems of systems — where nodes compose
          hierarchically and cross-reference laterally without cycles.
        </p>

        <p>The specification defines four node types:</p>

        <ul className="space-y-2 pl-5 marker:text-primary">
          <li>
            <strong className="text-foreground">Shape</strong> — the primary
            node, describing the intent and work to be done.
          </li>
          <li>
            <strong className="text-foreground">Constraint</strong> — a strict
            enforcement rule that MAY be applied to Shapes, Amendments, or other
            Constraints.
          </li>
          <li>
            <strong className="text-foreground">Amendment</strong> — an
            immutable change record applied to Shapes, Constraints, or Profiles
            to evolve the graph over time.
          </li>
          <li>
            <strong className="text-foreground">Profile</strong> — a governance
            configuration defining the semantic meanings, lifecycles, and
            validation rules for nodes in the graph. A Profile MAY encode a
            domain-specific lifecycle such as an SDLC or ADLC (Agentic
            Development Life Cycle).
          </li>
        </ul>

        <p>
          These node types compose to form a DAG that can describe a single
          system or an arbitrarily deep hierarchy of systems. Shapes serve as
          the primary structural nodes. Constraints enforce invariants across
          them and may compose hierarchically through their own parent/child
          relationships. Amendments preserve the lineage of changes while
          keeping targeted nodes lean — Shapes, Constraints, and Profiles
          describe the current state of the work. The graph may extend
          indefinitely, enabling expression of a system with increasing
          granularity at every layer.
        </p>

        <p>
          Implementations MUST reject any mutation that would introduce a cycle
          in the Shape composition graph or the Constraint composition graph.
          Mutual dependencies between nodes are modeled through shared parents
          or lateral cross-references via Bindings, not through cycles.
        </p>

        <p>
          IDs are unique per node type. A Shape and a Constraint MAY share the
          same ID value; they occupy separate namespaces.
        </p>

        <p>
          Both Shapes and Constraints support inline and standalone node
          references. When a child entry is an object, it is an inline node
          definition owned by its parent. When it is a scalar, it is a reference
          by ID to a standalone node that may live in a different project or
          organization. This pattern applies uniformly to Shape children and
          Constraint children.
        </p>

        <p>
          The following example illustrates how these node types compose across
          different domains.
        </p>

        <ExampleToggle />

        <SchemaExplorer schema={SHAPE_SCHEMA} caption="Shape schema" />
      </div>
    ),
  },
  // ── §4 Intent ─────────────────────────────────────────────────────────────
  {
    id: "intent",
    title: "Intent",
    content: (
      <div className="space-y-6">
        <p>
          Intent captures the purpose of a Shape — <em>what</em> is to be built
          and <em>why</em>. Every Intent MUST include a{" "}
          <code className="text-foreground">kind</code> (a domain label such as{" "}
          <code className="text-foreground">feature</code>,{" "}
          <code className="text-foreground">experiment</code>, or{" "}
          <code className="text-foreground">chapter</code>), a human-readable{" "}
          <code className="text-foreground">summary</code>, and a{" "}
          <code className="text-foreground">source</code> recording the origin
          from which it was created (e.g.{" "}
          <code className="text-foreground">human</code>,{" "}
          <code className="text-foreground">ai</code>,{" "}
          <code className="text-foreground">system</code>). The source value is
          free-form; Profiles MAY constrain the allowed set of sources for
          records under their governance.
        </p>

        <p>
          Beyond the required fields, Intent is an open map. Each domain extends
          it with its own vocabulary: a software team might add{" "}
          <code className="text-foreground">goals</code> and{" "}
          <code className="text-foreground">non_goals</code>, a research lab{" "}
          <code className="text-foreground">hypotheses</code> and{" "}
          <code className="text-foreground">success_criteria</code>, an
          editorial team <code className="text-foreground">themes</code> and{" "}
          <code className="text-foreground">target_audience</code>.
        </p>

        <p>
          When a Shape is decomposed into sub-Shapes, each sub-Shape's Intent
          MUST remain coherent with its parent. Since the graph may contain
          thousands of Shapes, agents are the primary mechanism for verifying
          coherency across the graph. Agents and humans can both evaluate
          whether a sub-Shape's Intent breaks cohesion with its ancestors.
        </p>

        <SchemaExplorer schema={INTENT_SCHEMA} caption="Intent schema" />
      </div>
    ),
  },
  // ── §5 Constraints ────────────────────────────────────────────────────────
  {
    id: "constraints",
    title: "Constraints",
    content: (
      <div className="space-y-6">
        <p>
          A Constraint is a standalone record expressing an invariant,
          requirement, or policy that MUST be satisfied by any Shape referencing
          it. Constraints are distinct from decisions recorded in provenance —
          they represent strict enforcement rules that MUST be upheld regardless
          of context.
        </p>

        <p>
          Constraints are referenced by ID from any number of Shapes and are
          discovered during graph traversal. When an agent traverses the DAG,
          Constraints referenced by a Shape are in scope for that Shape and all
          its descendants. This is reference-based discovery — child Shapes do
          not automatically carry their parent's Constraint IDs in their own
          records, but agents collecting Constraints from all ancestors will
          discover them. Constraints do not participate in the Shape composition
          graph, but they MAY form their own composition hierarchy through{" "}
          <code className="text-foreground">parents</code> and{" "}
          <code className="text-foreground">children</code> fields, enabling
          decomposition of complex policies into sub-constraints.
        </p>

        <p>
          Constraint kinds classify the nature of the rule:{" "}
          <code className="text-foreground">invariant</code>,{" "}
          <code className="text-foreground">requirement</code>,{" "}
          <code className="text-foreground">boundary</code>,{" "}
          <code className="text-foreground">guideline</code>,{" "}
          <code className="text-foreground">limit</code>,{" "}
          <code className="text-foreground">policy</code>. Both{" "}
          <code className="text-foreground">kind</code> and{" "}
          <code className="text-foreground">enforcement</code> are open strings;
          Profiles and projects MAY define additional values.
        </p>

        <SchemaExplorer
          schema={CONSTRAINT_SCHEMA}
          caption="Constraint schema"
        />

        <CodeBlock
          language="yaml"
          caption="A standalone Constraint describing an organization-wide policy"
        >
          {`Constraint:
  id: 4
  name: Admin Guard
  description: Only admin-role users may perform destructive actions.
  kind: policy
  rule: only users with the admin role may perform destructive actions
  enforcement: machine
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-11-20"
  intent:
    kind: governance
    summary: >
      Restrict destructive operations (delete, revoke, disable) to
      admin-role users to prevent accidental or unauthorized data loss.
    source: human
  realization:
    - bindings:
        - scheme: uri
          value: https://github.com/acme/platform/blob/main/src/middleware/admin_guard.rs#L12-L45
      role: primary`}
        </CodeBlock>
      </div>
    ),
  },
  // ── §6 Amendments ─────────────────────────────────────────────────────────
  {
    id: "amendments",
    title: "Amendments",
    content: (
      <div className="space-y-6">
        <p>
          Amendments are immutable change records applied to canonical Shapes,
          Constraints, or Profiles to evolve the graph over time. An Amendment
          MUST target at least one Shape, Constraint, or Profile. Amendments are
          separated from the target node to preserve the lineage of changes
          while keeping the targeted record lean. Shapes, Constraints, and
          Profiles describe the current state; the amendment log is append-only
          and preserves the full history in the order amendments were applied.
        </p>

        <p>
          Every Shape, Constraint, and Profile moves through a seven-state
          lifecycle: three progressive states and four terminal states.
          Amendments use a five-state subset — proposed, promoted, canonical,
          rejected, and abandoned — excluding superseded and reverted. The base
          specification defines the following default transitions:
        </p>

        <ul className="space-y-2 pl-5 marker:text-primary">
          <li>
            <strong className="text-foreground">Progressive</strong>: proposed →
            promoted → canonical
          </li>
          <li>
            <strong className="text-foreground">Terminal</strong>: any state →
            rejected | superseded | abandoned | reverted
          </li>
        </ul>

        <p>
          Profiles MAY define custom statuses beyond the base set and declare
          transitions involving them, enabling domain-specific workflows.
        </p>

        <table className="w-full text-left">
          <tbody>
            <tr>
              <td
                colSpan={2}
                className="pb-2 text-xs font-semibold tracking-widest text-muted-foreground/70 uppercase"
              >
                Progressive
              </td>
            </tr>
            <tr>
              <td className="py-1.5 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                <span className="inline-flex items-center gap-2">
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    className="shrink-0"
                    aria-hidden="true"
                  >
                    <circle
                      cx="7"
                      cy="7"
                      r="6"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1.5"
                    />
                  </svg>
                  Proposed
                </span>
              </td>
              <td className="py-1.5 align-top">Offered for consideration.</td>
            </tr>
            <tr>
              <td className="py-1.5 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                <span className="inline-flex items-center gap-2">
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    className="shrink-0"
                    aria-hidden="true"
                  >
                    <circle
                      cx="7"
                      cy="7"
                      r="6"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1.5"
                    />
                    <path d="M7 1 A6 6 0 0 0 7 13 Z" fill="currentColor" />
                  </svg>
                  Promoted
                </span>
              </td>
              <td className="py-1.5 align-top">
                Accepted and actively being worked on.
              </td>
            </tr>
            <tr>
              <td className="py-1.5 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                <span className="inline-flex items-center gap-2">
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    className="shrink-0"
                    aria-hidden="true"
                  >
                    <circle
                      cx="7"
                      cy="7"
                      r="6"
                      fill="currentColor"
                      stroke="currentColor"
                      strokeWidth="1.5"
                    />
                  </svg>
                  Canonical
                </span>
              </td>
              <td className="py-1.5 align-top">
                Authoritative — the accepted source of truth.
              </td>
            </tr>
            <tr>
              <td colSpan={2} className="py-4">
                <hr className="border-t border-dashed border-border/80" />
              </td>
            </tr>
            <tr>
              <td
                colSpan={2}
                className="pb-2 text-xs font-semibold tracking-widest text-muted-foreground/70 uppercase"
              >
                Terminal
              </td>
            </tr>
            <tr>
              <td className="py-1.5 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Rejected
              </td>
              <td className="py-1.5 align-top">Declined.</td>
            </tr>
            <tr>
              <td className="py-1.5 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Superseded
                <span className="ml-1.5 text-xs font-normal text-muted-foreground">
                  (not Amendments)
                </span>
              </td>
              <td className="py-1.5 align-top">
                Replaced by one or more successors.
              </td>
            </tr>
            <tr>
              <td className="py-1.5 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Abandoned
              </td>
              <td className="py-1.5 align-top">No longer pursued.</td>
            </tr>
            <tr>
              <td className="py-1.5 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Reverted
                <span className="ml-1.5 text-xs font-normal text-muted-foreground">
                  (not Amendments)
                </span>
              </td>
              <td className="py-1.5 align-top">
                Previously accepted, now withdrawn.
              </td>
            </tr>
          </tbody>
        </table>

        <p>
          While a Shape or Constraint remains Proposed, changes are direct
          edits. Once it reaches Promoted or Canonical, all further changes MUST
          be recorded as Amendments. How a canonical Amendment is applied to its
          targets is defined by the governing Profile (
          <a href="#profiles" className={linkClass}>
            §9
          </a>
          ). The specification recognizes four amendment models:
        </p>

        <table className="w-full text-left">
          <tbody>
            <tr>
              <td className="py-1 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Merge
              </td>
              <td className="py-1">
                Fields from the Amendment are integrated directly into the base
                record, replacing or extending existing values.
              </td>
            </tr>
            <tr>
              <td className="py-1 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Overlay
              </td>
              <td className="py-1">
                The base record is not modified. Effective state is computed at
                read time by layering Amendments in sequence on top of it.
              </td>
            </tr>
            <tr>
              <td className="py-1 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Edition
              </td>
              <td className="py-1">
                Each canonical Amendment produces a new immutable snapshot.
              </td>
            </tr>
            <tr>
              <td className="py-1 pr-4 align-top font-semibold whitespace-nowrap text-foreground">
                Append-only
              </td>
              <td className="py-1">
                The base record is never mutated; the amendment log is the sole
                source of truth.
              </td>
            </tr>
          </tbody>
        </table>

        <p>
          When a Shape or Constraint is superseded, its terminal status carries
          a <code className="text-foreground">successors</code> field and the
          replacement record carries a{" "}
          <code className="text-foreground">predecessors</code> field. A record
          MUST only list predecessors that are in a terminal state.
          Implementations MUST maintain reciprocal links: if Shape B lists Shape
          A in its predecessors, Shape A's terminal status MUST include Shape B
          in its successors.
        </p>

        <SchemaExplorer schema={AMENDMENT_SCHEMA} caption="Amendment schema" />

        <CodeBlock
          language="yaml"
          caption="Merge model: base Shape after amendment integration"
        >
          {`Shape:
  id: 3
  name: Invitations
  profile: 1
  version: 1.1.0
  status:
    canonical:
      metadata:
        date: "2026-01-15"
  intent:
    kind: feature
    summary: Allow administrators to invite users by email.
    source: human
    goals:
      - Reduce setup friction for new teams
      - Keep invitation issuance restricted to trusted actors
      # applied from amendment 5
      - Support Apple sign-in for invitation acceptance
  constraints:
    - 5  # invariant: only admins may create invitations
    # applied from amendment 5
    - 6  # requirement: invitation acceptance must support Apple sign-in
  realization:
    - bindings:
        - scheme: uri
          value: https://github.com/acme/invitations/blob/a1b2c3d/src/service.rs#L40-L118
        # applied from amendment 5
        - scheme: uri
          value: https://github.com/acme/invitations/blob/a1b2c3d/src/apple_oauth.rs#L1-L95
      role: primary
  amendment_log:
    - 5`}
        </CodeBlock>

        <CodeBlock
          language="yaml"
          caption="Amendment record for the Shape above"
        >
          {`Amendment:
  id: 5
  name: Apple Sign-In
  description: Add Apple sign-in for invitation acceptance.
  targets:
    shape_ids: [3]
  status:
    canonical:
      metadata:
        date: "2026-02-14"
  version_impact: minor
  intent:
    kind: enhancement
    summary: >
      Add Apple sign-in as an accepted authentication method
      for invitation acceptance, alongside existing Google support.
    source: human
    rationale: 34% of target users prefer Apple sign-in.
    goals:
      - Support Apple sign-in for invitation acceptance
  initiated_by:
    type: human
    identity: user.admin.jane`}
        </CodeBlock>

        <p>
          Amendments MAY target Shapes and Constraints in any combination. When
          an Amendment's targets span multiple Profiles, each Profile's gates
          are evaluated independently. If the resulting changes conflict,
          resolution is delegated to the implementing agent system. The
          following example targets the Admin Guard policy from{" "}
          <a href="#constraints" className={linkClass}>
            §5
          </a>
          , widening the rule from "only admins" to "admins and org owners."
        </p>

        <CodeBlock
          language="yaml"
          caption="Amendment targeting a standalone Constraint"
        >
          {`Amendment:
  id: 8
  name: Org Owner Access
  description: Extend destructive-action access to org owners.
  targets:
    constraint_ids: [4]
  status:
    canonical:
      metadata:
        date: "2026-02-01"
  version_impact: minor
  intent:
    kind: policy_change
    summary: >
      Allow org owners to perform destructive actions alongside admins.
    source: human
    rationale: 40% of admin escalations were org owners unable to remove stale projects.
  initiated_by:
    type: human
    identity: user.admin.ops`}
        </CodeBlock>
      </div>
    ),
  },
  // ── §7 Bindings ───────────────────────────────────────────────────────────
  {
    id: "bindings",
    title: "Bindings",
    content: (
      <div className="space-y-6">
        <p>
          Bindings connect spec records to external artifacts, test results, and
          provenance sources. A Binding is a typed reference — a{" "}
          <code className="text-foreground">scheme</code> that identifies the
          kind of reference (a URI, a file path, a query, a custom selector) and
          a <code className="text-foreground">value</code> that resolves to the
          target. Bindings appear throughout the spec wherever a record needs to
          point to an external resource.
        </p>

        <SchemaExplorer schema={BINDING_SCHEMA} caption="Binding schema" />
      </div>
    ),
    subsections: [
      {
        id: "realizations",
        title: "Realizations",
        content: (
          <div className="space-y-6">
            <p>
              A Realization connects a Shape to the deliverables that fulfill it
              — source files, endpoints, design documents, published chapters,
              or any other artifact. Each Realization MUST carry one or more
              Bindings and a <code className="text-foreground">role</code> that
              classifies its relationship to the Shape (e.g.{" "}
              <code className="text-foreground">primary</code>,{" "}
              <code className="text-foreground">supporting</code>,{" "}
              <code className="text-foreground">test</code>).
            </p>

            <p>
              Realizations MAY span across projects and repositories. For
              example, an authentication feature might reference server-side
              implementation files, frontend components, and API documentation
              across separate codebases.
            </p>

            <p>
              Because Realizations enumerate the concrete artifacts backing a
              Shape, the scope of any subsequent Amendment is bounded and
              unambiguous. Agents operating on the Shape have explicit
              visibility into which artifacts fall within its scope.
            </p>

            <SchemaExplorer
              schema={REALIZATION_SCHEMA}
              caption="Realization schema"
            />
          </div>
        ),
      },
      {
        id: "evidence",
        title: "Evidence",
        content: (
          <div className="space-y-6">
            <p>
              Evidence records demonstrate that a Shape's constraints are
              satisfied and its goals are met. Each Evidence entry MUST have a{" "}
              <code className="text-foreground">type</code> (such as{" "}
              <code className="text-foreground">test</code>,{" "}
              <code className="text-foreground">review</code>, or{" "}
              <code className="text-foreground">metric</code>), a{" "}
              <code className="text-foreground">trusted</code> indicator
              specifying whether the result is verified, and Bindings that point
              to the underlying results.
            </p>

            <p>
              Realizations and Evidence are distinct record types. The artifact
              that implements a requirement and the artifact that verifies it
              serve different roles and require independent review. Evidence
              MUST demonstrate that the Realization actually satisfies the
              Intent — not merely that the code executes without error.
            </p>

            <SchemaExplorer
              schema={EVIDENCE_SCHEMA}
              caption="Evidence schema"
            />
          </div>
        ),
      },
      {
        id: "provenance",
        title: "Provenance",
        content: (
          <div className="space-y-6">
            <p>
              Provenance tracks the origin and decision history of any node in
              the graph — Shape, Constraint, Amendment, or Profile. The
              specification is agnostic to the provenance systems used.
              Sessions, transcripts, discussions, and any other decision
              artifacts MAY be incorporated as Provenance entries.
            </p>

            <p>
              Provenance entries provide agents with the full decision history
              behind a node — the discussions, reasoning, and trade-offs that
              informed it. Bindings MAY reference any external source: a Slack
              thread, a Linear task, a Claude Code session, or a dedicated
              provenance platform such as{" "}
              <a
                href="https://github.com/cursor/agent-trace"
                target="_blank"
                rel="noopener noreferrer"
                className={linkClass}
              >
                Agent Trace
              </a>{" "}
              or{" "}
              <a
                href="https://entire.io"
                target="_blank"
                rel="noopener noreferrer"
                className={linkClass}
              >
                Entire
              </a>
              .
            </p>

            <SchemaExplorer
              schema={PROVENANCE_SCHEMA}
              caption="Provenance schema"
            />
          </div>
        ),
      },
    ],
  },
  // ── §8 Boundaries ─────────────────────────────────────────────────────────
  {
    id: "boundaries",
    title: "Boundaries",
    content: (
      <div className="space-y-6">
        <p>
          Since every Shape, Constraint, and Amendment carries Realizations and
          Evidence within the broader DAG structure, clear boundaries can be
          derived from the graph itself. This allows agent planning and review
          to stay properly scoped to the nodes being worked on.
        </p>

        <p>
          Any changes that affect artifacts outside the scope of the target
          node's Realization and Evidence records constitute a boundary
          violation. Implementations SHOULD surface such violations and MAY
          enforce them through Constraint rules.
        </p>
      </div>
    ),
  },
  // ── §9 Profiles ───────────────────────────────────────────────────────────
  {
    id: "profiles",
    title: "Profiles",
    content: (
      <div className="space-y-6">
        <p>
          Profiles are governance configurations that control how Shapes,
          Constraints, and Amendments behave within a domain. A single Profile
          MAY govern both Shapes and Constraints through separate field
          declaration sections. A Shape or standalone Constraint's{" "}
          <code className="text-foreground">profile</code> field references the
          Profile that governs it. Amendments inherit the Profile of their
          targets.
        </p>

        <p>
          Each Profile defines lifecycle gates — preconditions that MUST be
          satisfied before a record can transition from one state to the next.
          For example, a Profile MAY require that all Constraints have passing
          Evidence before a Shape can move from Promoted to Canonical. Profiles
          MAY also define custom statuses beyond the base set, enabling
          domain-specific workflows. A Profile specifies which amendment model (
          <a href="#amendments" className={linkClass}>
            §6
          </a>
          ) applies to its records and MAY extend the base schema with
          domain-specific custom fields via FieldDef declarations. The{" "}
          <code className="text-foreground">fields</code> block is split into{" "}
          <code className="text-foreground">shape</code> and{" "}
          <code className="text-foreground">constraint</code> sections — either
          or both MAY be defined. Each section covers intent, status,
          constraints, realization, evidence, provenance, and metadata.
        </p>

        <p>
          Since the specification is cross-domain and agnostic over the actual
          work, a Profile MAY encode any domain-specific lifecycle — an SDLC, an
          ADLC (Agentic Development Life Cycle), an editorial workflow, or a
          research methodology.
        </p>

        <SchemaExplorer schema={PROFILE_SCHEMA} caption="Profile schema" />

        <p>
          FieldDef allows a Profile to declare additional fields that nodes
          under its governance MAY or MUST carry. Each definition specifies a
          name, an optional type annotation, and a human-readable description. A
          FieldDef whose <code className="text-foreground">required</code> flag
          is <code className="text-foreground">true</code> must be present on
          governed records; when the flag is omitted it defaults to{" "}
          <code className="text-foreground">false</code>, making the field
          optional. This makes the specification extensible without modifying
          the core schema.
        </p>

        <SchemaExplorer schema={FIELDDEF_SCHEMA} caption="FieldDef schema" />
      </div>
    ),
  },
]
