import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { SESSION_QUERY_KEY } from '../lib/auth'
import {
  getSettings,
  replaceSettings,
  SETTINGS_QUERY_KEY,
  startGoogleLink,
  updateProfile,
} from '../lib/settings'

/** The user's Pomodoro settings. */
export function useSettings() {
  return useQuery({ queryKey: SETTINGS_QUERY_KEY, queryFn: getSettings })
}

export function useReplaceSettings() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: replaceSettings,
    onSuccess: (settings) => {
      queryClient.setQueryData(SETTINGS_QUERY_KEY, settings)
    },
  })
}

/** Saves the profile and keeps the cached signed-in user in step with it. */
export function useUpdateProfile() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: updateProfile,
    onSuccess: (user) => {
      queryClient.setQueryData(SESSION_QUERY_KEY, user)
    },
  })
}

/** Asks the backend for the Google URL, then sends the browser there. */
export function useLinkGoogle() {
  return useMutation({
    mutationFn: startGoogleLink,
    onSuccess: (url) => window.location.assign(url),
  })
}
