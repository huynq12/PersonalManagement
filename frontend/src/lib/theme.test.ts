import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  getThemePreference,
  initTheme,
  readStoredTheme,
  resolveTheme,
  setThemePreference,
  subscribeTheme,
  THEME_STORAGE_KEY,
} from './theme'

/** Stubs `matchMedia` so the system reports the given color scheme. */
function stubSystemTheme(prefersDark: boolean) {
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: prefersDark,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }))
}

function isDark() {
  return document.documentElement.classList.contains('dark')
}

describe('theme', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.classList.remove('dark')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('resolves the system preference to the color scheme the system reports', () => {
    expect(resolveTheme('system', true)).toBe('dark')
    expect(resolveTheme('system', false)).toBe('light')
    expect(resolveTheme('light', true)).toBe('light')
    expect(resolveTheme('dark', false)).toBe('dark')
  })

  it('falls back to the system preference when nothing valid is stored', () => {
    expect(readStoredTheme()).toBe('system')

    localStorage.setItem(THEME_STORAGE_KEY, 'purple')

    expect(readStoredTheme()).toBe('system')
  })

  it('applies and remembers an explicit choice', () => {
    stubSystemTheme(false)

    setThemePreference('dark')

    expect(isDark()).toBe(true)
    expect(document.documentElement.style.colorScheme).toBe('dark')
    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe('dark')
    expect(getThemePreference()).toBe('dark')

    setThemePreference('light')

    expect(isDark()).toBe(false)
    expect(readStoredTheme()).toBe('light')
  })

  it('follows the system when the preference is system', () => {
    stubSystemTheme(true)

    setThemePreference('system')

    expect(isDark()).toBe(true)
  })

  it('applies the stored choice on start-up', () => {
    stubSystemTheme(false)
    localStorage.setItem(THEME_STORAGE_KEY, 'dark')

    initTheme()

    expect(isDark()).toBe(true)
    expect(getThemePreference()).toBe('dark')
  })

  it('works when matchMedia is not available', () => {
    vi.stubGlobal('matchMedia', undefined)

    setThemePreference('system')

    expect(isDark()).toBe(false)
  })

  it('notifies subscribers when the preference changes', () => {
    stubSystemTheme(false)
    const listener = vi.fn()
    const unsubscribe = subscribeTheme(listener)

    setThemePreference('dark')
    unsubscribe()
    setThemePreference('light')

    expect(listener).toHaveBeenCalledTimes(1)
  })
})
