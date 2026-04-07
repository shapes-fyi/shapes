import { useEffect, useState } from "react"
import { RiGithubFill } from "@remixicon/react"
import { Link } from "@tanstack/react-router"
import { Button } from "@workspace/ui/components/button"
import { Container } from "@workspace/ui/components/container"
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
      <Container size={align === "spec" ? "narrow" : "default"}>
        <div className="flex h-14 items-center justify-between px-6 sm:px-10 lg:px-14">
          <Link
            to="/"
            className="font-serif text-lg font-semibold tracking-heading text-foreground transition hover:text-primary"
          >
            Shapes
          </Link>

          <div className="flex items-center gap-0.5">
            <Link
              to="/specification"
              className="rounded-full px-3 py-1.5 font-ui text-sm text-muted-foreground transition hover:text-foreground"
              activeProps={{
                className:
                  "font-ui rounded-full px-3 py-1.5 text-sm text-primary transition",
              }}
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
                className="rounded-full"
                aria-label="GitHub repository"
              >
                <RiGithubFill className="size-5" />
              </Button>
            </a>
            <ThemeToggle />
          </div>
        </div>
      </Container>
    </nav>
  )
}

export { Nav }
