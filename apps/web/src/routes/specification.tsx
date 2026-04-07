import { Fragment, useEffect, useMemo } from "react"
import { RiArrowUpLine } from "@remixicon/react"
import { createFileRoute } from "@tanstack/react-router"
import { Button } from "@workspace/ui/components/button"
import { Container } from "@workspace/ui/components/container"
import { LinkedHeading } from "@/components/spec/linked-heading"
import { Nav } from "@/components/nav"
import { SectionMinimap } from "@/components/spec/section-minimap"
import { SPEC_META, sections } from "@/components/spec/sections"
import { useActiveSection } from "@/components/spec/use-active-section"
import { useKeyboardNav } from "@/components/spec/use-keyboard-nav"
import { useScrollProgress } from "@/components/spec/use-scroll-progress"

export const Route = createFileRoute("/specification")({
  component: SpecPage,
  head: () => ({
    meta: [
      {
        title: "Specification — Shapes",
      },
      {
        name: "description",
        content:
          "The Shapes Specification — a structured graph of intent, constraints, and amendments that AI agents and humans share as a contract over a project.",
      },
    ],
  }),
})

function SpecPage() {
  const allSectionIds = useMemo(
    () =>
      sections.flatMap((s) => [
        s.id,
        ...(s.subsections?.map((sub) => sub.id) ?? []),
      ]),
    []
  )

  // id → "1.2 Shape" style label, used for the tab-title wayfinding effect.
  const sectionLabels = useMemo(() => {
    const map = new Map<string, string>()
    sections.forEach((section, i) => {
      map.set(section.id, `${i + 1}. ${section.title}`)
      section.subsections?.forEach((sub, j) => {
        map.set(sub.id, `${i + 1}.${j + 1} ${sub.title}`)
      })
    })
    return map
  }, [])

  const activeId = useActiveSection(allSectionIds)
  const scrollProgress = useScrollProgress()
  useKeyboardNav(allSectionIds)

  // Tab-title wayfinding: reflect the active section in document.title so
  // readers with multiple tabs open can locate where they left off. Reset
  // on unmount so navigating away does not leave the tab on a stale label.
  useEffect(() => {
    const baseTitle = "Specification — Shapes"
    if (!activeId) {
      document.title = baseTitle
      return
    }
    const label = sectionLabels.get(activeId)
    document.title = label ? `${label} · Shapes Spec` : baseTitle
    return () => {
      document.title = baseTitle
    }
  }, [activeId, sectionLabels])

  return (
    <main
      id="spec-content"
      className="relative min-h-screen overflow-x-clip bg-background text-foreground"
    >
      <a
        href="#spec-content"
        className="sr-only rounded-md bg-background px-4 py-2 font-ui text-sm text-primary shadow-[var(--shadow-elevated)] focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-[60]"
      >
        Skip to content
      </a>

      <div
        className="fixed top-0 left-0 z-50 h-[2px] bg-primary transition-[width] duration-150 ease-out"
        style={{ width: `${scrollProgress}%` }}
        role="progressbar"
        aria-valuenow={Math.round(scrollProgress)}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Reading progress"
      />

      <Nav align="spec" />

      <SectionMinimap activeId={activeId} />

      <Container
        as="article"
        size="wide"
        className="relative pt-24 pb-12 lg:pt-28 lg:pb-16"
      >
        <div className="mx-auto max-w-[68ch]">
          <header className="space-y-14">
            <div className="space-y-8">
              <h1 className="font-serif text-4xl leading-h1 font-semibold tracking-heading sm:text-5xl lg:text-6xl">
                Shapes Specification
              </h1>

              <dl className="space-y-1.5 text-base leading-7 text-muted-foreground">
                {SPEC_META.map((item) => (
                  <div key={item.label}>
                    <dt className="mr-2 inline font-ui text-code font-medium tracking-meta text-foreground uppercase">
                      {item.label}
                    </dt>
                    <dd className="inline">{item.value}</dd>
                  </div>
                ))}
              </dl>
            </div>

            <section className="space-y-5">
              <h2 className="font-serif text-2xl leading-subhead font-semibold tracking-subhead">
                Abstract
              </h2>
              <p className="text-xl leading-read text-foreground">
                AI agents today reconstruct project context from scattered
                artifacts — code, documents, conversations — losing intent and
                constraints along the way. The Shapes Specification replaces
                that reconstruction with a single structured graph: every node
                captures what is to be built, why it matters, and what must hold
                true. The graph is one versioned source of project intent,
                history, and boundaries that AI agents and humans share. The
                specification is domain-agnostic and applies to software,
                research, writing, and any other structured endeavor.
              </p>
            </section>

            <nav className="space-y-5" aria-label="Table of Contents">
              <h2 className="font-serif text-2xl leading-subhead font-semibold tracking-subhead">
                Table of Contents
              </h2>
              <ol className="space-y-2 text-lg leading-read text-foreground tabular-nums">
                {sections.map((section, index) => (
                  <li key={section.id}>
                    <a
                      href={`#${section.id}`}
                      className={`underline underline-offset-4 transition hover:text-primary hover:decoration-primary ${activeId === section.id ? "text-primary decoration-primary" : "decoration-muted-foreground/60"}`}
                    >
                      {index + 1}. {section.title}
                    </a>
                    {section.subsections && (
                      <ol className="mt-2 space-y-1 pl-7 text-base leading-[1.6] text-muted-foreground tabular-nums">
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
                  className="font-serif text-3xl leading-subhead font-semibold tracking-heading sm:text-4xl"
                >
                  {index + 1}. {section.title}
                </LinkedHeading>
                <div className="mt-6 space-y-6 text-lg leading-read text-foreground">
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
                          className="font-serif text-2xl leading-subhead font-semibold tracking-subhead sm:text-3xl"
                        >
                          {index + 1}.{subsectionIndex + 1} {subsection.title}
                        </LinkedHeading>
                        <div className="mt-5 space-y-6 text-lg leading-read text-foreground">
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
      </Container>

      <div
        className={`fixed right-4 bottom-6 z-20 transition-all duration-200 ease-out sm:right-6 ${
          scrollProgress > 20
            ? "pointer-events-auto translate-y-0 opacity-100"
            : "pointer-events-none translate-y-2 opacity-0"
        }`}
      >
        <div className="rounded-full border border-border/80 bg-background/85 p-1 shadow-[var(--shadow-elevated)] backdrop-blur">
          <Button
            variant="ghost"
            size="icon-lg"
            className="rounded-full"
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
