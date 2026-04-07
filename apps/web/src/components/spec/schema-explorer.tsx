import { useCallback, useEffect, useMemo, useRef, useState } from "react"

import { cn } from "@workspace/ui/lib/utils"
import type { ReactNode } from "react"
import type {
  SchemaDefinition,
  SchemaField,
} from "@/components/spec/schema-data"
import { TokenSpan, highlightYaml } from "@/components/spec/code-block"

type FieldRegion = {
  key: string
  startLine: number
  endLine: number // exclusive
}

function buildYaml(schema: SchemaDefinition): string {
  // For the Constraint schema the fields don't have a wrapping parent key
  const hasTopLevelIndent = schema.fields[0].lines.startsWith("  ")
  if (hasTopLevelIndent) {
    return `${schema.name}:\n${schema.fields.map((f) => f.lines).join("\n")}`
  }
  return schema.fields.map((f) => f.lines).join("\n")
}

function buildRegions(schema: SchemaDefinition): Array<FieldRegion> {
  const regions: Array<FieldRegion> = []
  const hasTopLevelIndent = schema.fields[0].lines.startsWith("  ")

  let currentLine = hasTopLevelIndent ? 1 : 0
  for (const field of schema.fields) {
    const fieldLineCount = field.lines.split("\n").length
    regions.push({
      key: field.key,
      startLine: currentLine,
      endLine: currentLine + fieldLineCount,
    })
    currentLine += fieldLineCount
  }

  return regions
}

