import type { LucideIcon } from 'lucide-react'
import type { ComponentProps } from 'react'

interface IconButtonProps extends Omit<ComponentProps<'button'>, 'children'> {
  icon: LucideIcon
  /** What the button does. Read by screen readers and shown as a tooltip. */
  label: string
}

/** A square button that shows only an icon. The label keeps it usable without seeing the icon. */
export default function IconButton({
  icon: Icon,
  label,
  className = '',
  type = 'button',
  ...props
}: IconButtonProps) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={`flex size-9 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:cursor-not-allowed disabled:opacity-60 ${className}`}
      {...props}
    >
      <Icon className="size-4" aria-hidden="true" />
    </button>
  )
}
