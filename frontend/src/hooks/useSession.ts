import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { login, logout, register, restoreSession, SESSION_QUERY_KEY } from '../lib/auth'

/**
 * The signed-in user: `data` is a `User`, or `null` when nobody is signed in.
 *
 * The first use restores the session from the refresh cookie. After that the cached value only
 * changes through login, logout, or a failed token refresh, so it never goes stale by itself.
 */
export function useSession() {
  return useQuery({
    queryKey: SESSION_QUERY_KEY,
    queryFn: restoreSession,
    staleTime: Infinity,
    gcTime: Infinity,
    retry: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  })
}

export function useLogin() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: login,
    onSuccess: (user) => {
      queryClient.setQueryData(SESSION_QUERY_KEY, user)
    },
  })
}

export function useRegister() {
  return useMutation({ mutationFn: register })
}

export function useLogout() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: logout,
    // Sign out locally even if the server could not be reached.
    onSettled: () => {
      queryClient.setQueryData(SESSION_QUERY_KEY, null)
      // Drop everything cached for the previous user.
      queryClient.removeQueries({ predicate: (query) => query.queryKey[0] !== SESSION_QUERY_KEY[0] })
    },
  })
}
