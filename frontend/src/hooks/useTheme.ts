import { useSyncExternalStore } from 'react'
import {
  getThemePreference,
  setThemePreference,
  subscribeTheme,
  type ThemePreference,
} from '../lib/theme'

/** The current theme preference and a setter; every caller stays in sync. */
export function useTheme(): [ThemePreference, (preference: ThemePreference) => void] {
  const preference = useSyncExternalStore(subscribeTheme, getThemePreference)
  return [preference, setThemePreference]
}
