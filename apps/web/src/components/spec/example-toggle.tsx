import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { RiCheckLine, RiFileCopyLine } from "@remixicon/react"
import { renderMermaidSVG } from "beautiful-mermaid"
import { cn } from "@workspace/ui/lib/utils"
import { HighlightedCode } from "./code-block"

type Status = "canonical" | "promoted" | "proposed"

type GraphNode = {
  /** Mermaid node ID (e.g. "PLATFORM") */
  mermaidId: string
  /** Short label shown in the node pill selector */
  label: string
  /** Node type badge */
  type: "Shape" | "Constraint" | "Amendment" | "Profile"
  /** Lifecycle status — drives color coding in diagrams */
  status: Status
  /** YAML definition for this node */
  yaml: string
}

type DomainExample = {
  id: string
  label: string
  diagram: string
  nodes: [GraphNode, ...Array<GraphNode>]
  caption: string
}

// ── Software ─────────────────────────────────

const SOFTWARE_NODES: [GraphNode, ...Array<GraphNode>] = [
  {
    mermaidId: "PLATFORM",
    label: "Platform",
    type: "Shape",
    status: "canonical",
    yaml: `Shape:
  id: 1
  name: Platform
  description: Top-level system encompassing authentication and team management.
  profile: 1
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-12-01"
  intent:
    kind: system
    summary: >
      Provide a unified platform for user identity, access control,
      and team collaboration.
    source: human
    requirements:
      - Unified identity management across all services
      - Role-based access control for team operations
    acceptance_criteria:
      - All services authenticate through a single identity provider
      - Admin, member, and viewer roles enforced across endpoints
  children:
    - shape: 2
      role: service
    - shape: 3
      role: feature`,
  },
  {
    mermaidId: "AUTH",
    label: "Auth",
    type: "Shape",
    status: "promoted",
    yaml: `Shape:
  id: 2
  name: Auth
  description: User authentication and session management.
  profile: 1
  version: 0.2.0
  parents:
    - id: 1
      role: service
  status:
    promoted:
      metadata:
        date: "2026-01-15"
  intent:
    kind: service
    summary: >
      Ensure secure identity verification and session lifecycle
      across all platform services.
    source: human
    requirements:
      - Secure credential verification with MFA support
      - Session lifecycle with configurable expiry
    acceptance_criteria:
      - MFA enrollment available for all accounts
      - Sessions expire per policy and revoke on password change
  constraints:
    - 4
  realization:
    - bindings:
        - scheme: uri
          value: https://github.com/acme/platform/blob/main/src/auth/
      role: primary
  amendment_log:
    - 5`,
  },
  {
    mermaidId: "INV",
    label: "Invitations",
    type: "Shape",
    status: "proposed",
    yaml: `Shape:
  id: 3
  name: Invitations
  description: Email-based team invitation workflow.
  profile: 1
  version: 0.1.0
  parents:
    - id: 1
      role: feature
  # → [§4 Lifecycle](#lifecycle)
  status:
    proposed:
      metadata:
        date: "2026-02-20"
  # → [§3.2 Intent](#intent)
  intent:
    kind: feature
    summary: Allow administrators to invite users by email.
    source: human
    requirements:
      - Admin-only invitation creation with email delivery
      - Token-based acceptance with expiry
    acceptance_criteria:
      - Non-admin users cannot issue invitations
      - Expired tokens return a clear error and allow re-invitation
    goals:
      - Reduce setup friction for new teams
      - Keep invitation issuance restricted to trusted actors
    non_goals:
      - Guest access
  # → [§6 Constraints](#constraints)
  constraints:
    - 5  # invariant: only admins may create invitations
    - 4  # shared: Admin Guard policy`,
  },
  {
    mermaidId: "CON",
    label: "Admin Guard",
    type: "Constraint",
    status: "canonical",
    yaml: `Constraint:
  id: 4
  name: Admin Guard
  description: Only admin-role users may perform destructive actions.
  kind: policy
  rule: only users with the admin role may perform destructive actions
  enforcement: machine
  profile: 1
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
      role: primary`,
  },
  {
    mermaidId: "AMEND",
    label: "MFA Support",
    type: "Amendment",
    status: "promoted",
    yaml: `Amendment:
  id: 5
  name: MFA Support
  description: Add multi-factor authentication to the Auth service.
  targets:
    shape_ids: [2]
  status:
    promoted:
      metadata:
        date: "2026-02-14"
  version_impact: minor
  intent:
    kind: enhancement
    summary: >
      Add multi-factor authentication as a required step for
      elevated-privilege sessions.
    source: human
    rationale: >
      Security audit identified password-only auth as the top
      risk vector for admin accounts.
  constraints:
    - 6  # requirement: elevated-privilege sessions require MFA
  realization:
    - bindings:
        - scheme: uri
          value: https://github.com/acme/platform/blob/main/src/auth/mfa.rs#L1-L120
      role: primary
  evidence:
    - id: ci_456
      type: test_report
      bindings:
        - scheme: uri
          value: https://ci.example/456
      trusted: true
  initiated_by:
    type: human
    identity: user.admin.jane`,
  },
  {
    mermaidId: "PROF",
    label: "SaaS",
    type: "Profile",
    status: "canonical",
    yaml: `Profile:
  id: 1
  name: SaaS
  description: Lifecycle gates for a SaaS product team.
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-09-01"
  intent:
    kind: governance
    summary: >
      Define lifecycle gates and field requirements for a SaaS
      product team building platform services and features.
    source: human
  lifecycle:
    gates:
      - from: proposed
        to: promoted
        preconditions:
          - tests pass
          - realization binding exists
      - from: promoted
        to: canonical
        preconditions:
          - security review evidence exists
  fields:
    shape:
      intent:
        required:
          - name: requirements
            description: What the Shape must satisfy.
          - name: acceptance_criteria
            description: Testable conditions proving requirements are met.
        kinds:
          - name: system
            description: A top-level system or platform.
          - name: service
            description: A deployable service within a system.
          - name: feature
            description: A user-facing capability.
        sources:
          - name: human
            description: Created by a human author.
          - name: ai
            description: Generated by an AI agent.
      status:
        required:
          - name: date
            description: UTC ISO 8601 timestamp of the state transition.
            type: iso8601
      evidence:
        kinds:
          - name: test_report
            description: Automated test results.
          - name: security_review
            description: Security audit findings.
    constraint:
      intent:
        kinds:
          - name: governance
            description: Organizational governance rule.
          - name: security
            description: Security-related enforcement.
        sources:
          - name: human
            description: Created by a human author.
      status:
        required:
          - name: date
            description: UTC ISO 8601 timestamp of the state transition.
            type: iso8601
  versioning:
    scheme: semver
  amendment_rules:
    application: merge`,
  },
]

