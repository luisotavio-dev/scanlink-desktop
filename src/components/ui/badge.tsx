import * as React from "react"

import { cn } from "@/lib/utils"

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'secondary' | 'destructive' | 'outline' | 'success'
}

function Badge({ className, variant = 'default', ...props }: BadgeProps) {
  return (
    <div
      className={cn(
        "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        {
          "border-transparent bg-slate-700 text-slate-200 hover:bg-slate-600":
            variant === "default",
          "border-transparent bg-slate-800 text-slate-400 hover:bg-slate-700":
            variant === "secondary",
          "border-transparent bg-red-500/20 text-red-400 border-red-500/30":
            variant === "destructive",
          "text-slate-300 border-slate-700": variant === "outline",
          "border-transparent bg-green-500/20 text-green-400 border-green-500/30":
            variant === "success",
        },
        className
      )}
      {...props}
    />
  )
}

export { Badge }
