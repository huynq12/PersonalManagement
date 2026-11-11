import { cleanup, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import viLocale from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import { renderApp, signedIn, stubApi, USER } from '../test/testUtils'

beforeEach(async () => {
  await i18n.changeLanguage('en')
  stubApi(signedIn)
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

/** The desktop sidebar's navigation (the bottom bar has its own). */
async function sidebarNav() {
  const [sidebar] = await screen.findAllByRole('navigation', { name: 'Main navigation' })
  return within(sidebar)
}

describe('app shell', () => {
  it('lists every planned feature in the sidebar and marks the current page', async () => {
    renderApp('/')
    const nav = await sidebarNav()

    for (const name of [
      'Today',
      'Weekly stats',
      'Habits',
      'Pomodoro',
      'Subjects',
      'Weekly meal plan',
      'Dishes',
      'Shopping list',
    ]) {
      expect(nav.getByRole('link', { name })).toBeTruthy()
    }
    expect(nav.getByRole('link', { name: 'Today' }).getAttribute('aria-current')).toBe('page')
    expect(screen.getByText(USER.display_name)).toBeTruthy()
  })

  it('shows a placeholder for a feature that is not built yet', async () => {
    renderApp('/')
    const nav = await sidebarNav()

    await userEvent.click(nav.getByRole('link', { name: 'Habits' }))

    expect(await screen.findByRole('heading', { name: 'Habits' })).toBeTruthy()
    expect(screen.getByText('This feature is not built yet.')).toBeTruthy()
    expect(nav.getByRole('link', { name: 'Habits' }).getAttribute('aria-current')).toBe('page')
  })

  it('opens the More sheet with the remaining features and closes it with Escape', async () => {
    renderApp('/')
    await sidebarNav()

    await userEvent.click(screen.getByRole('button', { name: 'More' }))

    const sheet = screen.getByRole('dialog', { name: 'All features' })
    expect(within(sheet).getByRole('link', { name: 'Shopping list' })).toBeTruthy()
    expect(within(sheet).getByLabelText('Theme')).toBeTruthy()
    expect(document.activeElement).toBe(within(sheet).getByRole('button', { name: 'Close' }))

    await userEvent.keyboard('{Escape}')

    expect(screen.queryByRole('dialog')).toBeNull()
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'More' }))
  })

  it('closes the More sheet after choosing a feature', async () => {
    renderApp('/')
    await sidebarNav()
    await userEvent.click(screen.getByRole('button', { name: 'More' }))

    await userEvent.click(
      within(screen.getByRole('dialog')).getByRole('link', { name: 'Shopping list' }),
    )

    expect(await screen.findByRole('heading', { name: 'Shopping list' })).toBeTruthy()
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('switches the whole app to Vietnamese from the sidebar', async () => {
    renderApp('/')
    await screen.findByRole('heading', { name: 'Today' })
    const [languageSelect] = screen.getAllByLabelText('Language')

    await userEvent.selectOptions(languageSelect, 'vi')

    expect(await screen.findByRole('heading', { name: viLocale.home.title })).toBeTruthy()
    expect(document.documentElement.lang).toBe('vi')
    expect(document.title).toBe(viLocale.app.title)
  })

  it('renders the navigation in Vietnamese', async () => {
    await i18n.changeLanguage('vi')

    renderApp('/')

    const [sidebar] = await screen.findAllByRole('navigation', { name: viLocale.nav.label })
    expect(within(sidebar).getByRole('link', { name: viLocale.nav.mealPlan })).toBeTruthy()
    expect(screen.getByRole('button', { name: viLocale.nav.more })).toBeTruthy()
  })
})
