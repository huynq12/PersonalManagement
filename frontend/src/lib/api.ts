import { z } from 'zod'

// In production VITE_API_URL points at the backend; in development it is empty and the Vite
// proxy forwards /api to the local server.
const BASE_URL = `${import.meta.env.VITE_API_URL ?? ''}/api/v1`

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

/** Sends a GET request and validates the response body against a Zod schema. */
export async function apiGet<T>(path: string, schema: z.ZodType<T>): Promise<T> {
  let response: Response
  try {
    response = await fetch(`${BASE_URL}${path}`, { credentials: 'include' })
  } catch (cause) {
    // fetch only rejects when the request never completed (offline, DNS, CORS, server down).
    throw new ApiError(0, 'network', `request to ${path} failed: ${String(cause)}`)
  }

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

export const getHealth = () => apiGet('/health', healthSchema)
