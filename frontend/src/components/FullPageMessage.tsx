import type { ReactNode } from 'react'

/** A short message centered on an otherwise empty page, for loading and blocking states. */
export default function FullPageMessage({ children }: { children: ReactNode }) {
  return (
    <main className="grid min-h-dvh place-items-center p-4">
      <div className="flex max-w-sm flex-col items-center gap-3 text-center">{children}</div>
    </main>
  )
}
