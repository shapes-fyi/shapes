import { useEffect, useState } from "react"
import { RiMoonLine, RiSunLine } from "@remixicon/react"
import { Button } from "@workspace/ui/components/button"

function ThemeToggle() {
  const [dark, setDark] = useState(false)

  useEffect(() => {
    setDark(document.documentElement.classList.contains("dark"))
  }, [])

  function toggle() {
    const next = !dark
    setDark(next)
    document.documentElement.classList.toggle("dark", next)
    localStorage.setItem("theme", next ? "dark" : "light")
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      className="rounded-full hover:rounded-full focus-visible:rounded-full"
      onClick={toggle}
      aria-label="Toggle theme"
    >
      {dark ? (
        <RiSunLine className="size-4" />
      ) : (
        <RiMoonLine className="size-4" />
      )}
    </Button>
  )
}

export { ThemeToggle }
