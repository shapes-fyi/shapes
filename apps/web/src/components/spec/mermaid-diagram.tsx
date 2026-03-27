import { renderMermaidSVG } from "beautiful-mermaid"
import { cn } from "@workspace/ui/lib/utils"

import type { ReactNode } from "react"

type MermaidDiagramProps = {
  title: string
  diagram: string
  caption?: ReactNode
  className?: string
}

function MermaidDiagram({
  title,
  diagram,
  caption,
  className,
}: MermaidDiagramProps) {
  try {
    const svg = renderMermaidSVG(diagram, {
      bg: "var(--background)",
      fg: "var(--foreground)",
      line: "color-mix(in oklch, var(--foreground) 34%, var(--background))",
      accent: "var(--primary)",
      muted: "color-mix(in oklch, var(--foreground) 58%, var(--background))",
      surface: "color-mix(in oklch, var(--foreground) 2.5%, var(--background))",
      border: "color-mix(in oklch, var(--foreground) 16%, var(--background))",
      font: "Geist Variable, sans-serif",
      transparent: true,
      padding: 28,
      nodeSpacing: 32,
      layerSpacing: 46,
      componentSpacing: 32,
    })

    return (
      <figure className={cn("space-y-4", className)}>
        <div className="overflow-x-auto rounded-[1.75rem] border border-border/80 bg-card/50 px-3 py-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)] sm:px-5 sm:py-6">
          <div
            aria-label={title}
            className="min-w-[42rem] [&_svg]:h-auto [&_svg]:w-full"
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        </div>
        {caption && (
          <figcaption className="max-w-3xl text-sm leading-[1.65] text-muted-foreground">
            {caption}
          </figcaption>
        )}
      </figure>
    )
  } catch (error) {
    const message =
      error instanceof Error
        ? error.message
        : "Unable to render Mermaid diagram"

    return (
      <figure className={cn("space-y-4", className)}>
        <div className="rounded-[1.75rem] border border-destructive/40 bg-destructive/8 px-5 py-4 text-sm text-destructive">
          Diagram render error: {message}
        </div>
        {caption && (
          <figcaption className="max-w-3xl text-sm leading-[1.65] text-muted-foreground">
            {caption}
          </figcaption>
        )}
      </figure>
    )
  }
}

export { MermaidDiagram }
