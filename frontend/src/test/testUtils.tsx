import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { vi } from 'vitest'
import App from '../App'
import { installSessionRefresh } from '../lib/auth'

export function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

export function errorResponse(status: number, code: string): Response {
  return jsonResponse(status, { error: { code, message: `developer message for ${code}` } })
}

/** A request the app sent to the stubbed API. */
export interface RecordedRequest {
  method: string
  path: string
  body: unknown
  authorization: string | null
}

type Handler = (request: RecordedRequest) => Response

/**
 * Replaces `fetch` with a router keyed by `"METHOD /path"` (the path is relative to `/api/v1`).
 * Routes that are not listed answer 404. Returns the list of requests received so far.
 */
export function stubApi(routes: Record<string, Handler>): RecordedRequest[] {
  const requests: RecordedRequest[] = []

  vi.stubGlobal('fetch', async (input: string, init?: RequestInit) => {
    const url = new URL(input, 'http://localhost')
    const headers = new Headers(init?.headers)
    const request: RecordedRequest = {
      method: init?.method ?? 'GET',
      path: url.pathname.replace(/^\/api\/v1/, '') + url.search,
      body: typeof init?.body === 'string' ? JSON.parse(init.body) : null,
      authorization: headers.get('Authorization'),
    }
    requests.push(request)

    const handler = routes[`${request.method} ${request.path}`]
    return handler ? handler(request) : errorResponse(404, 'not_found')
  })

  return requests
}

export const USER = {
  id: '6f1c2d0e-0000-4000-8000-000000000001',
  email: 'ada@example.com',
  display_name: 'Ada Lovelace',
  timezone: 'Asia/Ho_Chi_Minh',
  locale: 'en',
}

export function sessionResponse(accessToken = 'access-token-1'): Response {
  return jsonResponse(200, {
    access_token: accessToken,
    token_type: 'Bearer',
    expires_in: 900,
    user: USER,
  })
}

/** Common routes: a healthy backend. Spread into `stubApi` and add the routes under test. */
export const healthyBackend: Record<string, Handler> = {
  'GET /health': () => jsonResponse(200, { status: 'ok', database: 'ok' }),
}

/** A visitor with no session: the refresh cookie is missing or invalid. */
export const signedOut: Record<string, Handler> = {
  ...healthyBackend,
  'POST /auth/refresh': () => errorResponse(401, 'invalid_refresh_token'),
}

/** A visitor whose refresh cookie is valid. */
export const signedIn: Record<string, Handler> = {
  ...healthyBackend,
  'POST /auth/refresh': () => sessionResponse(),
}

/** Renders the whole app at `path` with a fresh query cache. */
export function renderApp(path: string) {
  // Retries are disabled so a failing request surfaces immediately.
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  installSessionRefresh(queryClient)

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[path]}>
        <App />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}