// ── Novel ────────────────────────────────────

const NOVEL_NODES: [GraphNode, ...Array<GraphNode>] = [
  {
    mermaidId: "NOVEL",
    label: "The Last Vigil",
    type: "Shape",
    status: "canonical",
    yaml: `Shape:
  id: 1
  name: The Last Vigil
  description: A disgraced knight's quest to reclaim a stolen relic from a cursed fortress, told in three acts.
  profile: 1
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-10-15"
  intent:
    kind: novel
    summary: >
      Explore the cost of duty and redemption through intimate character work
      within a taut three-act structure.
    source: human
    themes:
      - The cost of duty and the possibility of redemption
      - Trust as currency in a world governed by oaths
    target_audience: Adult fantasy readers (25-45) who prefer character-driven narratives
    goals:
      - Explore the cost of duty and redemption through intimate character work
      - Maintain a taut three-act structure under 80,000 words
  children:
    - shape: 2
      role: chapter
    - shape: 3
      role: chapter`,
  },
  {
    mermaidId: "CH7",
    label: "The Ashen Gate",
    type: "Shape",
    status: "promoted",
    yaml: `Shape:
  id: 2
  name: The Ashen Gate
  description: The fortress breach — Act II climax.
  profile: 1
  version: 0.4.0
  parents:
    - id: 1
      role: chapter
  # → [§4 Lifecycle](#lifecycle)
  status:
    promoted:
      metadata:
        date: "2026-01-10"
  # → [§3.2 Intent](#intent)
  intent:
    kind: chapter
    summary: >
      Show the knight breaching the cursed fortress and reveal the
      physical and emotional cost of confronting the guardian.
    source: human
    themes:
      - The physical toll of confronting what you fear
      - Loyalty tested by impossible odds
    target_audience: Adult fantasy readers (25-45) who prefer character-driven narratives
    goals:
      - Establish the physical and emotional stakes of the siege
      - Introduce the guardian character who reappears in Act III
    non_goals:
      - Resolve the central conflict
  # → [§6 Constraints](#constraints)
  constraints:
    - 5  # guideline: chapter length 4,000-6,000 words
    - 4  # shared: Narrative POV
  # → [§7.1 Realization](#realization)
  realization:
    - bindings:
        - scheme: uri
          value: https://docs.example/manuscripts/last-vigil/chapter-7-draft-3.md
      role: primary
    - bindings:
        - scheme: uri
          value: https://docs.example/manuscripts/last-vigil/outline.md#act-ii
      role: supporting
  # → [§7.2 Evidence](#evidence)
  evidence:
    - id: beta_read_12
      type: review
      bindings:
        - scheme: uri
          value: https://docs.example/feedback/beta-round-2/chapter-7
      trusted: true
  # → [§7 Bindings](#bindings)
  provenance:
    - type: agent_trace
      bindings:
        - scheme: trace
          value: 8a3b17c0-f1e4-4d2a-b9c6-1234567890ab
  # → [§5 Amendments](#amendments)
  amendment_log:
    - 5`,
  },
  {
    mermaidId: "CH8",
    label: "The Hollow Throne",
    type: "Shape",
    status: "proposed",
    yaml: `Shape:
  id: 3
  name: The Hollow Throne
  description: The inner sanctum — Act III opening.
  profile: 1
  version: 0.1.0
  parents:
    - id: 1
      role: chapter
  status:
    proposed:
      metadata:
        date: "2026-02-28"
  intent:
    kind: chapter
    summary: >
      Establish the relic chamber through sensory contrast with Act II
      and surface the first cracks in the knight's resolve.
    source: human
    themes:
      - Doubt as the true adversary once the external threat is past
      - The silence after violence
    target_audience: Adult fantasy readers (25-45) who prefer character-driven narratives
    goals:
      - Establish the relic chamber through sensory contrast with Act II
      - Surface the first cracks in the knight's resolve
  constraints:
    - 6  # guideline: chapter length 4,000-6,000 words
    - 4  # shared: Narrative POV`,
  },
  {
    mermaidId: "CON",
    label: "Narrative POV",
    type: "Constraint",
    status: "canonical",
    yaml: `Constraint:
  id: 4
  name: Narrative POV
  description: Third-person limited narration from the protagonist's perspective.
  kind: invariant
  rule: narration stays in third-person limited from the protagonist's perspective
  enforcement: human
  profile: 1
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-10-15"
  intent:
    kind: narrative
    summary: >
      Maintain a consistent narrative voice across all chapters
      to preserve reader immersion and emotional proximity.
    source: human`,
  },
  {
    mermaidId: "AMEND",
    label: "Guardian Expansion",
    type: "Amendment",
    status: "promoted",
    yaml: `Amendment:
  id: 5
  name: Guardian Expansion
  description: Strengthen the guardian's setup in Chapter 7.
  targets:
    shape_ids: [2]
  status:
    promoted:
      metadata:
        date: "2026-03-01"
  version_impact: minor
  intent:
    kind: revision
    summary: >
      Foreshadow the Act III revelation by expanding the guardian's
      introduction, per editor feedback.
    source: human
    rationale: >
      Beta readers found the guardian's reappearance in Act III
      felt unearned without stronger setup in Chapter 7.
  realization:
    - bindings:
        - scheme: uri
          value: https://docs.example/manuscripts/last-vigil/chapter-7-draft-4.md
      role: primary
  evidence:
    - id: editor_note_7
      type: review
      bindings:
        - scheme: uri
          value: https://docs.example/feedback/editor/chapter-7-revision
      trusted: true
  initiated_by:
    type: human
    identity: editor.maria`,
  },
  {
    mermaidId: "PROF",
    label: "Editorial",
    type: "Profile",
    status: "canonical",
    yaml: `Profile:
  id: 1
  name: Editorial
  description: Lifecycle gates for a fiction editorial workflow.
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-09-01"
  intent:
    kind: governance
    summary: >
      Define lifecycle gates and field requirements for a fiction
      editorial workflow from draft through publication.
    source: human
  lifecycle:
    gates:
      - from: proposed
        to: promoted
        preconditions:
          - beta read evidence exists
      - from: promoted
        to: canonical
        preconditions:
          - editor sign-off evidence exists
  fields:
    shape:
      intent:
        required:
          - name: themes
            description: Central themes the chapter must explore.
          - name: target_audience
            description: Intended reader demographic.
        kinds:
          - name: novel
            description: A full-length fiction work.
          - name: chapter
            description: A single chapter within a novel.
        sources:
          - name: human
            description: Written by a human author.
          - name: ai
            description: Generated or co-written by an AI assistant.
      status:
        required:
          - name: date
            description: UTC ISO 8601 timestamp of the state transition.
            type: iso8601
      evidence:
        kinds:
          - name: beta_read
            description: Feedback from beta readers.
          - name: editor_review
            description: Editorial sign-off and notes.
    constraint:
      intent:
        kinds:
          - name: narrative
            description: Narrative voice or structural constraint.
          - name: editorial
            description: Editorial process constraint.
      status:
        required:
          - name: date
            description: UTC ISO 8601 timestamp of the state transition.
            type: iso8601
  versioning:
    scheme: opaque
  amendment_rules:
    application: merge`,
  },
]

