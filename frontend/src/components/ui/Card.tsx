import { useId, type ReactNode } from 'react'

interface CardProps {
  title: string
  /** Shown next to the title, such as a progress count. */
  aside?: ReactNode
  children: ReactNode
}

/** A titled panel. It is a landmark region named by its title, so it can be found by name. */
export default function Card({ title, aside, children }: CardProps) {
  const titleId = useId()

  return (
    <section
      aria-labelledby={titleId}
      className="flex flex-col gap-3 rounded-xl border border-border p-5"
    >
      <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
        <h2 id={titleId} className="text-base font-semibold">
          {title}
        </h2>
        {aside ? <p className="text-sm text-muted-foreground">{aside}</p> : null}
      </div>
      {children}
    </section>
  )
}
