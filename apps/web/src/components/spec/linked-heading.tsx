import type { ReactNode } from "react"

export function LinkedHeading({
  id,
  level,
  children,
  className,
}: {
  id: string
  level: 2 | 3
  children: ReactNode
  className?: string
}) {
  const Tag = level === 2 ? "h2" : "h3"

  return (
    <Tag id={id} className={className}>
      <a
        href={`#${id}`}
        aria-label={`Link to section: ${typeof children === "string" ? children : id}`}
        className="group inline-flex items-center gap-3 decoration-transparent transition hover:text-primary active:scale-[0.97]"
      >
        <span>{children}</span>
        <span
          aria-hidden="true"
          className="text-base text-muted-foreground opacity-0 transition group-hover:opacity-100 group-focus-visible:opacity-100"
        >
          #
        </span>
      </a>
    </Tag>
  )
}