// ── Research ─────────────────────────────────

const RESEARCH_NODES: [GraphNode, ...Array<GraphNode>] = [
  {
    mermaidId: "STUDY",
    label: "Catalyst Study",
    type: "Shape",
    status: "canonical",
    yaml: `Shape:
  id: 1
  name: Catalyst Study
  description: Research program for catalyst degradation analysis.
  profile: 1
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-09-01"
  intent:
    kind: research_program
    summary: >
      Characterize catalyst degradation under thermal stress
      to validate and refine the batch-process lifetime model.
    source: human
    hypotheses:
      - Catalyst degradation follows first-order kinetics above 900 K
      - Batch-process lifetime model overestimates usable cycles by 15-20%
    success_criteria:
      - Degradation curves with R² > 0.95 across both experimental conditions
      - Model calibration error reduced below 10%
    goals:
      - Establish degradation curves for the current catalyst formulation
      - Provide data to calibrate the predictive model
  children:
    - shape: 2
      role: experiment
    - shape: 3
      role: experiment`,
  },
  {
    mermaidId: "EXP41",
    label: "Cyclic Load",
    type: "Shape",
    status: "canonical",
    yaml: `Shape:
  id: 2
  name: Cyclic Load
  description: Catalyst degradation experiment under repeated thermal cycles.
  profile: 1
  version: 1.1.0
  parents:
    - id: 1
      role: experiment
  # → [§4 Lifecycle](#lifecycle)
  status:
    canonical:
      metadata:
        date: "2025-11-15"
  # → [§3.2 Intent](#intent)
  intent:
    kind: experiment
    summary: >
      Quantify degradation rate across 500 thermal cycles and compare
      observed lifetime against the predictive model.
    source: human
    hypotheses:
      - Cyclic thermal stress accelerates degradation compared to sustained load
      - Degradation onset occurs before cycle 200
    success_criteria:
      - Degradation curve with R² > 0.95 for 500 cycles
      - Observed vs. predicted lifetime within 15% margin
    goals:
      - Quantify degradation rate across 500 thermal cycles
      - Compare observed lifetime against the predictive model
    non_goals:
      - Optimize catalyst composition
  # → [§6 Constraints](#constraints)
  constraints:
    - 5  # boundary: peak temperature must not exceed 1,120 K
    - 4  # shared: Sample Origin
  # → [§7.1 Realization](#realization)
  realization:
    - bindings:
        - scheme: uri
          value: https://lab.example/notebooks/experiment-41b/protocol.ipynb
      role: primary
    - bindings:
        - scheme: uri
          value: https://lab.example/instruments/furnace-config-41b.yaml
      role: supporting
  # → [§7.2 Evidence](#evidence)
  evidence:
    - id: run_data_41b
      type: dataset
      bindings:
        - scheme: uri
          value: https://lab.example/data/experiment-41b/results.parquet
      trusted: true
    - id: peer_review_41b
      type: review
      bindings:
        - scheme: uri
          value: https://lab.example/reviews/experiment-41b-internal
      trusted: true
  # → [§7 Bindings](#bindings)
  provenance:
    - type: agent_trace
      bindings:
        - scheme: trace
          value: c47d29a0-5e8f-4f3c-a012-abcdef012345
  # → [§5 Amendments](#amendments)
  amendment_log:
    - 5`,
  },
  {
    mermaidId: "EXP42",
    label: "Sustained Load",
    type: "Shape",
    status: "proposed",
    yaml: `Shape:
  id: 3
  name: Sustained Load
  description: Catalyst degradation experiment under constant high temperature.
  profile: 1
  version: 0.1.0
  parents:
    - id: 1
      role: experiment
  status:
    proposed:
      metadata:
        date: "2026-02-25"
  intent:
    kind: experiment
    summary: >
      Complement the cyclic-load data by measuring degradation
      at sustained 1,050 K over 200 hours.
    source: human
    hypotheses:
      - Sustained load produces linear degradation after an initial plateau
      - Plateau duration correlates with initial catalyst surface area
    success_criteria:
      - Degradation profile characterized over full 200-hour window
      - Plateau-to-degradation transition point identified within ±5 hours
    goals:
      - Quantify degradation rate at sustained 1,050 K over 200 hours
      - Compare sustained vs. cyclic degradation profiles
  constraints:
    - 4  # shared: Sample Origin`,
  },
  {
    mermaidId: "CON",
    label: "Sample Origin",
    type: "Constraint",
    status: "canonical",
    yaml: `Constraint:
  id: 4
  name: Sample Origin
  description: Lot-controlled constraint for catalyst sample sourcing.
  kind: invariant
  rule: all samples drawn from lot 2024-Q3-batch-7
  enforcement: machine
  profile: 1
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-09-01"
  intent:
    kind: reproducibility
    summary: >
      Ensure experimental reproducibility by restricting all
      catalyst samples to a single verified production lot.
    source: human`,
  },
  {
    mermaidId: "AMEND",
    label: "Cooling Rate",
    type: "Amendment",
    status: "canonical",
    yaml: `Amendment:
  id: 5
  name: Cooling Rate
  description: Protocol amendment adding a cooling-rate boundary.
  targets:
    shape_ids: [2]
  status:
    canonical:
      metadata:
        date: "2026-02-20"
  version_impact: minor
  intent:
    kind: protocol_change
    summary: >
      Prevent anomalous crystallization observed in Run 312
      by capping furnace cooling rate at 8 K/min.
    source: human
    rationale: >
      Post-run analysis revealed that rapid cooling introduced a
      confounding variable not controlled in the original protocol.
  constraints:
    - 6  # boundary: cooling rate must not exceed 8 K/min
  realization:
    - bindings:
        - scheme: uri
          value: https://lab.example/notebooks/experiment-41b/protocol-v2.ipynb
      role: primary
  evidence:
    - id: run_312_analysis
      type: dataset
      bindings:
        - scheme: uri
          value: https://lab.example/data/experiment-41b/run-312-anomaly.parquet
      trusted: true
  initiated_by:
    type: human
    identity: researcher.dr.chen`,
  },
  {
    mermaidId: "PROF",
    label: "Lab Protocol",
    type: "Profile",
    status: "canonical",
    yaml: `Profile:
  id: 1
  name: Lab Protocol
  description: Lifecycle gates for experimental research programs.
  version: 1.0.0
  status:
    canonical:
      metadata:
        date: "2025-09-01"
  intent:
    kind: governance
    summary: >
      Define lifecycle gates and field requirements for experimental
      research programs following peer-review methodology.
    source: human
  lifecycle:
    gates:
      - from: proposed
        to: promoted
        preconditions:
          - peer review evidence exists
      - from: promoted
        to: canonical
        preconditions:
          - reproducibility report exists
  fields:
    shape:
      intent:
        required:
          - name: hypotheses
            description: Testable predictions the experiment aims to validate.
          - name: success_criteria
            description: Measurable outcomes that determine success.
        kinds:
          - name: research_program
            description: A top-level research initiative.
          - name: experiment
            description: A single experiment within a program.
        sources:
          - name: human
            description: Designed by a human researcher.
          - name: system
            description: Auto-generated by a lab automation system.
      status:
        required:
          - name: date
            description: UTC ISO 8601 timestamp of the state transition.
            type: iso8601
      evidence:
        kinds:
          - name: peer_review
            description: Independent expert evaluation of methods and findings.
          - name: reproducibility_report
            description: Verification that results can be independently reproduced.
    constraint:
      intent:
        kinds:
          - name: reproducibility
            description: Constraint ensuring experimental reproducibility.
          - name: safety
            description: Safety boundary for experimental parameters.
      status:
        required:
          - name: date
            description: UTC ISO 8601 timestamp of the state transition.
            type: iso8601
  versioning:
    scheme: calver
  amendment_rules:
    application: edition`,
  },
]