function SchemaExplorer({
  schema,
  caption,
  className,
}: {
  schema: SchemaDefinition
  caption?: ReactNode
  className?: string
}) {
  const yaml = useMemo(() => buildYaml(schema), [schema])
  const tokenizedLines = useMemo(() => highlightYaml(yaml), [yaml])
  const regions = useMemo(() => buildRegions(schema), [schema])

  const fieldMap = useMemo(() => {
    const map = new Map<string, SchemaField>()
    for (const field of schema.fields) {
      map.set(field.key, field)
    }
    return map
  }, [schema.fields])

  const firstFieldKey = schema.fields[0].key

  // Selection state — default to first field
  const [selectedKey, setSelectedKey] = useState(firstFieldKey)
  const [displayedKey, setDisplayedKey] = useState(firstFieldKey)
  const [visible, setVisible] = useState(true)
  const [revealKey, setRevealKey] = useState(0)
  // Track hovered region via DOM attribute instead of React state
  // to avoid re-rendering all lines on every hover change
  const preRef = useRef<HTMLPreElement>(null)
  const handlePointerOver = useCallback((e: React.PointerEvent) => {
    const line = (e.target as Element).closest<HTMLElement>("[data-region]")
    const region = line?.dataset.region ?? null
    const pre = preRef.current
    if (pre) {
      if (region) {
        pre.dataset.hovered = region
      } else {
        delete pre.dataset.hovered
      }
    }
  }, [])
  const handlePointerLeave = useCallback(() => {
    const pre = preRef.current
    if (pre) delete pre.dataset.hovered
  }, [])

  // Animation: fade out → swap → fade in
  useEffect(() => {
    if (selectedKey === displayedKey) return
    setVisible(false)
    const timeout = setTimeout(() => {
      setDisplayedKey(selectedKey)
      setRevealKey((k) => k + 1)
      setVisible(true)
    }, 150)
    return () => clearTimeout(timeout)
  }, [selectedKey, displayedKey])

  const displayedField = fieldMap.get(displayedKey) ?? schema.fields[0]
  const displayedLabel = displayedField.label
  const displayedDescription = displayedField.description

  // Build a lookup: lineIndex → region key
  const lineToRegion = useMemo(() => {
    const map = new Map<number, string>()
    for (const region of regions) {
      for (let i = region.startLine; i < region.endLine; i++) {
        map.set(i, region.key)
      }
    }
    return map
  }, [regions])

  // Generate CSS for hover states — avoids React re-renders on hover
  const schemaId = useMemo(
    () => `se-${schema.name.toLowerCase().replace(/\W/g, "")}`,
    [schema.name]
  )
  const hoverCss = useMemo(() => {
    const rules = regions.map(
      (r) =>
        `#${schemaId}[data-hovered="${r.key}"] [data-region="${r.key}"][data-first] .su{text-decoration-color:color-mix(in srgb,var(--primary) 50%,transparent)}`
    )
    return rules.join("\n")
  }, [regions, schemaId])

  return (
    <figure className={cn("group/code space-y-4", className)}>
      {/* Schema description — static, above the interactive panel */}
      <div className="text-lg leading-read text-muted-foreground">
        {schema.description}
      </div>

      <div className="group/schema rounded-lg border border-border/80 bg-secondary/60">
        <div className="flex flex-col lg:flex-row lg:items-start">
          {/* Left: YAML with selectable fields */}
          <div className="relative min-w-0 lg:w-1/2">
            <style dangerouslySetInnerHTML={{ __html: hoverCss }} />
            <pre
              ref={preRef}
              id={schemaId}
              onPointerOver={handlePointerOver}
              onPointerLeave={handlePointerLeave}
              className="yaml-scroll h-full overflow-x-auto overflow-y-auto pt-5 pr-6 pb-4 pl-5 text-code leading-[1.15]"
            >
              <code className="font-mono">
                {tokenizedLines.map((tokens, lineIdx) => {
                  const regionKey = lineToRegion.get(lineIdx)
                  const isSelected = regionKey === selectedKey
                  const isFirstLineOfRegion =
                    regionKey !== undefined &&
                    lineToRegion.get(lineIdx - 1) !== regionKey

                  const renderToken = (
                    token: (typeof tokens)[number],
                    i: number
                  ) =>
                    typeof token === "string" ? (
                      token
                    ) : (
                      <TokenSpan key={i} token={token} />
                    )

                  return (
                    <span
                      key={lineIdx}
                      className={cn(
                        "block",
                        regionKey !== undefined && "cursor-pointer"
                      )}
                      data-region={regionKey}
                      data-first={isFirstLineOfRegion ? "" : undefined}
                      onClick={
                        regionKey !== undefined
                          ? () => setSelectedKey(regionKey)
                          : undefined
                      }
                    >
                      {lineIdx > 0 && "\n"}
                      {isFirstLineOfRegion
                        ? (() => {
                            const hasIndent =
                              typeof tokens[0] === "string" &&
                              tokens[0].trim() === "" &&
                              tokens.length > 1
                            const indent = hasIndent ? tokens[0] : null
                            const content = hasIndent ? tokens.slice(1) : tokens

                            return (
                              <>
                                {indent != null &&
                                  (typeof indent === "string" ? indent : null)}
                                <span
                                  className={cn(
                                    "su underline decoration-dashed underline-offset-4 transition-[text-decoration-color] duration-200",
                                    isSelected
                                      ? "decoration-primary/80"
                                      : "decoration-transparent group-hover/schema:decoration-primary/30"
                                  )}
                                >
                                  {content.map((t, i) =>
                                    renderToken(t, hasIndent ? i + 1 : i)
                                  )}
                                </span>
                              </>
                            )
                          })()
                        : tokens.map(renderToken)}
                    </span>
                  )
                })}
              </code>
            </pre>
          </div>

          {/* Divider */}
          <div className="hidden border-l border-border/60 lg:block" />

          {/* Right: field description */}
          <div className="min-w-0 border-t border-border/60 lg:sticky lg:top-0 lg:w-1/2 lg:border-t-0">
            <div
              key={revealKey}
              className="h-full px-6 py-4"
              onAnimationEnd={(e) => {
                if (e.animationName === "yaml-reveal") {
                  ;(e.currentTarget as HTMLElement).style.willChange = "auto"
                }
              }}
              style={
                visible
                  ? {
                      animation:
                        "yaml-reveal 350ms cubic-bezier(0.16, 1, 0.3, 1) both",
                      willChange: "transform, filter, opacity",
                    }
                  : {
                      opacity: 0,
                      transition: "opacity 150ms ease-in",
                    }
              }
            >
              <p className="mb-3 text-sm font-semibold tracking-widest text-muted-foreground/70 uppercase">
                {displayedLabel}
              </p>
              <div
                className="text-lg leading-read text-muted-foreground"
                aria-live="polite"
              >
                {displayedDescription}
              </div>
            </div>
          </div>
        </div>
      </div>
      {caption && (
        <figcaption className="text-sm leading-read text-muted-foreground">
          {caption}
        </figcaption>
      )}
    </figure>
  )
}

export { SchemaExplorer }
