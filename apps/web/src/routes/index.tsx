import { Fragment, useMemo } from "react"
import { RiArrowUpLine } from "@remixicon/react"
import { createFileRoute } from "@tanstack/react-router"
import { Button } from "@workspace/ui/components/button"
import { LinkedHeading } from "@/components/spec/linked-heading"
import { SectionMinimap } from "@/components/spec/section-minimap"
import { SPEC_META, sections } from "@/components/spec/sections"
import { ThemeToggle } from "@/components/spec/theme-toggle"
import { useActiveSection } from "@/components/spec/use-active-section"
import { useScrollProgress } from "@/components/spec/use-scroll-progress"

export const Route = createFileRoute("/")({ component: SpecPage })

function SpecPage() {
  const allSectionIds = useMemo(
    () =>
      sections.flatMap((s) => [
        s.id,
        ...(s.subsections?.map((sub) => sub.id) ?? []),
      ]),
    []
  )
  const activeId = useActiveSection(allSectionIds)
  const scrollProgress = useScrollProgress()

  return (
    <main className="relative min-h-screen overflow-x-clip bg-background text-foreground">
      <div
        className="fixed top-0 left-0 z-50 h-[2px] bg-primary transition-[width] duration-150 ease-out"
        style={{ width: `${scrollProgress}%` }}
        role="progressbar"
        aria-valuenow={Math.round(scrollProgress)}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Reading progress"
      />
      <div className="fixed top-4 right-4 z-20 sm:top-6 sm:right-6">
        <div className="rounded-full border border-border/80 bg-background/85 p-1 shadow-[var(--shadow-elevated)] backdrop-blur">
          <ThemeToggle />
        </div>
      </div>

      <SectionMinimap activeId={activeId} />

      <article className="relative mx-auto max-w-[78rem] px-6 py-12 sm:px-10 lg:px-16 lg:py-16">
        <div className="mx-auto max-w-3xl">
          <header className="space-y-14">
            <div className="space-y-8">
              <h1 className="font-serif text-3xl leading-none font-semibold tracking-[-0.04em] sm:text-5xl">
                Shapes Specification
              </h1>

              <dl className="space-y-1.5 text-[length:var(--spec-meta)] leading-7 text-muted-foreground">
                {SPEC_META.map((item) => (
                  <div key={item.label}>
                    <dt className="mr-2 inline font-semibold text-foreground">
                      {item.label}:
                    </dt>
                    <dd className="inline">{item.value}</dd>
                  </div>
                ))}
              </dl>
            </div>

            <section className="space-y-4">
              <h2 className="font-serif text-2xl font-semibold">Abstract</h2>
              <p className="max-w-2xl text-[length:var(--spec-abstract)] leading-7 text-muted-foreground">
                Agents reconstruct project context from scattered artifacts —
                code, documents, conversations — losing intent and constraints
                along the way. Shapes eliminates this by defining a structured
                graph where each node captures what is to be built, why it
                matters, and what must hold true. The graph is the shared
                contract between agents and humans: a single, versionable
                representation of a project's intent, history, and boundaries.
                The protocol is domain-agnostic and applies to software,
                research, writing, and any other structured endeavor.
              </p>
            </section>

            <nav className="space-y-5" aria-label="Table of Contents">
              <h2 className="font-serif text-2xl font-semibold">
                Table of Contents
              </h2>
              <ol className="space-y-2 text-[length:var(--spec-toc)] leading-[1.65] text-muted-foreground tabular-nums">
                {sections.map((section, index) => (
                  <li key={section.id}>
                    <a
                      href={`#${section.id}`}
                      className={`underline underline-offset-4 transition hover:text-primary hover:decoration-primary ${activeId === section.id ? "text-primary decoration-primary" : "text-foreground decoration-muted-foreground/60"}`}
                    >
                      {index + 1}. {section.title}
                    </a>
                    {section.subsections && (
                      <ol className="mt-2 space-y-1 pl-7 text-[length:var(--spec-toc-sub)] leading-[1.55] text-muted-foreground tabular-nums">
                        {section.subsections.map(
                          (subsection, subsectionIndex) => (
                            <li key={subsection.id}>
                              <a
                                href={`#${subsection.id}`}
                                className={`underline underline-offset-4 transition hover:text-primary hover:decoration-primary ${activeId === subsection.id ? "text-primary decoration-primary" : "decoration-muted-foreground/50"}`}
                              >
                                {index + 1}.{subsectionIndex + 1}{" "}
                                {subsection.title}
                              </a>
                            </li>
                          )
                        )}
                      </ol>
                    )}
                  </li>
                ))}
              </ol>
            </nav>
          </header>

          {sections.map((section, index) => (
            <Fragment key={section.id}>
              <hr className="hr-ornament my-12 sm:my-16" />
              <section id={section.id} className="scroll-mt-4">
                <LinkedHeading
                  id={section.id}
                  level={2}
                  className="font-serif text-[length:var(--spec-h2)] leading-tight font-semibold tracking-[-0.02em] sm:text-[length:var(--spec-h2-sm)]"
                >
                  {index + 1}. {section.title}
                </LinkedHeading>
                <div className="mt-6 space-y-6 text-[length:var(--spec-body)] leading-[1.65] tracking-[-0.01em] text-muted-foreground">
                  {section.content}
                </div>

                {section.subsections && (
                  <div className="mt-12 space-y-12">
                    {section.subsections.map((subsection, subsectionIndex) => (
                      <section
                        key={subsection.id}
                        id={subsection.id}
                        className="scroll-mt-4"
                      >
                        <LinkedHeading
                          id={subsection.id}
                          level={3}
                          className="font-serif text-[length:var(--spec-h3)] leading-tight font-semibold tracking-[-0.02em] sm:text-[length:var(--spec-h3-sm)]"
                        >
                          {index + 1}.{subsectionIndex + 1} {subsection.title}
                        </LinkedHeading>
                        <div className="mt-5 space-y-6 text-[length:var(--spec-body-sub)] leading-[1.65] tracking-[-0.01em] text-muted-foreground">
                          {subsection.content}
                        </div>
                      </section>
                    ))}
                  </div>
                )}
              </section>
            </Fragment>
          ))}
        </div>
      </article>

      <div
        className={`fixed bottom-6 right-4 z-20 transition-all duration-200 ease-out sm:right-6 ${
          scrollProgress > 90
            ? "pointer-events-auto translate-y-0 opacity-100"
            : "pointer-events-none translate-y-2 opacity-0"
        }`}
      >
        <div className="rounded-full border border-border/80 bg-background/85 p-1 shadow-[var(--shadow-elevated)] backdrop-blur">
          <Button
            variant="ghost"
            size="icon"
            className="rounded-full hover:rounded-full focus-visible:rounded-full"
            onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
            aria-label="Back to top"
          >
            <RiArrowUpLine className="size-4" />
          </Button>
        </div>
      </div>
    </main>
  )
}