// ── Examples ─────────────────────────────────

const DOMAIN_EXAMPLES: Array<DomainExample> = [
  {
    id: "software",
    label: "Software",
    caption:
      "A software project governed by an SaaS Profile, with parent/child Shapes, a shared Constraint, and an Amendment",
    diagram: `flowchart TB
  PROF("SaaS")
  PLATFORM(["Platform"])
  AUTH(["Auth"])
  INV(["Invitations"])
  CON{{"Admin Guard"}}
  AMEND[["MFA Support"]]

  PROF -.->|"governs"| PLATFORM
  PROF -.->|"governs"| CON
  PLATFORM -.->|"child"| AUTH
  PLATFORM -.->|"child"| INV
  AUTH -.->|"constraint"| CON
  INV -.->|"constraint"| CON
  AMEND -.->|"amends"| AUTH`,
    nodes: SOFTWARE_NODES,
  },
  {
    id: "novel",
    label: "Novel",
    caption:
      "A novel governed by an Editorial Profile, with parent/child Shapes for chapters, a shared narrative Constraint, and an Amendment",
    diagram: `flowchart TB
  PROF("Editorial")
  NOVEL(["The Last Vigil"])
  CH7(["The Ashen Gate"])
  CH8(["The Hollow Throne"])
  CON{{"Narrative POV"}}
  AMEND[["Guardian Expansion"]]

  PROF -.->|"governs"| NOVEL
  PROF -.->|"governs"| CON
  NOVEL -.->|"child"| CH7
  NOVEL -.->|"child"| CH8
  CH7 -.->|"constraint"| CON
  CH8 -.->|"constraint"| CON
  AMEND -.->|"amends"| CH7`,
    nodes: NOVEL_NODES,
  },
  {
    id: "research",
    label: "Research",
    caption:
      "A research program governed by a Lab Protocol Profile, with parent/child Shapes for experiments, a shared Constraint, and an Amendment",
    diagram: `flowchart TB
  PROF("Lab Protocol")
  STUDY(["Catalyst Study"])
  EXP41(["Cyclic Load"])
  EXP42(["Sustained Load"])
  CON{{"Sample Origin"}}
  AMEND[["Cooling Rate"]]

  PROF -.->|"governs"| STUDY
  PROF -.->|"governs"| CON
  STUDY -.->|"child"| EXP41
  STUDY -.->|"child"| EXP42
  EXP41 -.->|"constraint"| CON
  EXP42 -.->|"constraint"| CON
  AMEND -.->|"amends"| EXP41`,
    nodes: RESEARCH_NODES,
  },
]

