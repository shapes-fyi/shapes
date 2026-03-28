import { useEffect, useState } from "react"
import { RiGithubFill } from "@remixicon/react"
import { Link } from "@tanstack/react-router"
import { Button } from "@workspace/ui/components/button"
import { ThemeToggle } from "@/components/spec/theme-toggle"

function Nav({
  variant = "default",
  align = "home",
}: {
  variant?: "default" | "transparent"
  align?: "home" | "spec"
}) {
  const [scrolled, setScrolled] = useState(false)

  useEffect(() => {
    if (variant !== "transparent") return
    function onScroll() {
      setScrolled(window.scrollY > 50)
    }
    onScroll()
    window.addEventListener("scroll", onScroll, { passive: true })
    return () => window.removeEventListener("scroll", onScroll)
  }, [variant])

  const showBg = variant === "default" || scrolled

  return (
    <nav
      className={`fixed top-0 right-0 left-0 z-40 transition-all duration-300 ${
        showBg
          ? "border-b border-border/50 bg-background/90 backdrop-blur-md"
          : "border-b border-transparent"
      }`}
    >
      <div
        className={`mx-auto flex h-14 items-center justify-between ${
          align === "spec"
            ? "max-w-3xl px-6"
            : "max-w-5xl px-14 sm:px-[4.5rem] md:px-[5.5rem] lg:px-[7.5rem]"
        }`}
      >
        <Link
          to="/"
          className="font-serif text-lg font-semibold tracking-[-0.04em] text-foreground transition hover:text-primary"
        >
          Shapes
        </Link>

        <div className="flex items-center gap-0.5">
          <Link
            to="/specification"
            className="rounded-full px-3 py-1.5 text-base text-muted-foreground transition hover:text-foreground"
          >
            Specification
          </Link>
          <a
            href="https://github.com/shapes-fyi/shapes"
            target="_blank"
            rel="noopener noreferrer"
          >
            <Button
              variant="ghost"
              size="icon-lg"
              className="rounded-full hover:rounded-full focus-visible:rounded-full"
              aria-label="GitHub repository"
            >
              <RiGithubFill className="size-5" />
            </Button>
          </a>
          <ThemeToggle />
        </div>
      </div>
    </nav>
  )
}

export { Nav }
