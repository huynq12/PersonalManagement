export type ButtonVariant = 'primary' | 'secondary' | 'ghost'

const BASE_CLASSES =
  'inline-flex h-10 items-center justify-center gap-2 rounded-md px-4 text-sm font-medium whitespace-nowrap transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:cursor-not-allowed disabled:opacity-60'

/** Classes shared by buttons and links that look like buttons. */
export const BUTTON_CLASSES: Record<ButtonVariant, string> = {
  primary: `${BASE_CLASSES} bg-primary text-primary-foreground hover:bg-primary/90`,
  secondary: `${BASE_CLASSES} border border-input bg-background text-foreground hover:bg-accent`,
  ghost: `${BASE_CLASSES} text-foreground hover:bg-accent`,
}
