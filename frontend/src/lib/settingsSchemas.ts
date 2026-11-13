import { z } from 'zod'
import type { ValidationMessageKey } from './authSchemas'
import { MAX_MONEY, parseMoneyInput } from './money'

// Validation messages are translation keys; the forms pass them through `t()`.
const required: ValidationMessageKey = 'validation.required'
const minutesRange: ValidationMessageKey = 'validation.minutesRange'
const longBreakEveryRange: ValidationMessageKey = 'validation.longBreakEveryRange'
const moneyInvalid: ValidationMessageKey = 'validation.moneyInvalid'
const hoursInvalid: ValidationMessageKey = 'validation.hoursInvalid'

// Mirrors the backend rules: whole numbers, 1 to 240 minutes and 1 to 12 sessions.
// An empty number input reads as NaN, which Zod reports as the wrong type.
const minutes = z
  .number({ error: required })
  .int(minutesRange)
  .min(1, minutesRange)
  .max(240, minutesRange)

export const pomodoroFormSchema = z.object({
  focus_minutes: minutes,
  short_break_minutes: minutes,
  long_break_minutes: minutes,
  long_break_every: z
    .number({ error: required })
    .int(longBreakEveryRange)
    .min(1, longBreakEveryRange)
    .max(12, longBreakEveryRange),
})

export type PomodoroForm = z.infer<typeof pomodoroFormSchema>

/**
 * Converts typed money to minor units for the backend: `null` for empty text (clear the value),
 * `undefined` when the text is not an amount from 1 to the backend's maximum.
 */
export function parseOptionalAmount(text: string, currency: string): number | null | undefined {
  if (text.trim() === '') {
    return null
  }
  const amount = parseMoneyInput(text, currency)
  return amount !== null && amount >= 1 && amount <= MAX_MONEY ? amount : undefined
}

/**
 * Converts typed hours to a number for the backend: `null` for empty text (clear the value),
 * `undefined` when the text is not above 0 and at most 24 with at most two decimals.
 */
export function parseOptionalHours(text: string): number | null | undefined {
  if (text.trim() === '') {
    return null
  }
  if (!/^\d+(?:[.,]\d{1,2})?$/.test(text.trim())) {
    return undefined
  }
  const hours = Number(text.trim().replace(',', '.'))
  return hours > 0 && hours <= 24 ? hours : undefined
}

/** The work settings form: both fields are text, and an empty one clears the setting. */
export const workFormSchema = (currency: string) =>
  z.object({
    hourly_wage: z
      .string()
      .refine((text) => parseOptionalAmount(text, currency) !== undefined, moneyInvalid),
    base_work_hours: z
      .string()
      .refine((text) => parseOptionalHours(text) !== undefined, hoursInvalid),
  })

export type WorkForm = z.infer<ReturnType<typeof workFormSchema>>

/** The monthly budget form: an empty field removes the budget. */
export const budgetFormSchema = (currency: string) =>
  z.object({
    monthly_budget: z
      .string()
      .refine((text) => parseOptionalAmount(text, currency) !== undefined, moneyInvalid),
  })

export type BudgetForm = z.infer<ReturnType<typeof budgetFormSchema>>
