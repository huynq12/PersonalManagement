import { cleanup, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import vi_ from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import type { Settings } from '../lib/settings'
import {
  errorResponse,
  jsonResponse,
  renderApp,
  sessionResponse,
  signedIn,
  stubApi,
  USER,
  type Handler,
  type RecordedRequest,
} from '../test/testUtils'

const DEFAULT_SETTINGS: Settings = {
  focus_minutes: 25,
  short_break_minutes: 5,
  long_break_minutes: 15,
  long_break_every: 4,
  hourly_wage: null,
  base_work_hours: null,
  monthly_budget: null,
}

let settings: Settings
let user: typeof USER & { currency: string }
let requests: RecordedRequest[]

function renderSettings(path = '/settings', extraHandlers: Record<string, Handler> = {}) {
  requests = stubApi({
    ...signedIn,
    'POST /auth/refresh': () =>
      jsonResponse(200, {
        access_token: 'access-token-1',
        token_type: 'Bearer',
        expires_in: 900,
        user,
      }),
    'GET /settings': () => jsonResponse(200, settings),
    'PATCH /settings': (request) => {
      settings = { ...settings, ...(request.body as Partial<Settings>) }
      return jsonResponse(200, settings)
    },
    'PATCH /me': (request) => {
      user = { ...user, ...(request.body as { currency: string }) }
      return jsonResponse(200, user)
    },
    ...extraHandlers,
  })
  renderApp(path)
}

function patchesTo(path: string) {
  return requests.filter((request) => request.method === 'PATCH' && request.path === path)
}

/** The Card with the given title; every settings section is a named region. */
async function section(name: string) {
  return within(await screen.findByRole('region', { name }))
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  settings = { ...DEFAULT_SETTINGS }
  user = { ...USER, currency: 'VND' }
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Settings screen', () => {
  it('shows the saved settings in the units a person types', async () => {
    settings = {
      ...DEFAULT_SETTINGS,
      hourly_wage: 30000,
      base_work_hours: 7.5,
      monthly_budget: 5000000,
      focus_minutes: 50,
    }
    renderSettings()

    const work = await section('Work')
    expect((work.getByLabelText('Hourly wage (VND)') as HTMLInputElement).value).toBe('30000')
    expect((work.getByLabelText('Base hours per day') as HTMLInputElement).value).toBe('7.5')
    const budget = await section('Monthly budget')
    expect((budget.getByLabelText('Monthly expense budget (VND)') as HTMLInputElement).value).toBe(
      '5000000',
    )
    const pomodoro = await section('Pomodoro')
    expect((pomodoro.getByLabelText('Focus (minutes)') as HTMLInputElement).value).toBe('50')
  })

  it('shows an error with a retry button when the settings cannot be loaded', async () => {
    let failing = true
    vi.stubGlobal('fetch', async (input: string) => {
      const path = new URL(input, 'http://localhost').pathname.replace(/^\/api\/v1/, '')
      if (path === '/auth/refresh') {
        return sessionResponse()
      }
      return failing ? errorResponse(500, 'internal_error') : jsonResponse(200, DEFAULT_SETTINGS)
    })
    renderApp('/settings')

    expect((await screen.findByRole('alert')).textContent).toBe(
      'The server ran into a problem. Please try again later.',
    )
    failing = false
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }))

    expect(await screen.findByRole('region', { name: 'Work' })).toBeTruthy()
  })

  it('saves the work settings in minor units', async () => {
    renderSettings()
    const work = await section('Work')

    await userEvent.type(work.getByLabelText('Hourly wage (VND)'), '30000')
    await userEvent.type(work.getByLabelText('Base hours per day'), '7.5')
    await userEvent.click(work.getByRole('button', { name: 'Save' }))

    expect(await work.findByText('Saved.')).toBeTruthy()
    expect(patchesTo('/settings').map((request) => request.body)).toEqual([
      { hourly_wage: 30000, base_work_hours: 7.5 },
    ])
  })

  it('converts typed amounts using the fraction digits of the account currency', async () => {
    user = { ...user, currency: 'USD' }
    renderSettings()
    const work = await section('Work')

    await userEvent.type(work.getByLabelText('Hourly wage (USD)'), '12.5')
    await userEvent.click(work.getByRole('button', { name: 'Save' }))

    expect(await work.findByText('Saved.')).toBeTruthy()
    expect(patchesTo('/settings')[0].body).toEqual({
      hourly_wage: 1250,
      base_work_hours: null,
    })
    expect((work.getByLabelText('Hourly wage (USD)') as HTMLInputElement).value).toBe('12.5')
  })

  it('rejects more decimals than the currency has without calling the server', async () => {
    renderSettings()
    const work = await section('Work')

    await userEvent.type(work.getByLabelText('Hourly wage (VND)'), '100.5')
    await userEvent.click(work.getByRole('button', { name: 'Save' }))

    expect(await work.findByText(/Enter an amount greater than 0/)).toBeTruthy()
    expect(patchesTo('/settings')).toHaveLength(0)
  })

  it('rejects base hours above 24', async () => {
    renderSettings()
    const work = await section('Work')

    await userEvent.type(work.getByLabelText('Base hours per day'), '25')
    await userEvent.click(work.getByRole('button', { name: 'Save' }))

    expect(await work.findByText(/Enter hours greater than 0/)).toBeTruthy()
    expect(patchesTo('/settings')).toHaveLength(0)
  })

  it('clears a setting when its field is emptied', async () => {
    settings = { ...DEFAULT_SETTINGS, monthly_budget: 5000000 }
    renderSettings()
    const budget = await section('Monthly budget')

    await userEvent.clear(budget.getByLabelText('Monthly expense budget (VND)'))
    await userEvent.click(budget.getByRole('button', { name: 'Save' }))

    expect(await budget.findByText('Saved.')).toBeTruthy()
    expect(patchesTo('/settings')[0].body).toEqual({ monthly_budget: null })
  })

  it('saves the Pomodoro lengths as numbers and rejects values out of range', async () => {
    renderSettings()
    const pomodoro = await section('Pomodoro')
    const focus = pomodoro.getByLabelText('Focus (minutes)')

    await userEvent.clear(focus)
    await userEvent.type(focus, '300')
    await userEvent.click(pomodoro.getByRole('button', { name: 'Save' }))
    expect(await pomodoro.findByText('Enter a whole number from 1 to 240.')).toBeTruthy()
    expect(patchesTo('/settings')).toHaveLength(0)

    await userEvent.clear(focus)
    await userEvent.type(focus, '50')
    await userEvent.click(pomodoro.getByRole('button', { name: 'Save' }))

    expect(await pomodoro.findByText('Saved.')).toBeTruthy()
    expect(patchesTo('/settings')[0].body).toEqual({
      focus_minutes: 50,
      short_break_minutes: 5,
      long_break_minutes: 15,
      long_break_every: 4,
    })
  })

  it('changes the currency, warns that amounts are not converted, and relabels the fields', async () => {
    settings = { ...DEFAULT_SETTINGS, hourly_wage: 1250 }
    renderSettings()
    const currency = await section('Currency')
    expect(currency.getByText(/Existing amounts are not converted/)).toBeTruthy()
    expect((currency.getByRole('button', { name: 'Save' }) as HTMLButtonElement).disabled).toBe(
      true,
    )

    await userEvent.selectOptions(currency.getByLabelText('Currency'), 'USD')
    await userEvent.click(currency.getByRole('button', { name: 'Save' }))

    expect(await currency.findByText('Saved.')).toBeTruthy()
    expect(patchesTo('/me').map((request) => request.body)).toEqual([{ currency: 'USD' }])
    const work = await section('Work')
    // The stored 1250 is shown as 12.50 once the account currency has two decimals.
    expect(((await work.findByLabelText('Hourly wage (USD)')) as HTMLInputElement).value).toBe(
      '12.50',
    )
  })

  it('shows an error when saving fails', async () => {
    renderSettings()
    const work = await section('Work')
    vi.stubGlobal('fetch', async () => errorResponse(422, 'validation_failed'))

    await userEvent.type(work.getByLabelText('Hourly wage (VND)'), '100')
    await userEvent.click(work.getByRole('button', { name: 'Save' }))

    expect((await work.findByRole('alert')).textContent).toBe(
      'Some fields are not valid. Please check them and try again.',
    )
  })

  it('renders in Vietnamese', async () => {
    await i18n.changeLanguage('vi')
    renderSettings()

    expect(
      await screen.findByRole('heading', {
        name: vi_.settings.title,
        level: 1,
      }),
    ).toBeTruthy()
    expect(await screen.findByRole('region', { name: vi_.settings.work.title })).toBeTruthy()
  })

  describe('Google linking', () => {
    it('starts linking and sends the browser to the Google URL', async () => {
      const assign = vi.fn()
      vi.stubGlobal('location', { ...window.location, assign })
      renderSettings('/settings', {
        'POST /auth/google/link': () =>
          jsonResponse(200, {
            authorization_url: 'https://accounts.example/auth?x=1',
          }),
      })
      const google = await section('Google account')

      await userEvent.click(google.getByRole('button', { name: 'Link Google' }))

      await vi.waitFor(() =>
        expect(assign).toHaveBeenCalledWith('https://accounts.example/auth?x=1'),
      )
    })

    it('shows an error and stays on the page when linking cannot start', async () => {
      const assign = vi.fn()
      vi.stubGlobal('location', { ...window.location, assign })
      renderSettings('/settings', {
        'POST /auth/google/link': () => errorResponse(404, 'not_found'),
      })
      const google = await section('Google account')

      await userEvent.click(google.getByRole('button', { name: 'Link Google' }))

      expect((await google.findByRole('alert')).textContent).toBe(
        'The requested item was not found.',
      )
      expect(assign).not.toHaveBeenCalled()
    })

    it('confirms a successful link after Google sends the user back', async () => {
      renderSettings('/settings?google=linked')
      const google = await section('Google account')

      expect((await google.findByRole('status')).textContent).toBe('Your Google account is linked.')
    })

    it('translates the error code Google linking redirects back with', async () => {
      renderSettings('/settings?error=google_account_already_linked')
      const google = await section('Google account')

      expect((await google.findByRole('alert')).textContent).toBe(
        'This Google account is linked to another account, or your account already has a Google account linked.',
      )
    })

    it('shows the Vietnamese text from the locale file', async () => {
      await i18n.changeLanguage('vi')
      renderSettings('/settings?google=linked')

      const google = await section(vi_.settings.google.title)
      expect((await google.findByRole('status')).textContent).toBe(vi_.settings.google.linked)
    })
  })
})