const MERMAID_COLORS = {
  bg: "var(--background)",
  fg: "var(--foreground)",
  line: "color-mix(in oklch, var(--foreground) 34%, var(--background))",
  accent: "var(--primary)",
  muted: "color-mix(in oklch, var(--foreground) 58%, var(--background))",
  surface: "color-mix(in oklch, var(--foreground) 2.5%, var(--background))",
  border: "color-mix(in oklch, var(--foreground) 16%, var(--background))",
  font: "Crimson Pro Variable, Crimson Pro, serif",
  transparent: true,
  padding: 10,
  nodeSpacing: 16,
  layerSpacing: 24,
  componentSpacing: 16,
} as const

// Status colors for color-coding diagram nodes by lifecycle status
const STATUS_COLORS: Record<
  Status,
  { dot: string; stroke: string; label: string }
> = {
  canonical: {
    dot: "bg-emerald-500/70",
    stroke: "oklch(0.65 0.15 160)",
    label: "Canonical",
  },
  promoted: {
    dot: "bg-blue-500/70",
    stroke: "oklch(0.6 0.15 250)",
    label: "Promoted",
  },
  proposed: {
    dot: "bg-amber-500/70",
    stroke: "oklch(0.7 0.15 80)",
    label: "Proposed",
  },
}

