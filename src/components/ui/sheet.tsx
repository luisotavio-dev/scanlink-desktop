import { cn } from "@/lib/utils"
import { X } from "lucide-react"
import * as React from "react"

interface SheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  children: React.ReactNode
}

interface SheetContentProps {
  children: React.ReactNode
  className?: string
  side?: "right" | "left" | "top" | "bottom"
}

interface SheetHeaderProps {
  children: React.ReactNode
  className?: string
}

interface SheetTitleProps {
  children: React.ReactNode
  className?: string
}

interface SheetDescriptionProps {
  children: React.ReactNode
  className?: string
}

const SheetContext = React.createContext<{
  open: boolean
  onOpenChange: (open: boolean) => void
}>({
  open: false,
  onOpenChange: () => {},
})

export function Sheet({ open, onOpenChange, children }: SheetProps) {
  return (
    <SheetContext.Provider value={{ open, onOpenChange }}>
      {children}
    </SheetContext.Provider>
  )
}

export function SheetTrigger({ children, asChild }: { children: React.ReactNode; asChild?: boolean }) {
  const { onOpenChange } = React.useContext(SheetContext)

  if (asChild && React.isValidElement(children)) {
    return React.cloneElement(children as React.ReactElement<{ onClick?: () => void }>, {
      onClick: () => onOpenChange(true),
    })
  }

  return (
    <button onClick={() => onOpenChange(true)}>
      {children}
    </button>
  )
}

export function SheetContent({ children, className, side = "right" }: SheetContentProps) {
  const { open, onOpenChange } = React.useContext(SheetContext)

  React.useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && open) {
        onOpenChange(false)
      }
    }
    document.addEventListener("keydown", handleEscape)
    return () => document.removeEventListener("keydown", handleEscape)
  }, [open, onOpenChange])

  if (!open) return null

  const slideClasses = {
    right: "right-0 top-0 h-full w-full max-w-md translate-x-0 data-[state=closed]:translate-x-full",
    left: "left-0 top-0 h-full w-full max-w-md translate-x-0 data-[state=closed]:-translate-x-full",
    top: "top-0 left-0 w-full h-auto max-h-[85vh] translate-y-0 data-[state=closed]:-translate-y-full",
    bottom: "bottom-0 left-0 w-full h-auto max-h-[85vh] translate-y-0 data-[state=closed]:translate-y-full",
  }

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm animate-in fade-in-0"
        onClick={() => onOpenChange(false)}
      />

      {/* Sheet */}
      <div
        data-state={open ? "open" : "closed"}
        className={cn(
          "fixed z-50 bg-[var(--background-card)] border-[var(--border)] shadow-theme-lg",
          "animate-in duration-300",
          side === "right" && "slide-in-from-right border-l",
          side === "left" && "slide-in-from-left border-r",
          side === "top" && "slide-in-from-top border-b rounded-b-2xl",
          side === "bottom" && "slide-in-from-bottom border-t rounded-t-2xl",
          slideClasses[side],
          className
        )}
      >
        <button
          onClick={() => onOpenChange(false)}
          className="absolute right-4 top-4 rounded-lg p-1.5 hover:bg-[var(--surface)] text-[var(--foreground-muted)] hover:text-[var(--foreground)] transition-colors"
        >
          <X className="h-5 w-5" />
          <span className="sr-only">Close</span>
        </button>
        {children}
      </div>
    </>
  )
}

export function SheetHeader({ children, className }: SheetHeaderProps) {
  return (
    <div className={cn("flex flex-col space-y-2 p-6 pb-4", className)}>
      {children}
    </div>
  )
}

export function SheetTitle({ children, className }: SheetTitleProps) {
  return (
    <h2 className={cn("text-lg font-semibold text-[var(--foreground)]", className)}>
      {children}
    </h2>
  )
}

export function SheetDescription({ children, className }: SheetDescriptionProps) {
  return (
    <p className={cn("text-sm text-[var(--foreground-muted)]", className)}>
      {children}
    </p>
  )
}
