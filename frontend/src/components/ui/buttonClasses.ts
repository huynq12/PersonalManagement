export type ButtonVariant = 'primary' | 'secondary'

const BASE_CLASSES =
  'inline-flex items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500 disabled:cursor-not-allowed disabled:opacity-60'

/** Classes shared by buttons and links that look like buttons. */
export const BUTTON_CLASSES: Record<ButtonVariant, string> = {
  primary: `${BASE_CLASSES} bg-indigo-600 text-white hover:bg-indigo-500 dark:bg-indigo-500 dark:hover:bg-indigo-400`,
  secondary: `${BASE_CLASSES} border border-slate-300 bg-white text-slate-900 hover:bg-slate-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100 dark:hover:bg-slate-800`,
}
