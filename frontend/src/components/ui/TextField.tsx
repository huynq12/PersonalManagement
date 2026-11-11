import { useId, type ComponentProps } from 'react'

interface TextFieldProps extends ComponentProps<'input'> {
  label: string
  /** Already translated validation message. */
  error?: string
  /** Already translated help text, shown while there is no error. */
  hint?: string
}

/** A labelled input with its validation message wired up for screen readers. */
export default function TextField({ label, error, hint, ...input }: TextFieldProps) {
  const id = useId()
  const messageId = `${id}-message`
  const message = error ?? hint

  return (
    <div className="flex flex-col gap-1">
      <label htmlFor={id} className="text-sm font-medium">
        {label}
      </label>
      <input
        id={id}
        aria-invalid={error ? true : undefined}
        aria-describedby={message ? messageId : undefined}
        className="rounded-md border border-slate-300 bg-white px-3 py-2 text-sm shadow-xs outline-none focus:border-indigo-500 focus:ring-2 focus:ring-indigo-500/30 aria-invalid:border-red-500 dark:border-slate-700 dark:bg-slate-900"
        {...input}
      />
      {message ? (
        <p
          id={messageId}
          className={
            error
              ? 'text-sm text-red-700 dark:text-red-400'
              : 'text-sm text-slate-500 dark:text-slate-400'
          }
        >
          {message}
        </p>
      ) : null}
    </div>
  )
}
