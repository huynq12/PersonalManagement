import type { QueryClient } from '@tanstack/react-query'
import { z } from 'zod'
import { SUPPORTED_LANGUAGES } from '../i18n/i18n'
import { API_BASE_URL, ApiError, apiRequest, setAccessToken, setUnauthorizedHandler } from './api'

export const userSchema = z.object({
  id: z.string(),
  email: z.string(),
  display_name: z.string(),
  timezone: z.string(),
  locale: z.enum(SUPPORTED_LANGUAGES),
})

export type User = z.infer<typeof userSchema>

const sessionSchema = z.object({
  access_token: z.string(),
  token_type: z.string(),
  expires_in: z.number(),
  user: userSchema,
})

/** Query key of the signed-in user: a `User`, or `null` when nobody is signed in. */
export const SESSION_QUERY_KEY = ['session'] as const

// Name of the cross-tab lock that serializes refresh requests.
const REFRESH_LOCK = 'auth.refresh'

export interface RegisterInput {
  email: string
  password: string
  display_name?: string
  locale: string
}

export interface LoginInput {
  email: string
  password: string
}

/** Creates an account. The backend does not sign the user in. */
export function register(input: RegisterInput): Promise<User> {
  return apiRequest('/auth/register', userSchema, {
    method: 'POST',
    body: input,
    refreshOnUnauthorized: false,
  })
}

/** Signs in and keeps the access token in memory; the refresh token arrives as a cookie. */
export async function login(input: LoginInput): Promise<User> {
  const session = await apiRequest('/auth/login', sessionSchema, {
    method: 'POST',
    body: input,
    refreshOnUnauthorized: false,
  })
  setAccessToken(session.access_token)
  return session.user
}

/** Revokes the refresh token on the server, then forgets the access token either way. */
export async function logout(): Promise<void> {
  try {
    await apiRequest('/auth/logout', z.null(), { method: 'POST', refreshOnUnauthorized: false })
  } finally {
    setAccessToken(null)
  }
}

let refreshInFlight: Promise<User | null> | null = null

/**
 * Exchanges the refresh cookie for a new access token. Resolves to the signed-in user, or to
 * `null` when there is no valid session.
 *
 * Every refresh rotates the cookie, and the backend treats a second use of the old cookie as
 * theft and revokes the whole session. So refreshes must never overlap: callers in this tab
 * share one request, and a Web Lock makes other tabs wait their turn.
 */
export function restoreSession(): Promise<User | null> {
  refreshInFlight ??= withRefreshLock(refresh).finally(() => {
    refreshInFlight = null
  })
  return refreshInFlight
}

async function refresh(): Promise<User | null> {
  try {
    const session = await apiRequest('/auth/refresh', sessionSchema, {
      method: 'POST',
      refreshOnUnauthorized: false,
    })
    setAccessToken(session.access_token)
    return session.user
  } catch (error) {
    if (error instanceof ApiError && error.status === 401) {
      setAccessToken(null)
      return null
    }
    throw error
  }
}

function withRefreshLock<T>(task: () => Promise<T>): Promise<T> {
  // Web Locks are missing in old browsers and in the test environment.
  return 'locks' in navigator ? navigator.locks.request(REFRESH_LOCK, task) : task()
}

/**
 * Makes API calls recover from an expired access token: the session is refreshed, the cached
 * user is updated (to `null` if the session is gone, which sends the user to the login page),
 * and the failed request is retried.
 */
export function installSessionRefresh(queryClient: QueryClient) {
  setUnauthorizedHandler(async () => {
    const user = await restoreSession().catch(() => null)
    queryClient.setQueryData(SESSION_QUERY_KEY, user)
    return user !== null
  })
}

/** URL that starts Google sign-in. It is a page navigation, not an API call. */
export function googleSignInUrl(locale: string): string {
  return `${API_BASE_URL}/auth/google?locale=${encodeURIComponent(locale)}`
}
