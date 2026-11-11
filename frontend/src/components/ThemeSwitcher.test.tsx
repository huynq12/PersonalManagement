import { cleanup, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import i18n from '../i18n/i18n'
import viLocale from '../i18n/locales/vi.json'
import { setThemePreference } from '../lib/theme'
import ThemeSwitcher from './ThemeSwitcher'

describe('ThemeSwitcher', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('en')
    setThemePreference('light')
  })

  afterEach(() => {
    cleanup()
  })

  it('switches the whole document to the dark theme', async () => {
    render(<ThemeSwitcher />)

    await userEvent.selectOptions(screen.getByLabelText('Theme'), 'dark')

    expect(document.documentElement.classList.contains('dark')).toBe(true)
    expect((screen.getByLabelText('Theme') as HTMLSelectElement).value).toBe('dark')
  })

  it('keeps several switchers in sync', async () => {
    render(
      <>
        <ThemeSwitcher />
        <ThemeSwitcher />
      </>,
    )
    const [first, second] = screen.getAllByLabelText('Theme') as HTMLSelectElement[]

    await userEvent.selectOptions(first, 'dark')

    expect(second.value).toBe('dark')
  })

  it('labels the options in Vietnamese', async () => {
    await i18n.changeLanguage('vi')

    render(<ThemeSwitcher />)

    expect(screen.getByLabelText(viLocale.theme.label)).toBeTruthy()
    expect(screen.getByRole('option', { name: viLocale.theme.system })).toBeTruthy()
  })
})
