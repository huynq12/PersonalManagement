import type { ComponentProps } from 'react'

/** A native select styled like the other form controls. */
export default function Select({ className = '', ...props }: ComponentProps<'select'>) {
  return (
    <select
      className={`h-9 rounded-md border border-input bg-background px-2 text-sm text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring ${className}`}
      {...props}
    />
  )
}
