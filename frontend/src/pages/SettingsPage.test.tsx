import { cleanup, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import enLocale from '../i18n/locales/en.json'
import viLocale from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import {
  errorResponse,
  jsonResponse,
  renderApp,
  signedIn,
  stubApi,
  USER,
  type RecordedRequest,
} from '../test/testUtils'

const SETTINGS = {
  focus_minutes: 25,
  short_break_minutes: 5,
  long_break_minutes: 15,
  long_break_every: 4,
}

let requests: RecordedRequest[]

function renderSettings(path = '/settings', routes = {}) {
  requests = stubApi({
    ...signedIn,
    'GET /settings': () => jsonResponse(200, SETTINGS),
    ...routes,
  })
  renderApp(path)
}

const sent = (method: string, path: string) =>
  requests.filter((request) => request.method === method && request.path === path)

async function section(name: string) {
  return within(await screen.findByRole('region', { name }))
}

/** The Pomodoro card, once its settings have loaded and the form has replaced the placeholder. */
async function loadedPomodoro() {
  await screen.findByLabelText('Focus length (minutes)')
  return section('Pomodoro')
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Settings screen: profile', () => {
  it('shows the current profile', async () => {
    renderSettings()

    const profile = await section('Profile')
    await waitFor(() =>
      expect((profile.getByLabelText('Display name') as HTMLInputElement).value).toBe(
        USER.display_name,
      ),
    )
    expect(profile.getByText(USER.email)).toBeTruthy()
    expect((profile.getByLabelText('Time zone') as HTMLSelectElement).value).toBe(USER.timezone)
    expect((profile.getByLabelText('Language') as HTMLSelectElement).value).toBe('en')
  })

  it('saves the profile and switches the interface language', async () => {
    renderSettings('/settings', {
      'PATCH /me': () =>
        jsonResponse(200, {
          ...USER,
          display_name: 'Ada L.',
          timezone: 'Europe/London',
          locale: 'vi',
        }),
    })

    const profile = await section('Profile')
    const name = await profile.findByLabelText('Display name')
    await waitFor(() => expect((name as HTMLInputElement).value).toBe(USER.display_name))
    await userEvent.clear(name)
    await userEvent.type(name, 'Ada L.')
    await userEvent.selectOptions(profile.getByLabelText('Time zone'), 'Europe/London')
    await userEvent.selectOptions(profile.getByLabelText('Language'), 'vi')
    await userEvent.click(profile.getByRole('button', { name: 'Save' }))

    expect(await screen.findByText(viLocale.settings.profile.saved)).toBeTruthy()
    expect(sent('PATCH', '/me')[0].body).toEqual({
      display_name: 'Ada L.',
      timezone: 'Europe/London',
      locale: 'vi',
    })
    expect(i18n.resolvedLanguage).toBe('vi')
  })

  it('rejects a blank display name without calling the API', async () => {
    renderSettings()

    const profile = await section('Profile')
    const name = await profile.findByLabelText('Display name')
    await waitFor(() => expect((name as HTMLInputElement).value).toBe(USER.display_name))
    await userEvent.clear(name)
    await userEvent.click(profile.getByRole('button', { name: 'Save' }))

    expect(await profile.findByText(enLocale.validation.required)).toBeTruthy()
    expect(sent('PATCH', '/me')).toHaveLength(0)
  })

  it('shows a translated message when saving fails', async () => {
    renderSettings('/settings', {
      'PATCH /me': () => errorResponse(422, 'validation_failed'),
    })

    const profile = await section('Profile')
    await waitFor(() =>
      expect((profile.getByLabelText('Display name') as HTMLInputElement).value).toBe(
        USER.display_name,
      ),
    )
    await userEvent.click(profile.getByRole('button', { name: 'Save' }))

    expect(await profile.findByRole('alert')).toHaveProperty(
      'textContent',
      enLocale.errors.validation_failed,
    )
  })
})

