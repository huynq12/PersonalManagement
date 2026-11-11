import { cleanup, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from './i18n/i18n'
import viLocale from './i18n/locales/vi.json'
import { setAccessToken } from './lib/api'
import {
  errorResponse,
  jsonResponse,
  renderApp,
  sessionResponse,
  signedIn,
  signedOut,
  stubApi,
  USER,
} from './test/testUtils'

const PASSWORD = 'correct horse battery'

async function fillLoginForm(email: string, password: string) {
  const user = userEvent.setup()
  await user.type(await screen.findByLabelText('Email'), email)
  await user.type(screen.getByLabelText('Password'), password)
  await user.click(screen.getByRole('button', { name: 'Sign in' }))
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('route protection', () => {
  it('sends a visitor without a session to the login page', async () => {
    stubApi(signedOut)

    renderApp('/')

    expect(await screen.findByRole('heading', { name: 'Sign in' })).toBeTruthy()
    expect(screen.queryByText('API and database are running.')).toBeNull()
  })

  it('restores the session from the refresh cookie and shows the home page', async () => {
    stubApi(signedIn)

    renderApp('/')

    expect(await screen.findByText('API and database are running.')).toBeTruthy()
    expect(screen.getByText(USER.display_name)).toBeTruthy()
  })

  it('restores the session with a single refresh request', async () => {
    const requests = stubApi(signedIn)

    renderApp('/')
    await screen.findByText(USER.display_name)

    expect(requests.filter((request) => request.path === '/auth/refresh')).toHaveLength(1)
  })

  it('sends a signed-in user away from the login page', async () => {
    stubApi(signedIn)

    renderApp('/login')

    expect(await screen.findByText(USER.display_name)).toBeTruthy()
    expect(screen.queryByRole('heading', { name: 'Sign in' })).toBeNull()
  })

  it('offers a retry when the session cannot be checked', async () => {
    stubApi({
      'POST /auth/refresh': () => errorResponse(503, 'service_unavailable'),
    })

    renderApp('/')

    expect(await screen.findByRole('alert')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy()
  })
})

describe('login', () => {
  it('signs in with email and password and shows the home page', async () => {
    const requests = stubApi({
      ...signedOut,
      'POST /auth/login': () => sessionResponse(),
    })
    renderApp('/login')

    await fillLoginForm(USER.email, PASSWORD)

    expect(await screen.findByText(USER.display_name)).toBeTruthy()
    const login = requests.find((request) => request.path === '/auth/login')
    expect(login?.body).toEqual({ email: USER.email, password: PASSWORD })
  })

  it('translates a rejected login by its error code', async () => {
    stubApi({
      ...signedOut,
      'POST /auth/login': () => errorResponse(401, 'invalid_credentials'),
    })
    renderApp('/login')

    await fillLoginForm(USER.email, 'wrong password')

    expect((await screen.findByRole('alert')).textContent).toBe('Incorrect email or password.')
    expect(screen.queryByText(/developer message/)).toBeNull()
  })

  it('validates the fields before sending anything', async () => {
    const requests = stubApi(signedOut)
    renderApp('/login')

    await fillLoginForm('not-an-email', ' ')

    expect(await screen.findByText('Enter a valid email address.')).toBeTruthy()
    expect(requests.some((request) => request.path === '/auth/login')).toBe(false)
  })

  it('shows the error code Google sign-in redirected back with', async () => {
    stubApi(signedOut)

    renderApp('/login?error=email_already_registered')

    expect((await screen.findByRole('alert')).textContent).toContain(
      'already registered with a password',
    )
  })

  it('shows a generic message for an unknown error code in the URL', async () => {
    stubApi(signedOut)

    renderApp('/login?error=%3Cscript%3E')

    expect((await screen.findByRole('alert')).textContent).toBe(
      'Something went wrong. Please try again.',
    )
  })

  it('links to Google sign-in with the active language', async () => {
    stubApi(signedOut)
    await i18n.changeLanguage('vi')

    renderApp('/login')

    const link = await screen.findByRole('link', { name: viLocale.auth.google })
    expect(link.getAttribute('href')).toBe('/api/v1/auth/google?locale=vi')
  })

  it('renders in Vietnamese', async () => {
    stubApi(signedOut)
    await i18n.changeLanguage('vi')

    renderApp('/login')

    expect(await screen.findByRole('heading', { name: viLocale.auth.login.title })).toBeTruthy()
    expect(screen.getByLabelText(viLocale.auth.fields.password)).toBeTruthy()
  })
})

describe('register', () => {
  async function fillRegisterForm(password: string) {
    const user = userEvent.setup()
    await user.type(await screen.findByLabelText('Email'), USER.email)
    await user.type(screen.getByLabelText('Display name (optional)'), USER.display_name)
    await user.type(screen.getByLabelText('Password'), password)
    await user.click(screen.getByRole('button', { name: 'Create account' }))
  }

  it('creates the account and sends the user to the login page with a notice', async () => {
    const requests = stubApi({
      ...signedOut,
      'POST /auth/register': () => jsonResponse(201, USER),
    })
    renderApp('/register')

    await fillRegisterForm(PASSWORD)

    expect(await screen.findByRole('heading', { name: 'Sign in' })).toBeTruthy()
    expect(screen.getByRole('status').textContent).toBe('Account created. You can sign in now.')
    expect((screen.getByLabelText('Email') as HTMLInputElement).value).toBe(USER.email)
    const register = requests.find((request) => request.path === '/auth/register')
    expect(register?.body).toEqual({
      email: USER.email,
      password: PASSWORD,
      display_name: USER.display_name,
      locale: 'en',
    })
  })

  it('omits the display name when it is left empty', async () => {
    const requests = stubApi({
      ...signedOut,
      'POST /auth/register': () => jsonResponse(201, USER),
    })
    renderApp('/register')
    const user = userEvent.setup()

    await user.type(await screen.findByLabelText('Email'), USER.email)
    await user.type(screen.getByLabelText('Password'), PASSWORD)
    await user.click(screen.getByRole('button', { name: 'Create account' }))

    await screen.findByRole('heading', { name: 'Sign in' })
    const register = requests.find((request) => request.path === '/auth/register')
    expect(register?.body).toEqual({ email: USER.email, password: PASSWORD, locale: 'en' })
  })

  it('rejects a password shorter than 8 characters without sending anything', async () => {
    const requests = stubApi(signedOut)
    renderApp('/register')

    await fillRegisterForm('1234567')

    expect(await screen.findByText('Use 8 to 128 characters.')).toBeTruthy()
    expect(requests.some((request) => request.path === '/auth/register')).toBe(false)
  })

  it('translates the error when the email is already taken', async () => {
    stubApi({
      ...signedOut,
      'POST /auth/register': () => errorResponse(409, 'email_taken'),
    })
    renderApp('/register')

    await fillRegisterForm(PASSWORD)

    expect((await screen.findByRole('alert')).textContent).toBe('This email is already registered.')
  })
})

describe('google callback', () => {
  it('finishes sign-in by exchanging the refresh cookie and shows the home page', async () => {
    stubApi(signedIn)

    renderApp('/auth/callback')

    expect(await screen.findByText(USER.display_name)).toBeTruthy()
  })

  it('returns to the login page with an error when no session was created', async () => {
    stubApi(signedOut)

    renderApp('/auth/callback')

    expect((await screen.findByRole('alert')).textContent).toBe(
      'Google sign-in could not be completed. Please try again.',
    )
    expect(screen.getByRole('heading', { name: 'Sign in' })).toBeTruthy()
  })
})

describe('logout', () => {
  it('revokes the session and returns to the login page', async () => {
    const requests = stubApi({
      ...signedIn,
      'POST /auth/logout': () => new Response(null, { status: 204 }),
    })
    renderApp('/')

    await userEvent.click(await screen.findByRole('button', { name: 'Sign out' }))

    expect(await screen.findByRole('heading', { name: 'Sign in' })).toBeTruthy()
    expect(requests.some((request) => request.path === '/auth/logout')).toBe(true)
  })

  it('signs out locally even when the server cannot be reached', async () => {
    stubApi({
      ...signedIn,
      'POST /auth/logout': () => errorResponse(503, 'service_unavailable'),
    })
    renderApp('/')

    await userEvent.click(await screen.findByRole('button', { name: 'Sign out' }))

    await waitFor(() => expect(screen.getByRole('heading', { name: 'Sign in' })).toBeTruthy())
  })
})
