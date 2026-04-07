import type { ReactNode } from "react"
import { useEffect } from "react"
import { RiArrowRightLine, RiGithubFill } from "@remixicon/react"
import { Link, createFileRoute } from "@tanstack/react-router"
import { motion, useReducedMotion, type Variants } from "motion/react"
import { Button } from "@workspace/ui/components/button"
import { Container } from "@workspace/ui/components/container"
import { SectionDivider } from "@workspace/ui/components/section-divider"
import { SectionLabel } from "@workspace/ui/components/section-label"
import { CodeBlock } from "@/components/spec/code-block"
import { Footer } from "@/components/footer"
import { Nav } from "@/components/nav"

/* ── Motion system ──
 *
 * Two shared patterns:
 *   heroParent/heroChild — staggered mount choreography for the hero
 *   Reveal              — scroll-triggered fade+slide for each section
 *
 * Both honor prefers-reduced-motion: when reduce is requested, children
 * render instantly with no transform or opacity animation.
 */
const EASE_OUT_QUART = [0.25, 1, 0.5, 1] as const

const heroParent: Variants = {
  hidden: {},
  visible: {
    transition: { staggerChildren: 0.08, delayChildren: 0.1 },
  },
}

const heroChild: Variants = {
  hidden: { opacity: 0, y: 16 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.6, ease: EASE_OUT_QUART },
  },
}

function Reveal({
  children,
  className,
  delay = 0,
}: {
  children: ReactNode
  className?: string
  delay?: number
}) {
  const reduce = useReducedMotion()
  if (reduce) return <div className={className}>{children}</div>
  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.2, margin: "0px 0px -10% 0px" }}
      transition={{ duration: 0.7, ease: EASE_OUT_QUART, delay }}
    >
      {children}
    </motion.div>
  )
}

export const Route = createFileRoute("/")({ component: HomePage })

const TREE_EXAMPLE = `shape:1 Platform [canonical]
├── shape:3 Auth Service [promoted]
│   ├── constraint:5 Security Policy
│   ├── shape:7 OAuth Login [promoted]
│   │   └── constraint:8 Rate Limiting
│   └── shape:8 MFA [proposed]
└── shape:4 API Gateway [promoted]
    └── constraint:6 Uptime SLA`

const SHAPE_TREE = `shape:1 Platform [canonical]
├── shape:3 Auth Service [promoted]
│   ├── shape:7 OAuth Login [promoted]
│   └── shape:8 MFA [proposed]
└── shape:4 API Gateway [promoted]`

const CONSTRAINT_TREE = `constraint:1 Security Policy
├── constraint:5 Uptime SLA          → shape:4
├── constraint:7 Rate Limiting       → shape:7
└── constraint:8 MFA Required        → shape:3`

const SHAPE_EXAMPLE = `shape:
  id: 12
  name: User Authentication
  kind: feature
  status: promoted

  intent:
    summary: "OAuth2 login flow with MFA support"
    source: human
    acceptance:
      - Users can sign in with Google or GitHub
      - MFA is required for admin accounts

  parent: 3        # → Auth Service
  constraints:
    - 5            # → Security Policy
    - 8            # → Rate Limiting

  realization:
    - bindings:
        - scheme: path
          value: src/auth/oauth.rs
      role: primary`

const BORDER = "border-border/40"

/* ── Shapes lifecycle token renderer ──
 *
 * Walks an ASCII tree line by line and wraps semantic tokens in colored
 * spans that match the Shapes vocabulary:
 *   [canonical]  → primary, bold   (most-committed state)
 *   [promoted]   → primary/75      (progressing state)
 *   [proposed]   → water           (provisional state)
 *   [rejected|superseded|abandoned|reverted] → destructive/70
 *   → shape:N | → constraint:N     → water/80  (cross-DAG reference)
 *
 * Color is never the sole indicator — the bracketed text and arrow
 * glyphs carry the same meaning for non-sighted or colorblind readers.
 */
const LIFECYCLE_RE =
  /\[(canonical|promoted|proposed|rejected|superseded|abandoned|reverted)\]/g
const REFERENCE_RE = /→\s+(shape|constraint):\d+/g

