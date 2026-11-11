import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import viLocale from '../i18n/locales/vi.json'
import HomePage from './HomePage'

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

function renderHomePage() {
  // Retries are disabled so a failing request surfaces immediately.
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })

  return render(
    <QueryClientProvider client={queryClient}>
      <HomePage />
    </QueryClientProvider>,
  )
}

describe('HomePage', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('en')
  })

  afterEach(() => {
    cleanup()
    vi.unstubAllGlobals()
  })

  it('shows the healthy status in English', async () => {
    vi.stubGlobal('fetch', async () => jsonResponse(200, { status: 'ok', database: 'ok' }))

    renderHomePage()

    expect(await screen.findByText('API and database are running.')).toBeTruthy()
  })

  it('shows the healthy status in Vietnamese', async () => {
    vi.stubGlobal('fetch', async () => jsonResponse(200, { status: 'ok', database: 'ok' }))
    await i18n.changeLanguage('vi')

    renderHomePage()

    expect(await screen.findByText(viLocale.home.healthy)).toBeTruthy()
  })

  it('translates an API error by its code instead of showing the backend message', async () => {
    vi.stubGlobal('fetch', async () =>
      jsonResponse(503, {
        error: { code: 'service_unavailable', message: 'service temporarily unavailable' },
      }),
    )
    await i18n.changeLanguage('vi')

    renderHomePage()

    expect(await screen.findByText(viLocale.errors.service_unavailable, { exact: false })).toBeTruthy()
    expect(screen.queryByText(/service temporarily unavailable/)).toBeNull()
  })

  it('shows a network error message when the request cannot be sent', async () => {
    vi.stubGlobal('fetch', async () => {
      throw new TypeError('Failed to fetch')
    })

    renderHomePage()

    expect(await screen.findByText(/Could not reach the server/)).toBeTruthy()
  })

  it('shows the date in the active language', async () => {
    vi.stubGlobal('fetch', async () => jsonResponse(200, { status: 'ok', database: 'ok' }))
    await i18n.changeLanguage('vi')
    const expected = new Intl.DateTimeFormat('vi', { dateStyle: 'full' }).format(new Date())

    renderHomePage()

    expect(await screen.findByRole('heading', { name: viLocale.home.title })).toBeTruthy()
    expect(screen.getByText(expected)).toBeTruthy()
  })
})