export function ExampleToggle() {
  const [domainId, setDomainId] = useState("software")
  const domain = DOMAIN_EXAMPLES.find((e) => e.id === domainId)!
  const [selectedNodeId, setSelectedNodeId] = useState(
    domain.nodes[0].mermaidId
  )
  // When domain changes, reset to first node
  const handleDomainChange = useCallback((id: string) => {
    setDomainId(id)
    const example = DOMAIN_EXAMPLES.find((e) => e.id === id)!
    setSelectedNodeId(example.nodes[0].mermaidId)
  }, [])

  const selectedNode =
    domain.nodes.find((n) => n.mermaidId === selectedNodeId) ?? domain.nodes[0]

  const [copied, setCopied] = useState(false)
  const [yamlVisible, setYamlVisible] = useState(true)
  const [displayedNode, setDisplayedNode] = useState(selectedNode)
  const [revealKey, setRevealKey] = useState(0)
  const yamlRef = useRef<HTMLDivElement>(null)
  const preRef = useRef<HTMLPreElement>(null)

  // Canvas zoom/pan state
  const canvasRef = useRef<HTMLDivElement>(null)
  const DEFAULT_ZOOM = 0.5
  const MIN_ZOOM = 0.3
  const MAX_ZOOM = 2.5
  const DRAG_THRESHOLD = 3
  const [transform, setTransform] = useState({
    zoom: DEFAULT_ZOOM,
    panX: 0,
    panY: 0,
  })
  // Mutable ref for gesture handlers to avoid stale closures
  const tRef = useRef(transform)
  tRef.current = transform

  const gestureRef = useRef({
    active: false,
    pointerId: -1,
    startX: 0,
    startY: 0,
    lastX: 0,
    lastY: 0,
    didDrag: false,
    // touch
    touchActive: false,
    lastTouchDist: 0,
    lastTouchCenterX: 0,
    lastTouchCenterY: 0,
    dragEndTime: 0,
  })

  // Tab sliding indicator
  const tablistRef = useRef<HTMLDivElement>(null)
  const tabRefs = useRef<Map<string, HTMLButtonElement>>(new Map())
  const [indicator, setIndicator] = useState({ left: 0, width: 0 })

  // Update sliding indicator position when active tab changes
  useEffect(() => {
    const tabEl = tabRefs.current.get(domainId)
    const container = tablistRef.current
    if (!tabEl || !container) return
    const containerRect = container.getBoundingClientRect()
    const tabRect = tabEl.getBoundingClientRect()
    setIndicator({
      left: tabRect.left - containerRect.left,
      width: tabRect.width,
    })
  }, [domainId])

  // Show scrollbar while actively scrolling
  useEffect(() => {
    const el = preRef.current
    if (!el) return
    let timeout: ReturnType<typeof setTimeout>
    function onScroll() {
      el!.classList.add("is-scrolling")
      clearTimeout(timeout)
      timeout = setTimeout(() => el!.classList.remove("is-scrolling"), 800)
    }
    el.addEventListener("scroll", onScroll, { passive: true })
    return () => {
      el.removeEventListener("scroll", onScroll)
      clearTimeout(timeout)
    }
  }, [])

  // When selected node changes, fade out → swap content → reveal in
  useEffect(() => {
    if (selectedNode.mermaidId === displayedNode.mermaidId) return
    setYamlVisible(false)
    const timeout = setTimeout(() => {
      setDisplayedNode(selectedNode)
      setRevealKey((k) => k + 1)
      setYamlVisible(true)
    }, 150)
    return () => clearTimeout(timeout)
  }, [selectedNode, displayedNode.mermaidId])

  function handleCopy() {
    navigator.clipboard.writeText(selectedNode.yaml).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  // Center the diagram within the canvas
  const centerDiagram = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas) return { zoom: DEFAULT_ZOOM, panX: 0, panY: 0 }
    const svg = canvas.querySelector("svg")
    if (!svg) return { zoom: DEFAULT_ZOOM, panX: 0, panY: 0 }
    const canvasW = canvas.clientWidth
    const canvasH = canvas.clientHeight
    const svgW = svg.clientWidth || svg.getBoundingClientRect().width
    const svgH = svg.clientHeight || svg.getBoundingClientRect().height
    const zoom = DEFAULT_ZOOM
    const panX = (canvasW - svgW * zoom) / 2
    const panY = (canvasH - svgH * zoom) / 2
    return { zoom, panX, panY }
  }, [DEFAULT_ZOOM])

  // Center on initial render and when domain changes
  useEffect(() => {
    // Small delay to ensure SVG is rendered
    const raf = requestAnimationFrame(() => {
      setTransform(centerDiagram())
    })
    return () => cancelAnimationFrame(raf)
  }, [domainId, centerDiagram])

  // Canvas gesture handling
  useEffect(() => {
    const el = canvasRef.current
    if (!el) return
    const g = gestureRef.current

    function clampZoom(z: number) {
      return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z))
    }

    // Zoom toward a point (in client/viewport coords)
    function zoomAt(clientX: number, clientY: number, newZoom: number) {
      const t = tRef.current
      const rect = el!.getBoundingClientRect()
      // Point in diagram space under cursor
      const dx = clientX - rect.left - t.panX
      const dy = clientY - rect.top - t.panY
      const diagramX = dx / t.zoom
      const diagramY = dy / t.zoom
      // Recompute pan so diagram point stays under cursor
      const clamped = clampZoom(newZoom)
      setTransform({
        zoom: clamped,
        panX: clientX - rect.left - diagramX * clamped,
        panY: clientY - rect.top - diagramY * clamped,
      })
    }

    // ── Wheel: Cmd/Ctrl + scroll → cursor-centered zoom ──
    function onWheel(e: WheelEvent) {
      if (!e.metaKey && !e.ctrlKey) return
      e.preventDefault()
      const factor = 1 - e.deltaY * 0.003
      zoomAt(e.clientX, e.clientY, tRef.current.zoom * factor)
    }

    // ── Pointer: left-click drag to pan, with click/drag disambiguation ──
    function onPointerDown(e: PointerEvent) {
      // Left (0) or middle (1) button
      if (e.button !== 0 && e.button !== 1) return
      g.active = true
      g.pointerId = e.pointerId
      g.startX = e.clientX
      g.startY = e.clientY
      g.lastX = e.clientX
      g.lastY = e.clientY
      g.didDrag = false
      // Don't capture yet — wait until we know it's a drag
    }

    function onPointerMove(e: PointerEvent) {
      if (!g.active || e.pointerId !== g.pointerId) return
      const dx = e.clientX - g.lastX
      const dy = e.clientY - g.lastY

      // Check drag threshold before committing to a pan
      if (!g.didDrag) {
        const totalDx = e.clientX - g.startX
        const totalDy = e.clientY - g.startY
        if (Math.abs(totalDx) < DRAG_THRESHOLD && Math.abs(totalDy) < DRAG_THRESHOLD) return
        g.didDrag = true
        el!.setPointerCapture(e.pointerId)
        el!.style.cursor = "grabbing"
      }

      g.lastX = e.clientX
      g.lastY = e.clientY
      setTransform((t) => ({ ...t, panX: t.panX + dx, panY: t.panY + dy }))
    }

    function onPointerUp(e: PointerEvent) {
      if (!g.active || e.pointerId !== g.pointerId) return
      if (g.didDrag) {
        el!.releasePointerCapture(e.pointerId)
      }
      el!.style.cursor = ""
      g.active = false

      // Record drag end time so handleDiagramClick can ignore post-drag clicks
      if (g.didDrag) {
        g.dragEndTime = Date.now()
      }
    }

    // ── Touch: pinch-zoom + two-finger pan ──
    function getTouchDist(touches: TouchList) {
      const dx = touches[0].clientX - touches[1].clientX
      const dy = touches[0].clientY - touches[1].clientY
      return Math.hypot(dx, dy)
    }

    function onTouchStart(e: TouchEvent) {
      if (e.touches.length !== 2) return
      e.preventDefault()
      g.touchActive = true
      g.lastTouchDist = getTouchDist(e.touches)
      g.lastTouchCenterX = (e.touches[0].clientX + e.touches[1].clientX) / 2
      g.lastTouchCenterY = (e.touches[0].clientY + e.touches[1].clientY) / 2
    }

    function onTouchMove(e: TouchEvent) {
      if (!g.touchActive || e.touches.length !== 2) return
      e.preventDefault()
      const dist = getTouchDist(e.touches)
      const cx = (e.touches[0].clientX + e.touches[1].clientX) / 2
      const cy = (e.touches[0].clientY + e.touches[1].clientY) / 2

      // Cursor-centered zoom at pinch midpoint
      const scaleFactor = dist / g.lastTouchDist
      const newZoom = clampZoom(tRef.current.zoom * scaleFactor)
      const rect = el!.getBoundingClientRect()
      const t = tRef.current
      // Diagram point under old pinch center
      const diagX = (g.lastTouchCenterX - rect.left - t.panX) / t.zoom
      const diagY = (g.lastTouchCenterY - rect.top - t.panY) / t.zoom
      // New pan: keep diagram point under new pinch center
      setTransform({
        zoom: newZoom,
        panX: cx - rect.left - diagX * newZoom,
        panY: cy - rect.top - diagY * newZoom,
      })

      g.lastTouchDist = dist
      g.lastTouchCenterX = cx
      g.lastTouchCenterY = cy
    }

    function onTouchEnd() {
      g.touchActive = false
    }

    el.addEventListener("wheel", onWheel, { passive: false })
    el.addEventListener("pointerdown", onPointerDown)
    el.addEventListener("pointermove", onPointerMove)
    el.addEventListener("pointerup", onPointerUp)
    el.addEventListener("touchstart", onTouchStart, { passive: false })
    el.addEventListener("touchmove", onTouchMove, { passive: false })
    el.addEventListener("touchend", onTouchEnd)

    return () => {
      el.removeEventListener("wheel", onWheel)
      el.removeEventListener("pointerdown", onPointerDown)
      el.removeEventListener("pointermove", onPointerMove)
      el.removeEventListener("pointerup", onPointerUp)
      el.removeEventListener("touchstart", onTouchStart)
      el.removeEventListener("touchmove", onTouchMove)
      el.removeEventListener("touchend", onTouchEnd)
    }
  }, [])

  // Pre-render all domain diagrams so the container keeps a stable size
  const diagramSvgs = useMemo(() => {
    const map: Record<string, string> = {}
    for (const example of DOMAIN_EXAMPLES) {
      try {
        map[example.id] = renderMermaidSVG(example.diagram, MERMAID_COLORS)
      } catch {
        // diagram render failed
      }
    }
    return map
  }, [])

  // Click handler for diagram nodes — React onClick bubbles from SVG children
  const handleDiagramClick = useCallback(
    (e: React.MouseEvent) => {
      // Ignore clicks that immediately follow a drag gesture
      if (Date.now() - gestureRef.current.dragEndTime < 200) return
      const node = (e.target as Element).closest<SVGGElement>("[data-id]")
      if (!node) return
      const mermaidId = node.getAttribute("data-id")
      if (mermaidId && domain.nodes.some((n) => n.mermaidId === mermaidId)) {
        setSelectedNodeId(mermaidId)
      }
    },
    [domain.nodes]
  )

  // Declarative CSS for cursor, opacity highlight, hover, and node-type colors
  const diagramStyles = useMemo(() => {
    const nodeIds = domain.nodes.map((n) => n.mermaidId)
    const unselectedIds = nodeIds.filter((id) => id !== selectedNodeId)

    const allSel = nodeIds.map((id) => `[data-id="${id}"]`).join(",")
    const allChildSel = nodeIds.map((id) => `[data-id="${id}"] *`).join(",")
    const selSel = `[data-id="${selectedNodeId}"]`

    const rules = [
      `${allSel}, ${allChildSel} { cursor: pointer; }`,
      `${allSel} { transition: opacity 0.15s; }`,
    ]
    if (unselectedIds.length > 0) {
      const unselSel = unselectedIds.map((id) => `[data-id="${id}"]`).join(",")
      const unselHover = unselectedIds
        .map((id) => `[data-id="${id}"]:hover`)
        .join(",")
      rules.push(`${unselSel} { opacity: 0.45; }`)
      rules.push(`${unselHover} { opacity: 0.75; }`)
    }
    rules.push(`${selSel} { opacity: 1; }`)

    // Color-code node strokes by lifecycle status
    for (const node of domain.nodes) {
      const color = STATUS_COLORS[node.status].stroke
      rules.push(
        `[data-id="${node.mermaidId}"] rect, [data-id="${node.mermaidId}"] polygon, [data-id="${node.mermaidId}"] circle, [data-id="${node.mermaidId}"] ellipse { stroke: ${color}; }`
      )
    }

    return rules.join("\n")
  }, [domain.nodes, selectedNodeId])

  return (
    <figure className="group/code full-bleed space-y-4">
      <div className="overflow-hidden rounded-xl border border-border/80 bg-secondary/60">
        {/* Domain toggle */}
        <div className="flex items-center justify-center px-5 pt-4 pb-3 sm:px-6">
          <div
            ref={tablistRef}
            role="tablist"
            aria-label="Example domain"
            className="relative inline-flex rounded-full border border-border/80 bg-background/60 p-1"
          >
            <div
              className="absolute top-1 h-[calc(100%-0.5rem)] rounded-full bg-background shadow-[var(--shadow-elevated)]"
              style={{
                left: indicator.left,
                width: indicator.width,
                transition:
                  indicator.width > 0
                    ? "left 300ms cubic-bezier(0.4, 0, 0.2, 1), width 300ms cubic-bezier(0.4, 0, 0.2, 1)"
                    : "none",
              }}
            />
            {DOMAIN_EXAMPLES.map((example) => (
              <button
                key={example.id}
                ref={(el) => {
                  if (el) tabRefs.current.set(example.id, el)
                  else tabRefs.current.delete(example.id)
                }}
                role="tab"
                aria-selected={example.id === domainId}
                onClick={() => handleDomainChange(example.id)}
                className={cn(
                  "relative z-10 rounded-full px-3 py-1 text-xs transition-colors",
                  example.id === domainId
                    ? "font-medium text-foreground"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                {example.label}
              </button>
            ))}
          </div>
        </div>

        {/* Diagram + YAML side by side — fixed height prevents layout shift */}
        <div className="flex flex-col lg:h-[42rem] lg:flex-row">
          {/* Left: diagram + legend */}
          <div className="relative mx-4 mb-3 flex flex-col overflow-hidden rounded-lg border border-border/60 bg-card/50 px-3 py-3 sm:mx-5 sm:px-4 lg:mx-0 lg:mb-0 lg:w-1/2 lg:shrink-0 lg:rounded-none lg:border-0 lg:bg-transparent lg:py-4 lg:pr-4 lg:pl-6">
            <div
              ref={canvasRef}
              className="min-h-0 flex-1 cursor-grab select-none overflow-hidden active:cursor-grabbing"
              style={{ touchAction: "none" }}
              onDoubleClick={() =>
                setTransform(centerDiagram())
              }
            >
              <div
                className="grid [&_svg]:h-auto [&_svg]:w-full"
                style={{
                  transform: `translate(${transform.panX}px, ${transform.panY}px) scale(${transform.zoom})`,
                  transformOrigin: "0 0",
                  willChange: "transform",
                }}
              >
                <style dangerouslySetInnerHTML={{ __html: diagramStyles }} />
                {DOMAIN_EXAMPLES.map((example) => {
                  const svg = diagramSvgs[example.id]
                  if (!svg) return null
                  const isActive = example.id === domainId
                  return (
                    <div
                      key={example.id}
                      aria-label={
                        isActive
                          ? "Shape graph — click a node to view its definition"
                          : undefined
                      }
                      aria-hidden={!isActive}
                      className="col-start-1 row-start-1"
                      style={{
                        visibility: isActive ? "visible" : "hidden",
                        pointerEvents: isActive ? "auto" : "none",
                      }}
                      onClick={isActive ? handleDiagramClick : undefined}
                      dangerouslySetInnerHTML={{ __html: svg }}
                    />
                  )
                })}
              </div>
            </div>
            {/* Legend: shapes + status colors */}
            <div className="mt-2 shrink-0 rounded-lg border border-dashed border-border/60 px-2.5 py-1.5">
              <div className="mb-1 flex items-center justify-end gap-3">
                <div className="flex items-center gap-1.5">
                  <svg
                    width="14"
                    height="10"
                    viewBox="0 0 14 10"
                    className="shrink-0"
                  >
                    <rect
                      x="0.5"
                      y="0.5"
                      width="13"
                      height="9"
                      rx="4.5"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-muted-foreground/40"
                    />
                  </svg>
                  <span className="text-xs text-muted-foreground/70">
                    Shape
                  </span>
                </div>
                <div className="flex items-center gap-1.5">
                  <svg
                    width="14"
                    height="10"
                    viewBox="0 0 14 10"
                    className="shrink-0"
                  >
                    <polygon
                      points="3,5 7,0.5 11,5 7,9.5"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-muted-foreground/40"
                    />
                  </svg>
                  <span className="text-xs text-muted-foreground/70">
                    Constraint
                  </span>
                </div>
                <div className="flex items-center gap-1.5">
                  <svg
                    width="14"
                    height="10"
                    viewBox="0 0 14 10"
                    className="shrink-0"
                  >
                    <rect
                      x="0.5"
                      y="0.5"
                      width="13"
                      height="9"
                      rx="1"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-muted-foreground/40"
                    />
                    <line
                      x1="3"
                      y1="0.5"
                      x2="3"
                      y2="9.5"
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-muted-foreground/40"
                    />
                    <line
                      x1="11"
                      y1="0.5"
                      x2="11"
                      y2="9.5"
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-muted-foreground/40"
                    />
                  </svg>
                  <span className="text-xs text-muted-foreground/70">
                    Amendment
                  </span>
                </div>
                <div className="flex items-center gap-1.5">
                  <svg
                    width="14"
                    height="10"
                    viewBox="0 0 14 10"
                    className="shrink-0"
                  >
                    <rect
                      x="0.5"
                      y="0.5"
                      width="13"
                      height="9"
                      rx="2.5"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1"
                      className="text-muted-foreground/40"
                    />
                  </svg>
                  <span className="text-xs text-muted-foreground/70">
                    Profile
                  </span>
                </div>
              </div>
              <div className="flex items-center justify-end gap-3">
                {(["canonical", "promoted", "proposed"] as const).map(
                  (status) => (
                    <div key={status} className="flex items-center gap-1">
                      <span
                        className={cn(
                          "size-1.5 rounded-full",
                          STATUS_COLORS[status].dot
                        )}
                      />
                      <span className="text-xs text-muted-foreground/70">
                        {STATUS_COLORS[status].label}
                      </span>
                    </div>
                  )
                )}
              </div>
            </div>
          </div>

          {/* Right: YAML */}
          <div className="flex min-w-0 flex-1 flex-col overflow-hidden lg:mr-4 lg:mb-4">
            {/* YAML code block */}
            <div className="relative min-h-0 flex-1">
              <button
                type="button"
                onClick={handleCopy}
                aria-label={copied ? "Copied" : "Copy code"}
                className="absolute top-2 right-4 z-10 rounded-md p-2 text-muted-foreground opacity-0 transition group-hover/code:opacity-100 hover:text-foreground focus-visible:opacity-100"
              >
                {copied ? (
                  <RiCheckLine className="size-3.5" />
                ) : (
                  <RiFileCopyLine className="size-3.5" />
                )}
              </button>

              <div
                ref={yamlRef}
                key={revealKey}
                className="h-full"
                onAnimationEnd={(e) => {
                  if (e.animationName === "yaml-reveal") {
                    ;(e.currentTarget as HTMLElement).style.willChange = "auto"
                  }
                }}
                style={
                  yamlVisible
                    ? {
                        animation:
                          "yaml-reveal 350ms cubic-bezier(0.16, 1, 0.3, 1) both",
                        willChange: "transform, filter, opacity",
                      }
                    : {
                        opacity: 0,
                        transition: "opacity 150ms ease-in",
                      }
                }
              >
                <pre
                  ref={preRef}
                  className="yaml-scroll h-full max-h-[25rem] overflow-x-auto overflow-y-auto px-5 pt-2 pb-5 text-[0.8125rem] leading-relaxed sm:px-6 lg:max-h-none lg:px-0 lg:pr-4"
                >
                  <code className="font-mono">
                    <HighlightedCode>{displayedNode.yaml}</HighlightedCode>
                  </code>
                </pre>
              </div>
            </div>
          </div>
        </div>
      </div>

      <figcaption className="text-center text-sm text-muted-foreground">
        {domain.caption}
      </figcaption>
    </figure>
  )
}