function lifecycleClass(state: string): string {
  switch (state) {
    case "canonical":
      return "text-primary font-medium"
    case "promoted":
      return "text-primary/75"
    case "proposed":
      return "text-water"
    default:
      return "text-destructive/70"
  }
}

const LIFECYCLE_TOOLTIP: Record<string, string> = {
  canonical:
    "Canonical — the most committed state. Edits require an amendment.",
  promoted:
    "Promoted — progressing through the lifecycle. Edits require an amendment.",
  proposed: "Proposed — provisional. Editable directly until promoted.",
  rejected: "Rejected — terminal. Reviewed and not adopted.",
  superseded: "Superseded — terminal. Replaced by a successor shape.",
  abandoned: "Abandoned — terminal. Work stopped before completion.",
  reverted: "Reverted — terminal. Promoted then rolled back.",
}

function referenceTooltip(text: string): string {
  // text looks like "→ shape:7" or "→ constraint:8"
  const kind = text.includes("shape") ? "shape" : "constraint"
  return `Cross-DAG reference — points from this constraint into the ${kind} graph.`
}

function renderShapesTreeLine(line: string, key: number) {
  // Split on the union of both patterns so we can wrap each match.
  const merged = new RegExp(
    `${LIFECYCLE_RE.source}|${REFERENCE_RE.source}`,
    "g"
  )
  const parts: Array<React.ReactNode> = []
  let lastIndex = 0
  let match: RegExpExecArray | null
  let idx = 0

  while ((match = merged.exec(line)) !== null) {
    if (match.index > lastIndex) {
      parts.push(line.slice(lastIndex, match.index))
    }
    const text = match[0]
    if (text.startsWith("[")) {
      const state = text.slice(1, -1)
      parts.push(
        <span
          key={`s-${idx}`}
          className={`${lifecycleClass(state)} cursor-help`}
          title={LIFECYCLE_TOOLTIP[state] ?? state}
        >
          {text}
        </span>
      )
    } else {
      parts.push(
        <span
          key={`r-${idx}`}
          className="cursor-help text-water/80"
          title={referenceTooltip(text)}
        >
          {text}
        </span>
      )
    }
    lastIndex = merged.lastIndex
    idx++
  }
  if (lastIndex < line.length) {
    parts.push(line.slice(lastIndex))
  }
  return (
    <span key={key}>
      {parts}
      {"\n"}
    </span>
  )
}

function ShapesTree({ children }: { children: string }) {
  const lines = children.split("\n")
  return (
    <pre className="yaml-scroll overflow-x-auto font-mono text-code leading-relaxed text-foreground">
      {lines.map((line, i) => renderShapesTreeLine(line, i))}
    </pre>
  )
}

function GridLines() {
  return (
    <div
      className="pointer-events-none fixed inset-0 z-0 hidden sm:block"
      aria-hidden="true"
    >
      <Container className="h-full">
        <div className={`h-full border-x ${BORDER}`} />
      </Container>
    </div>
  )
}

/* ─── Two Graphs: structural demonstration ───
 *
 * Replaces the removed 2×2 primitives grid with an actual dual-DAG view.
 * The Profile label frames the whole thing (top). Shapes and Constraints
 * each render their own tree in the same monospace vocabulary as the hero
 * example. Amendment is annotated at the bottom of the frame. The section
 * escapes the container to sit in a full-bleed band, with the content
 * re-narrowed inside.
 */
