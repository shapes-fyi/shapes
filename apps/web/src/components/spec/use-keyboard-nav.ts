import { useEffect, useRef } from "react"

/* ── useKeyboardNav ──
 *
 * Vim-style keyboard navigation for a long scrollable document.
 *
 *   j        next section
 *   k        previous section
 *   G        last section (end)
 *   g g      first section (top)
 *
 * Ignores events when focus is inside an input, textarea, select, or
 * contenteditable element. Honors prefers-reduced-motion: when reduce
 * is requested, scroll behavior degrades to instant.
 *
 * Reuses the spec page's existing allSectionIds array as the source
 * of truth for section ordering.
 */
function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  const tag = target.tagName
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true
  if (target.isContentEditable) return true
  return false
}

function findCurrentSection(sectionIds: ReadonlyArray<string>): number {
  // The current section is the last one whose top edge is above the
  // 25% scroll line — matches use-active-section's intersection cutoff.
  const cutoff = window.innerHeight * 0.25
  let current = 0
  for (let i = 0; i < sectionIds.length; i++) {
    const el = document.getElementById(sectionIds[i])
    if (!el) continue
    const top = el.getBoundingClientRect().top
    if (top - cutoff <= 0) current = i
    else break
  }
  return current
}

function scrollToSection(id: string, instant: boolean) {
  const el = document.getElementById(id)
  if (!el) return
  el.scrollIntoView({
    behavior: instant ? "auto" : "smooth",
    block: "start",
  })
}

function useKeyboardNav(sectionIds: ReadonlyArray<string>) {
  const lastG = useRef<number>(0)

  useEffect(() => {
    if (sectionIds.length === 0) return

    const reduceMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches

    function onKeyDown(event: KeyboardEvent) {
      if (event.metaKey || event.ctrlKey || event.altKey) return
      if (isEditableTarget(event.target)) return

      const key = event.key

      if (key === "j") {
        event.preventDefault()
        const cur = findCurrentSection(sectionIds)
        const next = Math.min(cur + 1, sectionIds.length - 1)
        scrollToSection(sectionIds[next], reduceMotion)
        return
      }

      if (key === "k") {
        event.preventDefault()
        const cur = findCurrentSection(sectionIds)
        const prev = Math.max(cur - 1, 0)
        scrollToSection(sectionIds[prev], reduceMotion)
        return
      }

      if (key === "G") {
        event.preventDefault()
        scrollToSection(sectionIds[sectionIds.length - 1], reduceMotion)
        return
      }

      if (key === "g") {
        event.preventDefault()
        const now = Date.now()
        if (now - lastG.current < 500) {
          // Second g within 500ms — jump to top
          window.scrollTo({
            top: 0,
            behavior: reduceMotion ? "auto" : "smooth",
          })
          lastG.current = 0
        } else {
          lastG.current = now
        }
        return
      }
    }

    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [sectionIds])
}

export { useKeyboardNav }
