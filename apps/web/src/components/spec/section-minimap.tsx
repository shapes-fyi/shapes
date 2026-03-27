import { useEffect, useRef, useState } from "react"
import { sections } from "@/components/spec/sections"

export function SectionMinimap({ activeId }: { activeId: string }) {
  const [visible, setVisible] = useState(false)
  const [open, setOpen] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleScroll() {
      setVisible(window.scrollY > 400)
    }
    window.addEventListener("scroll", handleScroll, { passive: true })
    handleScroll()
    return () => window.removeEventListener("scroll", handleScroll)
  }, [])

  function handleEnter() {
    if (timeoutRef.current) clearTimeout(timeoutRef.current)
    setOpen(true)
  }

  function handleLeave() {
    timeoutRef.current = setTimeout(() => setOpen(false), 200)
  }

  const activeSectionIndex = sections.findIndex((s) => {
    if (s.id === activeId) return true
    return s.subsections?.some((sub) => sub.id === activeId)
  })

  return (
    <div
      ref={containerRef}
      className={`fixed top-1/2 right-4 z-30 -translate-y-1/2 p-4 -m-4 transition-all duration-200 ease-out sm:right-6 ${
        visible
          ? "pointer-events-auto translate-x-0 opacity-100"
          : "pointer-events-none translate-x-3 opacity-0"
      }`}
      onMouseEnter={handleEnter}
      onMouseLeave={handleLeave}
    >
      {/* Minimap bars */}
      <div
        className={`flex flex-col items-end gap-1.5 transition-opacity duration-200 ${open ? "pointer-events-none opacity-0" : "opacity-100"}`}
        aria-hidden="true"
      >
        {sections.map((section, index) => (
          <div
            key={section.id}
            className={`rounded-full transition-all duration-200 ${
              index === activeSectionIndex
                ? "h-[3px] w-6 bg-primary"
                : "h-[2px] w-2.5 bg-muted-foreground/25"
            }`}
          />
        ))}
      </div>

      {/* Popover TOC */}
      <nav
        aria-label="Section navigation"
        className={`absolute top-1/2 right-0 -translate-y-1/2 transition-all duration-200 ${
          open
            ? "pointer-events-auto translate-x-0 opacity-100"
            : "pointer-events-none translate-x-2 opacity-0"
        }`}
      >
        <div className="min-w-[14rem] rounded-xl border border-border/80 bg-background/95 px-4 py-3 shadow-[var(--shadow-popover)] backdrop-blur">
          <ul className="space-y-1 text-[0.8125rem] leading-relaxed whitespace-nowrap tabular-nums">
            {sections.map((section, index) => {
              const isSectionActive =
                activeId === section.id ||
                section.subsections?.some((sub) => sub.id === activeId)

              return (
                <li key={section.id}>
                  <a
                    href={`#${section.id}`}
                    onClick={() => setOpen(false)}
                    className={`block rounded-md px-2 py-0.5 transition-colors ${
                      activeId === section.id
                        ? "text-primary"
                        : "text-foreground/80 hover:text-primary"
                    }`}
                  >
                    {index + 1}. {section.title}
                  </a>
                  {section.subsections && isSectionActive && (
                    <ul className="mt-0.5 space-y-0.5 pl-4">
                      {section.subsections.map((sub, subIndex) => (
                        <li key={sub.id}>
                          <a
                            href={`#${sub.id}`}
                            onClick={() => setOpen(false)}
                            className={`block rounded-md px-2 py-0.5 text-xs transition-colors ${
                              activeId === sub.id
                                ? "text-primary"
                                : "text-muted-foreground hover:text-primary"
                            }`}
                          >
                            {index + 1}.{subIndex + 1} {sub.title}
                          </a>
                        </li>
                      ))}
                    </ul>
                  )}
                </li>
              )
            })}
          </ul>
        </div>
      </nav>
    </div>
  )
}
