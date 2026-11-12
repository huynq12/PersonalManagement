import { z } from 'zod'
import { apiRequest } from './api'
import { userSchema, type User } from './auth'

export const pomodoroSettingsSchema = z.object({
  focus_minutes: z.number(),
  short_break_minutes: z.number(),
  long_break_minutes: z.number(),
  long_break_every: z.number(),
})

export type PomodoroSettings = z.infer<typeof pomodoroSettingsSchema>

/** Query key of the Pomodoro settings. */
export const SETTINGS_QUERY_KEY = ['settings'] as const

export const getSettings = () => apiRequest('/settings', pomodoroSettingsSchema)

/** Replaces every Pomodoro setting. */
export const replaceSettings = (settings: PomodoroSettings) =>
  apiRequest('/settings', pomodoroSettingsSchema, {
    method: 'PUT',
    body: settings,
  })

/** Fields of the profile a user can change; fields left out keep their value. */
export interface ProfileChanges {
  display_name?: string
  timezone?: string
  locale?: User['locale']
}

export const updateProfile = (changes: ProfileChanges) =>
  apiRequest('/me', userSchema, { method: 'PATCH', body: changes })

const googleLinkSchema = z.object({ authorization_url: z.string() })

/** Starts linking Google; resolves to the Google URL the browser must navigate to. */
export async function startGoogleLink(): Promise<string> {
  const { authorization_url } = await apiRequest('/auth/google/link', googleLinkSchema, {
    method: 'POST',
  })
  return authorization_url
}