function TwoGraphs() {
  return (
    <section
      aria-labelledby="two-graphs-heading"
      className={`relative border-y ${BORDER} bg-secondary/30`}
    >
      <Container>
        <div className={`border-x ${BORDER}`}>
          {/* Profile frame label */}
          <div
            className={`flex items-center justify-between gap-4 border-b ${BORDER} px-6 py-3 sm:px-10 lg:px-14`}
          >
            <span className="font-ui text-tiny font-medium tracking-label text-muted-foreground uppercase">
              Profile · Software
            </span>
            <span className="font-ui text-tiny font-medium tracking-label text-muted-foreground/70 uppercase">
              v0.1.0 — Working Draft
            </span>
          </div>

          {/* Title block */}
          <div
            className={`border-b ${BORDER} px-6 py-12 sm:px-10 sm:py-16 lg:px-14 lg:py-20`}
          >
            <SectionLabel number={2}>Two Graphs</SectionLabel>
            <h2
              id="two-graphs-heading"
              className="mt-5 font-serif text-3xl leading-heading font-semibold tracking-heading sm:text-4xl md:text-5xl lg:text-6xl"
            >
              Shapes and Constraints
              <br />
              <span className="text-muted-foreground">
                each form their own DAG.
              </span>
            </h2>
            <p className="mt-8 max-w-2xl text-lg leading-relaxed text-muted-foreground">
              Two independent graphs framed by one profile. Shapes compose work
              — features within services within systems. Constraints compose
              rules — invariants within policies. Every shape inherits the
              constraints of its ancestors, so the rules that govern any node
              are the rules of every node above it.
            </p>
          </div>

          {/* Dual DAG demonstration */}
          <div className={`grid grid-cols-1 md:grid-cols-2`}>
            <div
              className={`border-b ${BORDER} min-w-0 px-6 py-10 sm:px-10 md:border-r md:border-b-0 lg:px-14 lg:py-14`}
            >
              <div className="flex items-baseline justify-between gap-4">
                <span className="font-ui text-tiny font-medium tracking-label text-muted-foreground uppercase">
                  Shape DAG
                </span>
                <span className="font-ui text-tiny font-medium tracking-label-tight text-muted-foreground/60 uppercase">
                  composition
                </span>
              </div>
              <div className="mt-6">
                <ShapesTree>{SHAPE_TREE}</ShapesTree>
              </div>
            </div>
            <div className={`min-w-0 px-6 py-10 sm:px-10 lg:px-14 lg:py-14`}>
              <div className="flex items-baseline justify-between gap-4">
                <span className="font-ui text-tiny font-medium tracking-label text-muted-foreground uppercase">
                  Constraint DAG
                </span>
                <span className="font-ui text-tiny font-medium tracking-label-tight text-muted-foreground/60 uppercase">
                  policy
                </span>
              </div>
              <div className="mt-6">
                <ShapesTree>{CONSTRAINT_TREE}</ShapesTree>
              </div>
            </div>
          </div>

          {/* Amendment annotation */}
          <div
            className={`flex flex-col items-start justify-between gap-2 border-t ${BORDER} px-6 py-4 sm:flex-row sm:items-center sm:px-10 lg:px-14`}
          >
            <span className="font-ui text-tiny font-medium tracking-label text-muted-foreground uppercase">
              Amendment · Change history
            </span>
            <span className="text-xs text-muted-foreground/70 italic">
              Once promoted, shapes evolve only through amendments.
            </span>
          </div>
        </div>
      </Container>
    </section>
  )
}

/* ── Console greeting ──
 *
 * One quiet brand-appropriate log on first mount per session, in the
 * project's own monospace tree-art voice. Devs reading the source see
 * it; everyone else never knows. Guarded by a session flag so hot
 * reloads do not spam.
 */
const GREETING = `
shape:1 Shapes
├── intent:  a structured graph for project context
├── kind:    open specification
└── source:  github.com/shapes-fyi/shapes

Reading the source? Amendments and PRs welcome.
`

function useConsoleGreeting() {
  useEffect(() => {
    if (typeof window === "undefined") return
    if (sessionStorage.getItem("shapes:greeted") === "1") return
    sessionStorage.setItem("shapes:greeted", "1")
    // eslint-disable-next-line no-console
    console.log(GREETING)
  }, [])
}

