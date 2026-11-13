import { z } from 'zod'
import { apiRequest } from './api'
import type { IsoDate, IsoMonth } from './dates'

export const comparisonSchema = z.enum(['above', 'equal', 'below'])

export type Comparison = z.infer<typeof comparisonSchema>

export const workEntrySchema = z.object({
  date: z.string(),
  hours: z.number(),
  /** Pay per hour in minor units, as copied when the entry was created. */
  hourly_wage: z.number(),
  base_hours: z.number(),
  /** `hours × hourly_wage` in minor units, computed by the server. */
  pay: z.number(),
  comparison: comparisonSchema,
  note: z.string().nullable(),
})

export type WorkEntry = z.infer<typeof workEntrySchema>

export const workMonthSchema = z.object({
  entries: z.array(workEntrySchema),
  summary: z.object({
    total_hours: z.number(),
    total_pay: z.number(),
    days_worked: z.number(),
  }),
})

export type WorkMonth = z.infer<typeof workMonthSchema>

/** Prefix of every work log query, to invalidate all months at once. */
export const WORK_ENTRIES_QUERY_KEY = ['work-entries'] as const

export const workMonthQueryKey = (month: IsoMonth) => [...WORK_ENTRIES_QUERY_KEY, month] as const

/** The API error code for a new entry when the wage or base hours are not set. */
export const WORK_SETTINGS_MISSING = 'work_settings_missing'

/** Body of a response with no content (204). */
const noContent = z.null()

export const getWorkMonth = (month: IsoMonth) =>
  apiRequest(`/work-entries?month=${month}`, workMonthSchema)

/** Creates or updates the entry of a day. A `null` note clears it. */
export const putWorkEntry = (date: IsoDate, input: { hours: number; note: string | null }) =>
  apiRequest(`/work-entries/${date}`, workEntrySchema, { method: 'PUT', body: input })

export const deleteWorkEntry = (date: IsoDate) =>
  apiRequest(`/work-entries/${date}`, noContent, { method: 'DELETE' })

/** Formats hours with the active language's decimal separator and at most two decimals. */
export function formatHours(hours: number, language: string): string {
  return new Intl.NumberFormat(language, { maximumFractionDigits: 2 }).format(hours)
}
