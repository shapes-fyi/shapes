import type { ComponentPropsWithoutRef, ElementType } from "react"
import { cn } from "@workspace/ui/lib/utils"

type ContainerSize = "default" | "wide" | "narrow"

const SIZE_CLASS: Record<ContainerSize, string> = {
  default: "max-w-5xl",
  wide: "max-w-[78rem]",
  narrow: "max-w-3xl",
}

type ContainerProps<T extends ElementType = "div"> = {
  as?: T
  size?: ContainerSize
} & Omit<ComponentPropsWithoutRef<T>, "as">

function Container<T extends ElementType = "div">({
  as,
  size = "default",
  className,
  ...props
}: ContainerProps<T>) {
  const Component = (as ?? "div") as ElementType
  return (
    <Component
      className={cn(
        "mx-auto w-full px-6 sm:px-10 lg:px-16",
        SIZE_CLASS[size],
        className
      )}
      {...props}
    />
  )
}

export { Container }
export type { ContainerProps, ContainerSize }
