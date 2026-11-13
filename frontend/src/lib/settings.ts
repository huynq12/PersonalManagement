import { z } from 'zod'
import { apiRequest } from './api'

export const settingsSchema = z.object({
  focus_minutes: z.number(),
  short_break_minutes: z.number(),
  long_break_minutes: z.number(),
  long_break_every: z.number(),
  /** Pay per hour in minor units, or null until set. */
  hourly_wage: z.number().nullable(),
  /** Usual hours per working day, or null until set. */
  base_work_hours: z.number().nullable(),
  /** Monthly expense budget in minor units, or null for no budget. */
  monthly_budget: z.number().nullable(),
})

export type Settings = z.infer<typeof settingsSchema>

/** Query key of the user's settings. */
export const SETTINGS_QUERY_KEY = ['settings'] as const

/** Any subset of the settings. `null` clears a nullable one; the Pomodoro fields cannot be null. */
export type SettingsChanges = Partial<Settings>

export const getSettings = () => apiRequest('/settings', settingsSchema)

export const updateSettings = (changes: SettingsChanges) =>
  apiRequest('/settings', settingsSchema, { method: 'PATCH', body: changes })
