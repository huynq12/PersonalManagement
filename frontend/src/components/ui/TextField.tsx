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
    <div className="flex flex-col gap-2">
      <label htmlFor={id} className="text-sm font-medium">
        {label}
      </label>
      <input
        id={id}
        aria-invalid={error ? true : undefined}
        aria-describedby={message ? messageId : undefined}
        className="h-10 rounded-md border border-input bg-background px-3 text-sm text-foreground outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring aria-invalid:border-destructive"
        {...input}
      />
      {message ? (
        <p
          id={messageId}
          className={error ? 'text-sm text-destructive' : 'text-sm text-muted-foreground'}
        >
          {message}
        </p>
      ) : null}
    </div>
  )
}
