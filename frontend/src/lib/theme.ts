/**
 * Light/dark theme preference: a tiny external store that React reads with
 * `useSyncExternalStore` (see `hooks/useTheme.ts`).
 *
 * The choice is kept in the browser only. "system" follows the operating system's color scheme,
 * including when it changes while the app is open.
 */

export const THEME_PREFERENCES = ['light', 'dark', 'system'] as const
export type ThemePreference = (typeof THEME_PREFERENCES)[number]
export type ResolvedTheme = 'light' | 'dark'

// Versioned like the language key so the stored value can change shape later.
export const THEME_STORAGE_KEY = 'app.theme.v1'

const DARK_QUERY = '(prefers-color-scheme: dark)'

function isThemePreference(value: unknown): value is ThemePreference {
  return THEME_PREFERENCES.includes(value as ThemePreference)
}

/** The saved preference, or "system" when nothing valid is saved or storage is blocked. */
export function readStoredTheme(): ThemePreference {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY)
    return isThemePreference(stored) ? stored : 'system'
  } catch {
    return 'system'
  }
}

function saveTheme(preference: ThemePreference) {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, preference)
  } catch {
    // Storage can be blocked (private mode, site settings); the choice then lasts for this tab.
  }
}

function systemPrefersDark(): boolean {
  // `matchMedia` is missing in some test environments.
  return typeof window.matchMedia === 'function' && window.matchMedia(DARK_QUERY).matches
}

export function resolveTheme(preference: ThemePreference, prefersDark: boolean): ResolvedTheme {
  if (preference === 'system') {
    return prefersDark ? 'dark' : 'light'
  }
  return preference
}

/** Puts the theme on `<html>`: the `dark` class drives Tailwind, `color-scheme` native controls. */
function applyTheme(preference: ThemePreference) {
  const resolved = resolveTheme(preference, systemPrefersDark())
  const root = document.documentElement
  root.classList.toggle('dark', resolved === 'dark')
  root.style.colorScheme = resolved
}

let current: ThemePreference = readStoredTheme()
const listeners = new Set<() => void>()

export function getThemePreference(): ThemePreference {
  return current
}

export function setThemePreference(preference: ThemePreference) {
  current = preference
  saveTheme(preference)
  applyTheme(preference)
  listeners.forEach((listener) => listener())
}

export function subscribeTheme(listener: () => void): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** Applies the saved theme and starts following system changes. Call once before rendering. */
export function initTheme() {
  current = readStoredTheme()
  applyTheme(current)

  if (typeof window.matchMedia === 'function') {
    window.matchMedia(DARK_QUERY).addEventListener('change', () => {
      if (current === 'system') {
        applyTheme(current)
      }
    })
  }
}
