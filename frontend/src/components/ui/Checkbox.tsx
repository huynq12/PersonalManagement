import type { ComponentProps } from 'react'

/** A native checkbox styled like the other form controls. Wrap it in a `<label>` to name it. */
export default function Checkbox({ className = '', ...props }: ComponentProps<'input'>) {
  return (
    <input
      type="checkbox"
      className={`size-5 shrink-0 cursor-pointer rounded border-input accent-primary focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:cursor-not-allowed disabled:opacity-60 ${className}`}
      {...props}
    />
  )
}
