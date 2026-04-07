import { Container } from "@workspace/ui/components/container"
import { cn } from "@workspace/ui/lib/utils"
import type { ComponentPropsWithoutRef } from "react"

type SectionDividerProps = ComponentPropsWithoutRef<"div">

function SectionDivider({ className, ...props }: SectionDividerProps) {
  return (
    <Container
      className={cn("relative my-12 sm:my-20 lg:my-24", className)}
      aria-hidden="true"
      {...props}
    >
      <div className="border-t border-border/40" />
    </Container>
  )
}

export { SectionDivider }
export type { SectionDividerProps }
