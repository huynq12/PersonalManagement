import { z } from 'zod'

// In production VITE_API_URL points at the backend; in development it is empty and the Vite
// proxy forwards /api to the local server.
export const API_BASE_URL = `${import.meta.env.VITE_API_URL ?? ''}/api/v1`

const errorSchema = z.object({
  error: z.object({ code: z.string(), message: z.string() }),
})

/**
 * A failed API call. `code` is the stable identifier used to pick a translated message;
 * `message` is developer-facing English and must not be shown to users.
 */
export class ApiError extends Error {
  readonly status: number
  readonly code: string

  constructor(status: number, code: string, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.code = code
  }
}

// The access token lives only in memory, never in localStorage, so a script injected into the
// page cannot read it from storage. After a reload it is obtained again from the refresh
// cookie, which JavaScript cannot read at all.
let accessToken: string | null = null

export function setAccessToken(token: string | null) {
  accessToken = token
}

/** Tries to obtain a new access token; resolves to whether the request is worth retrying. */
type UnauthorizedHandler = () => Promise<boolean>

let unauthorizedHandler: UnauthorizedHandler | null = null

/**
 * Registers what to do when the API rejects the access token. Set by the auth module, which
 * keeps this file free of any dependency on it.
 */
export function setUnauthorizedHandler(handler: UnauthorizedHandler | null) {
  unauthorizedHandler = handler
}

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
  /** Sent as JSON. */
  body?: unknown
  /** Set to false for the auth endpoints themselves, which must not trigger a refresh. */
  refreshOnUnauthorized?: boolean
}

/**
 * Sends a request and validates the response body against a Zod schema.
 *
 * When the access token has expired the API answers 401 `unauthorized`; the session is then
 * refreshed and the request repeated once, so callers never see an expired token.
 */
export async function apiRequest<T>(
  path: string,
  schema: z.ZodType<T>,
  options: RequestOptions = {},
): Promise<T> {
  const { refreshOnUnauthorized = true } = options

  try {
    return await send(path, schema, options)
  } catch (error) {
    const expired = error instanceof ApiError && error.code === 'unauthorized'
    if (expired && refreshOnUnauthorized && unauthorizedHandler && (await unauthorizedHandler())) {
      return send(path, schema, options)
    }
    throw error
  }
}

async function send<T>(path: string, schema: z.ZodType<T>, options: RequestOptions): Promise<T> {
  const headers = new Headers()
  if (accessToken) {
    headers.set('Authorization', `Bearer ${accessToken}`)
  }
  if (options.body !== undefined) {
    headers.set('Content-Type', 'application/json')
  }

  let response: Response
  try {
    response = await fetch(`${API_BASE_URL}${path}`, {
      method: options.method ?? 'GET',
      headers,
      body: options.body === undefined ? undefined : JSON.stringify(options.body),
      // Sends and stores the refresh cookie.
      credentials: 'include',
    })
  } catch (cause) {
    // fetch only rejects when the request never completed (offline, DNS, CORS, server down).
    throw new ApiError(0, 'network', `request to ${path} failed: ${String(cause)}`)
  }

  // Responses without a body (204) parse to null.
  const body: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    const parsed = errorSchema.safeParse(body)
    throw parsed.success
      ? new ApiError(response.status, parsed.data.error.code, parsed.data.error.message)
      : new ApiError(response.status, 'unknown', `unexpected HTTP ${response.status} response`)
  }

  return schema.parse(body)
}

export const healthSchema = z.object({
  status: z.string(),
  database: z.string(),
})

export const getHealth = () => apiRequest('/health', healthSchema)
