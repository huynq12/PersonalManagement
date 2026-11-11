import type { ReactNode } from 'react'

type Variant = 'error' | 'success'

const VARIANT_CLASSES: Record<Variant, string> = {
  error: 'border-destructive-border bg-destructive-muted text-destructive',
  success: 'border-success-border bg-success-muted text-success',
}

interface AlertProps {
  variant: Variant
  children: ReactNode
}

/** A message box. Errors interrupt screen readers (`alert`); successes are announced politely. */
export default function Alert({ variant, children }: AlertProps) {
  return (
    <p
      role={variant === 'error' ? 'alert' : 'status'}
      className={`rounded-md border px-3 py-2 text-sm ${VARIANT_CLASSES[variant]}`}
    >
      {children}
    </p>
  )
}