describe('Settings screen: Pomodoro', () => {
  it('shows the stored values', async () => {
    renderSettings()

    const pomodoro = await loadedPomodoro()
    await waitFor(() =>
      expect((pomodoro.getByLabelText('Focus length (minutes)') as HTMLInputElement).value).toBe(
        '25',
      ),
    )
    expect((pomodoro.getByLabelText('Short break (minutes)') as HTMLInputElement).value).toBe('5')
    expect((pomodoro.getByLabelText('Long break (minutes)') as HTMLInputElement).value).toBe('15')
    expect(
      (pomodoro.getByLabelText('Focus sessions before a long break') as HTMLInputElement).value,
    ).toBe('4')
  })

  it('saves every value as a number', async () => {
    renderSettings('/settings', {
      'PUT /settings': () => jsonResponse(200, { ...SETTINGS, focus_minutes: 50 }),
    })

    const pomodoro = await loadedPomodoro()
    const focus = await pomodoro.findByLabelText('Focus length (minutes)')
    await waitFor(() => expect((focus as HTMLInputElement).value).toBe('25'))
    await userEvent.clear(focus)
    await userEvent.type(focus, '50')
    await userEvent.click(pomodoro.getByRole('button', { name: 'Save' }))

    expect(await pomodoro.findByText(enLocale.settings.pomodoro.saved)).toBeTruthy()
    expect(sent('PUT', '/settings')[0].body).toEqual({
      ...SETTINGS,
      focus_minutes: 50,
    })
  })

  it('rejects a value outside the allowed range without calling the API', async () => {
    renderSettings()

    const pomodoro = await loadedPomodoro()
    const focus = await pomodoro.findByLabelText('Focus length (minutes)')
    await waitFor(() => expect((focus as HTMLInputElement).value).toBe('25'))
    await userEvent.clear(focus)
    await userEvent.type(focus, '181')
    await userEvent.click(pomodoro.getByRole('button', { name: 'Save' }))

    expect(await pomodoro.findByText(enLocale.validation.focusMinutesRange)).toBeTruthy()
    expect(sent('PUT', '/settings')).toHaveLength(0)
  })

  it('rejects an empty value', async () => {
    renderSettings()

    const pomodoro = await loadedPomodoro()
    const every = await pomodoro.findByLabelText('Focus sessions before a long break')
    await waitFor(() => expect((every as HTMLInputElement).value).toBe('4'))
    await userEvent.clear(every)
    await userEvent.click(pomodoro.getByRole('button', { name: 'Save' }))

    expect(await pomodoro.findByText(enLocale.validation.longBreakEveryRange)).toBeTruthy()
    expect(sent('PUT', '/settings')).toHaveLength(0)
  })
})

describe('Settings screen: Google and sign out', () => {
  it('starts linking Google and sends the browser to the returned URL', async () => {
    const assign = vi.fn()
    vi.stubGlobal('location', { ...window.location, assign })
    renderSettings('/settings', {
      'POST /auth/google/link': () =>
        jsonResponse(200, {
          authorization_url: 'https://accounts.google.com/auth?x=1',
        }),
    })

    const google = await section('Google')
    await userEvent.click(google.getByRole('button', { name: 'Link Google account' }))

    await waitFor(() => expect(assign).toHaveBeenCalledWith('https://accounts.google.com/auth?x=1'))
  })

  it('confirms a successful link', async () => {
    renderSettings('/settings?google=linked')

    expect(await screen.findByText(enLocale.settings.google.linked)).toBeTruthy()
  })

  it('translates the error code Google linking redirects back with', async () => {
    renderSettings('/settings?error=google_account_already_linked')

    expect(await screen.findByText(enLocale.errors.google_account_already_linked)).toBeTruthy()
  })

  it('treats an unknown error code as a generic failure', async () => {
    renderSettings('/settings?error=<script>')

    expect(await screen.findByText(enLocale.errors.unknown)).toBeTruthy()
  })

  it('signs out', async () => {
    renderSettings('/settings', {
      'POST /auth/logout': () => new Response(null, { status: 204 }),
    })

    const account = await section('Account')
    await userEvent.click(account.getByRole('button', { name: 'Sign out' }))

    await waitFor(() => expect(sent('POST', '/auth/logout')).toHaveLength(1))
  })
})