function HomePage() {
  useConsoleGreeting()
  return (
    <main className="relative min-h-screen bg-background text-foreground">
      <GridLines />
      <Nav variant="transparent" />

      {/* ─── Hero ─── */}
      <section className="flex min-h-svh items-center py-20">
        <Container>
          <motion.div
            className={`grid grid-cols-1 border-y ${BORDER} md:grid-cols-12`}
            variants={heroParent}
            initial="hidden"
            animate="visible"
          >
            <div
              className={`min-w-0 p-6 sm:p-10 md:col-span-5 md:border-r ${BORDER} lg:p-14`}
            >
              {/*
               * SectionLabel + h1 render statically (no motion variants)
               * so the LCP element — the "Shapes" wordmark — hits the
               * screen on first paint instead of waiting for the 700ms
               * motion entrance. Supporting content (paragraph, buttons,
               * code) still staggers in via heroChild children below.
               */}
              <SectionLabel number={1}>Open Specification</SectionLabel>
              <h1 className="mt-6 font-serif text-7xl leading-display font-semibold tracking-display sm:text-8xl lg:text-9xl">
                Shapes
              </h1>
              <motion.p
                variants={heroChild}
                className="mt-8 max-w-md text-lg leading-relaxed text-muted-foreground"
              >
                An open specification that captures what to build, why it
                matters, and what must hold true — one structured graph that AI
                agents and humans can read together.
              </motion.p>
              <motion.div
                variants={heroChild}
                className="mt-10 flex flex-wrap gap-3"
              >
                <Link to="/specification">
                  <Button size="lg" className="gap-1.5 px-5 text-sm">
                    Read the Specification
                    <RiArrowRightLine className="size-4" />
                  </Button>
                </Link>
                <a
                  href="https://github.com/shapes-fyi/shapes"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Button
                    variant="outline"
                    size="lg"
                    className="gap-1.5 px-5 text-sm"
                  >
                    <RiGithubFill className="size-4" />
                    GitHub
                  </Button>
                </a>
              </motion.div>
            </div>

            <motion.div
              variants={heroChild}
              className={`flex min-w-0 items-center border-t ${BORDER} p-6 sm:p-10 md:col-span-7 md:border-t-0 lg:p-14`}
            >
              <CodeBlock className="w-full">{TREE_EXAMPLE}</CodeBlock>
            </motion.div>
          </motion.div>
        </Container>
      </section>

      <SectionDivider />

      {/* ─── Two Graphs demonstration (replaces deleted primitives grid) ─── */}
      <Reveal>
        <TwoGraphs />
      </Reveal>

      <SectionDivider />

      {/* ─── Code Example ─── */}
      <Reveal>
        <section>
          <Container>
            <div className={`border-y ${BORDER}`}>
              <div className={`border-b ${BORDER} p-6 sm:p-10 lg:p-14`}>
                <SectionLabel number={3}>In Practice</SectionLabel>
                <h2 className="mt-5 max-w-3xl font-serif text-4xl leading-heading font-semibold tracking-heading sm:text-5xl lg:text-6xl">
                  Structured intent
                </h2>
                <p className="mt-6 max-w-2xl text-lg leading-relaxed text-muted-foreground">
                  Every shape is a YAML file in your repository. The graph lives
                  alongside your code — versionable, diffable, reviewable.
                </p>
              </div>
              <div className="p-6 sm:p-10 lg:p-14">
                <CodeBlock language="yaml" wrap>
                  {SHAPE_EXAMPLE}
                </CodeBlock>
              </div>
            </div>
          </Container>
        </section>
      </Reveal>

      <SectionDivider />

      {/* ─── Get Started ─── */}
      <Reveal>
        <section>
          <Container>
            <div className={`border-y ${BORDER}`}>
              <div className={`border-b ${BORDER} p-6 sm:p-10 lg:p-14`}>
                <SectionLabel number={4}>Install</SectionLabel>
                <h2 className="mt-5 max-w-3xl font-serif text-4xl leading-heading font-semibold tracking-heading sm:text-5xl lg:text-6xl">
                  One command.
                </h2>
                <p className="mt-6 max-w-2xl text-lg leading-relaxed text-muted-foreground">
                  The CLI is a single Rust binary with zero runtime
                  dependencies. Install it once, run <code>shapes init</code> in
                  your project.
                </p>
              </div>
              <div className="p-6 sm:p-10 lg:p-14">
                <CodeBlock className="w-full" wrap>
                  {`cargo install --git https://github.com/shapes-fyi/shapes`}
                </CodeBlock>
              </div>
            </div>
          </Container>
        </section>
      </Reveal>

      <SectionDivider />

      <Footer />
    </main>
  )
}
