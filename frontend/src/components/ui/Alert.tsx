import type { ReactNode } from 'react'

type Variant = 'error' | 'success'

const VARIANT_CLASSES: Record<Variant, string> = {
  error:
    'border-red-300 bg-red-50 text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-200',
  success:
    'border-green-300 bg-green-50 text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-200',
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
