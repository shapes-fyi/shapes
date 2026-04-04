import { useState } from "react"
import {
  RiArrowRightLine,
  RiCheckLine,
  RiFileCopyLine,
  RiGithubFill,
} from "@remixicon/react"
import { Link, createFileRoute } from "@tanstack/react-router"
import { Button } from "@workspace/ui/components/button"
import { CodeBlock } from "@/components/spec/code-block"
import { Nav } from "@/components/nav"

export const Route = createFileRoute("/")({ component: HomePage })

const TREE_EXAMPLE = `shape:1 Platform [canonical]
├── shape:3 Auth Service [promoted]
│   ├── constraint:5 Security Policy
│   ├── shape:7 OAuth Login [promoted]
│   │   └── constraint:8 Rate Limiting
│   └── shape:8 MFA [proposed]
└── shape:4 API Gateway [promoted]
    └── constraint:6 Uptime SLA`

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

/* ── Grid infrastructure ── */

const BORDER = "border-border/40"

function GridLines() {
  return (
    <div
      className="pointer-events-none fixed inset-0 z-0 hidden sm:block"
      aria-hidden="true"
    >
      <div className="mx-auto h-full max-w-5xl px-6 sm:px-10 lg:px-16">
        <div className={`h-full border-x ${BORDER}`} />
      </div>
    </div>
  )
}

function SectionDivider() {
  return (
    <div className="relative mx-auto my-6 max-w-5xl px-6 sm:my-8 sm:px-10 lg:px-16">
      <div className={`relative border-t ${BORDER}`}>
        <div className="absolute -top-[3px] -left-[3px] hidden size-1.5 rounded-full bg-border/40 sm:block" />
        <div className="absolute -top-[3px] left-1/2 -ml-[3px] hidden size-1.5 rounded-full bg-border/40 sm:block" />
        <div className="absolute -top-[3px] -right-[3px] hidden size-1.5 rounded-full bg-border/40 sm:block" />
      </div>
    </div>
  )
}

/* ── Helpers ── */

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)

  function handleCopy() {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <button
      type="button"
      onClick={handleCopy}
      className="rounded-md p-1 text-muted-foreground transition hover:text-foreground"
      aria-label={copied ? "Copied" : "Copy to clipboard"}
    >
      {copied ? (
        <RiCheckLine className="size-3.5" />
      ) : (
        <RiFileCopyLine className="size-3.5" />
      )}
    </button>
  )
}

/* ── Page ── */

const primitives = [
  {
    name: "Shape",
    desc: "The primary work node. Captures what to build and why — intent, acceptance criteria, status, and bindings to code.",
  },
  {
    name: "Constraint",
    desc: "A rule that must hold true. Falsifiable invariants inherited downward through the graph, each with evidence requirements.",
  },
  {
    name: "Amendment",
    desc: "An immutable change record. Once promoted, shapes evolve only through amendments — the full history is always preserved.",
  },
  {
    name: "Profile",
    desc: "Governance configuration. Defines lifecycle gates, custom fields, allowed kinds, and amendment models for each domain.",
  },
]

