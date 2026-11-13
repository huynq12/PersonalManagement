import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { SESSION_QUERY_KEY, updateMe } from '../lib/auth'
import {
  getSettings,
  SETTINGS_QUERY_KEY,
  updateSettings,
  type SettingsChanges,
} from '../lib/settings'

/** The user's Pomodoro, work, and budget settings. */
export function useSettings() {
  return useQuery({ queryKey: SETTINGS_QUERY_KEY, queryFn: getSettings })
}

/** Saves any subset of the settings and puts the server's answer in the cache. */
export function useUpdateSettings() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (changes: SettingsChanges) => updateSettings(changes),
    onSuccess: (settings) => {
      queryClient.setQueryData(SETTINGS_QUERY_KEY, settings)
    },
  })
}

/** Changes the account currency; the signed-in user in the cache is replaced with the result. */
export function useUpdateCurrency() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (currency: string) => updateMe({ currency }),
    onSuccess: (user) => {
      queryClient.setQueryData(SESSION_QUERY_KEY, user)
    },
  })
}
