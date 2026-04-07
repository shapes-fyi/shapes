import { useMemo, useState } from "react"
import { RiCheckLine, RiFileCopyLine } from "@remixicon/react"
import { cn } from "@workspace/ui/lib/utils"
import type { ReactNode } from "react"

type Token = {
  type: "key" | "value" | "comment" | "sign" | "string" | "type"
  text: string
}

const TYPE_KEYWORDS = new Set([
  "string",
  "string?",
  "int",
  "bool",
  "boolean",
  "boolean?",
  "iso8601",
  "[string]",
  "[string]?",
])

function tokenizeYamlLine(line: string): Array<Token | string> {
  const trimmed = line.trimStart()
  const indent = line.slice(0, line.length - trimmed.length)

  // Full-line comment
  if (trimmed.startsWith("#")) {
    return [indent, { type: "comment", text: trimmed }]
  }

  const tokens: Array<Token | string> = [indent]

  // Check for list marker
  let rest = trimmed
  if (rest.startsWith("- ")) {
    tokens.push({ type: "sign", text: "- " })
    rest = rest.slice(2)
  }

  // Check for key: value pattern
  const colonMatch = rest.match(/^([^:]+?)(:\s*)(.*)$/)
  if (colonMatch) {
    const [, key, colon, value] = colonMatch
    tokens.push({ type: "key", text: key })
    tokens.push({ type: "sign", text: colon })

    if (value) {
      // Check for inline comment
      const commentIdx = findInlineComment(value)
      const valuePart = commentIdx >= 0 ? value.slice(0, commentIdx) : value
      const commentPart = commentIdx >= 0 ? value.slice(commentIdx) : ""

      if (valuePart.trim()) {
        tokenizeValue(valuePart.trimEnd(), tokens)
        if (commentPart) {
          tokens.push(" ")
        }
      }
      if (commentPart) {
        tokens.push({ type: "comment", text: commentPart })
      }
    }
  } else if (rest.startsWith(">") || rest.startsWith("|")) {
    tokens.push({ type: "sign", text: rest })
  } else if (rest) {
    tokenizeValue(rest, tokens)
  }

  return tokens
}

function tokenizeValue(value: string, tokens: Array<Token | string>) {
  // Quoted string
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    tokens.push({ type: "string", text: value })
    return
  }

  // Type keyword
  if (TYPE_KEYWORDS.has(value.trim())) {
    tokens.push({ type: "type", text: value })
    return
  }

  // Pipe-separated values (e.g. "proposed | promoted | canonical")
  if (value.includes(" | ")) {
    const parts = value.split(/( \| )/)
    for (const part of parts) {
      if (part === " | ") {
        tokens.push({ type: "sign", text: part })
      } else {
        tokens.push({ type: "value", text: part })
      }
    }
    return
  }

  tokens.push({ type: "value", text: value })
}

function findInlineComment(value: string): number {
  let inString = false
  let quote = ""
  for (let i = 0; i < value.length; i++) {
    const ch = value[i]
    if (inString) {
      if (ch === quote) inString = false
    } else if (ch === '"' || ch === "'") {
      inString = true
      quote = ch
    } else if (ch === "#" && (i === 0 || value[i - 1] === " ")) {
      return i
    }
  }
  return -1
}

function highlightYaml(code: string): Array<Array<Token | string>> {
  return code.split("\n").map(tokenizeYamlLine)
}

const COMMENT_LINK_RE = /\[([^\]]+)\]\(([^)]+)\)/g

function CommentWithLinks({ text }: { text: string }) {
  const parts: Array<ReactNode> = []
  let lastIndex = 0
  let match: RegExpExecArray | null

  COMMENT_LINK_RE.lastIndex = 0
  while ((match = COMMENT_LINK_RE.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index))
    }
    parts.push(
      <a
        key={match.index}
        href={match[2]}
        className="not-italic underline decoration-muted-foreground/30 underline-offset-2 transition hover:text-primary hover:decoration-primary"
      >
        {match[1]}
      </a>
    )
    lastIndex = COMMENT_LINK_RE.lastIndex
  }
  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex))
  }

  return <>{parts}</>
}

function TokenSpan({ token }: { token: Token }) {
  const colorClass = {
    key: "text-foreground font-medium",
    value: "text-muted-foreground",
    comment: "text-muted-foreground/50 italic",
    sign: "text-muted-foreground/70",
    string: "text-primary",
    type: "text-primary/80",
  }[token.type]

  COMMENT_LINK_RE.lastIndex = 0
  if (token.type === "comment" && COMMENT_LINK_RE.test(token.text)) {
    return (
      <span className={colorClass}>
        <CommentWithLinks text={token.text} />
      </span>
    )
  }

  return <span className={colorClass}>{token.text}</span>
}

function HighlightedCode({ children }: { children: string }) {
  const lines = useMemo(() => highlightYaml(children), [children])

  return (
    <>
      {lines.map((tokens, lineIdx) => (
        <span key={lineIdx}>
          {lineIdx > 0 && "\n"}
          {tokens.map((token, tokenIdx) =>
            typeof token === "string" ? (
              token
            ) : (
              <TokenSpan key={tokenIdx} token={token} />
            )
          )}
        </span>
      ))}
    </>
  )
}

function CodeBlock({
  language,
  children,
  caption,
  className,
  wrap = false,
}: {
  language?: string
  children: string
  caption?: ReactNode
  className?: string
  /**
   * When true, long lines wrap (whitespace-pre-wrap + break-all) instead
   * of horizontally scrolling. Use for YAML and shell snippets where
   * wrapping is acceptable. Leave false for ASCII tree art that relies
   * on monospace alignment.
   */
  wrap?: boolean
}) {
  const [copied, setCopied] = useState(false)

  function handleCopy() {
    navigator.clipboard.writeText(children).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  const highlighted = language === "yaml"

  return (
    <figure
      className={cn("group/code relative", caption && "space-y-4", className)}
    >
      <div className="relative">
        <div className="absolute top-3 right-4 flex items-center gap-2">
          <button
            type="button"
            onClick={handleCopy}
            aria-label={copied ? "Copied" : "Copy code"}
            className="rounded-md p-2 text-muted-foreground opacity-0 transition group-hover/code:opacity-100 hover:text-foreground focus-visible:opacity-100 [@media(pointer:coarse)]:opacity-100"
          >
            {copied ? (
              <RiCheckLine className="size-3.5" />
            ) : (
              <RiFileCopyLine className="size-3.5" />
            )}
          </button>
          {language && (
            <span className="font-ui text-tiny font-medium tracking-label-tight text-muted-foreground uppercase select-none">
              {language}
            </span>
          )}
        </div>
        <pre
          className={cn(
            "rounded-xl border border-border/80 bg-secondary/60 px-5 py-5 text-code leading-relaxed sm:px-6",
            wrap
              ? "break-all whitespace-pre-wrap"
              : "yaml-scroll overflow-x-auto"
          )}
        >
          <code className="font-mono">
            {highlighted ? (
              <HighlightedCode>{children}</HighlightedCode>
            ) : (
              children
            )}
          </code>
        </pre>
      </div>
      {caption && (
        <figcaption className="max-w-[68ch] text-sm leading-read text-muted-foreground">
          {caption}
        </figcaption>
      )}
    </figure>
  )
}

export { CodeBlock, HighlightedCode }
export { highlightYaml, tokenizeYamlLine, TokenSpan, CommentWithLinks }
export type { Token }