function HomePage() {
  return (
    <main className="relative min-h-screen bg-background text-foreground">
      <GridLines />
      <Nav variant="transparent" />

      {/* ─── Hero ─── */}
      <section className="flex min-h-svh items-center py-20">
        <div className="mx-auto w-full max-w-5xl px-6 sm:px-10 lg:px-16">
          <div className={`grid grid-cols-1 border-y ${BORDER} md:grid-cols-2`}>
            <div className={`p-8 md:border-r ${BORDER} md:p-12 lg:p-14`}>
              <p className="text-xs font-semibold uppercase tracking-[0.25em] text-muted-foreground">
                Open Specification
              </p>
              <h1 className="mt-5 font-serif text-5xl leading-[0.9] font-semibold tracking-[-0.04em] sm:text-6xl lg:text-7xl">
                Shapes
              </h1>
              <p className="mt-6 text-lg leading-relaxed text-muted-foreground">
                A structured graph for capturing the intent, structure, and
                constraints of any project — the shared contract between agents
                and humans.
              </p>
              <div className="mt-8 flex flex-wrap gap-3">
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
              </div>
            </div>

            <div
              className={`flex items-center border-t ${BORDER} p-8 md:border-t-0 md:p-12 lg:p-14`}
            >
              <pre className="yaml-scroll w-full overflow-x-auto rounded-xl border border-border/80 bg-secondary/60 px-5 py-5 font-mono text-[0.8125rem] leading-relaxed text-muted-foreground">
                {TREE_EXAMPLE}
              </pre>
            </div>
          </div>
        </div>
      </section>

      <SectionDivider />

      {/* ─── The Spec ─── */}
      <section>
        <div className="mx-auto max-w-5xl px-6 sm:px-10 lg:px-16">
          <div className={`border-y ${BORDER}`}>
            {/* Header cell */}
            <div className={`border-b ${BORDER} p-8 md:p-12 lg:p-14`}>
              <p className="text-xs font-semibold uppercase tracking-[0.25em] text-muted-foreground">
                The Spec
              </p>
              <h2 className="mt-4 max-w-xl font-serif text-3xl font-semibold tracking-[-0.02em] sm:text-4xl">
                Four primitives, two graphs
              </h2>
              <p className="mt-5 max-w-2xl text-lg leading-relaxed text-muted-foreground">
                Shapes and Constraints each form their own directed acyclic
                graph. Amendments record how the graph evolves. Profiles govern
                how each domain uses it.
              </p>
            </div>

            {/* 2×2 primitives grid */}
            <div className="grid grid-cols-1 sm:grid-cols-2">
              {primitives.map((item, i) => (
                <div
                  key={item.name}
                  className={`p-8 md:p-10 ${i % 2 === 0 ? `sm:border-r ${BORDER}` : ""} ${i < 2 ? `border-b ${BORDER}` : ""}`}
                >
                  <h3 className="font-serif text-xl font-semibold">
                    {item.name}
                  </h3>
                  <p className="mt-2 text-lg leading-relaxed text-muted-foreground">
                    {item.desc}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      <SectionDivider />

      {/* ─── Code Example ─── */}
      <section>
        <div className="mx-auto max-w-5xl px-6 sm:px-10 lg:px-16">
          <div className={`grid grid-cols-1 border-y ${BORDER} md:grid-cols-6`}>
            <div
              className={`p-8 md:col-span-2 md:border-r ${BORDER} md:p-12 lg:p-14`}
            >
              <p className="text-xs font-semibold uppercase tracking-[0.25em] text-muted-foreground">
                In Practice
              </p>
              <h2 className="mt-4 font-serif text-3xl font-semibold tracking-[-0.02em] sm:text-4xl">
                Structured intent
              </h2>
              <p className="mt-5 text-lg leading-relaxed text-muted-foreground">
                Every shape is a YAML file in your repository. The graph lives
                alongside your code — versionable, diffable, reviewable.
              </p>
            </div>
            <div
              className={`border-t ${BORDER} p-8 md:col-span-4 md:border-t-0 md:p-12 lg:p-14`}
            >
              <CodeBlock language="yaml">{SHAPE_EXAMPLE}</CodeBlock>
            </div>
          </div>
        </div>
      </section>

      <SectionDivider />

      {/* ─── Get Started ─── */}
      <section>
        <div className="mx-auto max-w-5xl px-6 sm:px-10 lg:px-16">
          <div
            className={`border-y ${BORDER} p-8 text-center md:p-12 lg:p-14`}
          >
            <h2 className="font-serif text-3xl font-semibold tracking-[-0.02em] sm:text-4xl">
              Get started
            </h2>

            <div className="mt-10 flex flex-col items-center gap-6">
              <div>
                <p className="mb-2 text-sm font-medium text-muted-foreground">
                  Install the CLI
                </p>
                <div className="inline-flex items-center gap-3 rounded-lg border border-border/80 bg-secondary/50 px-5 py-3 font-mono text-sm">
                  <span className="text-muted-foreground/60 select-none">
                    $
                  </span>
                  <span className="text-foreground">
                    cargo install --git https://github.com/shapes-fyi/shapes
                  </span>
                  <CopyButton text="cargo install --git https://github.com/shapes-fyi/shapes" />
                </div>
              </div>

              <div>
                <p className="mb-2 text-sm font-medium text-muted-foreground">
                  Install the Agent Skills
                </p>
                <div className="inline-flex items-center gap-3 rounded-lg border border-border/80 bg-secondary/50 px-5 py-3 font-mono text-sm">
                  <span className="text-muted-foreground/60 select-none">
                    $
                  </span>
                  <span className="text-foreground">
                    npx skills add shapes-fyi/shapes
                  </span>
                  <CopyButton text="npx skills add shapes-fyi/shapes" />
                </div>
              </div>
            </div>

            <p className="mt-8 text-lg leading-relaxed text-muted-foreground">
              Run <code className="text-foreground">shapes init</code> in your
              project to create the graph.
              <br />
              Read the{" "}
              <Link
                to="/specification"
                className="text-primary underline underline-offset-4 transition hover:decoration-primary"
              >
                specification
              </Link>{" "}
              for the full spec.
            </p>
          </div>
        </div>
      </section>

      <SectionDivider />

      {/* ─── Footer ─── */}
      <footer className="py-10">
        <div className="mx-auto flex max-w-5xl flex-col items-center justify-between gap-4 px-14 sm:flex-row sm:px-[4.5rem] md:px-[5.5rem] lg:px-[7.5rem]">
          <div className="text-center sm:text-left">
            <p className="font-serif text-lg font-semibold tracking-[-0.04em]">
              Shapes
            </p>
            <p className="mt-0.5 text-sm text-muted-foreground">
              v0.1.0 — Working Draft
            </p>
          </div>
          <div className="flex items-center gap-6 text-sm text-muted-foreground">
            <Link
              to="/specification"
              className="transition hover:text-foreground"
            >
              Specification
            </Link>
            <a
              href="https://github.com/shapes-fyi/shapes"
              target="_blank"
              rel="noopener noreferrer"
              className="transition hover:text-foreground"
            >
              GitHub
            </a>
          </div>
        </div>
      </footer>
    </main>
  )
}
