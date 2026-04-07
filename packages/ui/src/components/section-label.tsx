import type { ComponentPropsWithoutRef } from "react"
import { cn } from "@workspace/ui/lib/utils"

type SectionLabelProps = {
  number?: number
} & ComponentPropsWithoutRef<"p">

function SectionLabel({
  number,
  className,
  children,
  ...props
}: SectionLabelProps) {
  return (
    <p
      className={cn(
        "font-ui text-label font-medium tracking-label text-muted-foreground uppercase",
        className
      )}
      {...props}
    >
      {typeof number === "number" && (
        <>
          <span className="text-primary/80 tabular-nums">
            §{number.toString().padStart(2, "0")}
          </span>
          <span aria-hidden="true" className="mx-2 text-muted-foreground/40">
            ·
          </span>
        </>
      )}
      {children}
    </p>
  )
}

export { SectionLabel }
export type { SectionLabelProps }
