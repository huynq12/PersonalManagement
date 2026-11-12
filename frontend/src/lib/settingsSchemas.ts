import { z } from 'zod'
import { SUPPORTED_LANGUAGES } from '../i18n/i18n'
import type { ValidationMessageKey } from './authSchemas'

// Validation messages are translation keys; the forms pass them through `t()`.
const required: ValidationMessageKey = 'validation.required'
const displayNameLength: ValidationMessageKey = 'validation.displayNameLength'

// Mirrors the backend rules: a display name of 1 to 100 characters.
export const profileFormSchema = z.object({
  displayName: z.string().trim().min(1, required).max(100, displayNameLength),
  timezone: z.string().min(1, required),
  locale: z.enum(SUPPORTED_LANGUAGES),
})

export type ProfileForm = z.infer<typeof profileFormSchema>

/** Allowed range of each Pomodoro setting; mirrors the backend rules. */
export const POMODORO_LIMITS = {
  focus_minutes: { min: 1, max: 180, message: 'validation.focusMinutesRange' },
  short_break_minutes: {
    min: 1,
    max: 60,
    message: 'validation.shortBreakMinutesRange',
  },
  long_break_minutes: {
    min: 1,
    max: 120,
    message: 'validation.longBreakMinutesRange',
  },
  long_break_every: {
    min: 1,
    max: 12,
    message: 'validation.longBreakEveryRange',
  },
} as const satisfies Record<string, { min: number; max: number; message: ValidationMessageKey }>

type PomodoroField = keyof typeof POMODORO_LIMITS

// The inputs are registered with `valueAsNumber`, so an empty input arrives as NaN.
const wholeNumber = (field: PomodoroField) => {
  const { min, max, message } = POMODORO_LIMITS[field]
  return z.number(message).int(message).min(min, message).max(max, message)
}

export const pomodoroFormSchema = z.object({
  focus_minutes: wholeNumber('focus_minutes'),
  short_break_minutes: wholeNumber('short_break_minutes'),
  long_break_minutes: wholeNumber('long_break_minutes'),
  long_break_every: wholeNumber('long_break_every'),
})

export type PomodoroForm = z.infer<typeof pomodoroFormSchema>
