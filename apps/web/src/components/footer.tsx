import { Link } from "@tanstack/react-router"
import { Container } from "@workspace/ui/components/container"

function Footer() {
  return (
    <footer className="py-16 sm:py-20">
      <Container>
        <div className="flex flex-col items-center justify-between gap-4 px-6 sm:flex-row sm:px-10 lg:px-14">
          <div className="text-center sm:text-left">
            <p className="font-serif text-lg font-semibold tracking-heading">
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
      </Container>
    </footer>
  )
}

export { Footer }
