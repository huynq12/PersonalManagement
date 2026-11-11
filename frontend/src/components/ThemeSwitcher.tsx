import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { useTheme } from '../hooks/useTheme'
import { THEME_PREFERENCES, type ThemePreference } from '../lib/theme'
import Select from './ui/Select'

export default function ThemeSwitcher() {
  const { t } = useTranslation()
  const [preference, setPreference] = useTheme()
  const selectId = useId()

  return (
    <div className="flex items-center justify-between gap-2 text-sm">
      <label htmlFor={selectId}>{t('theme.label')}</label>
      <Select
        id={selectId}
        value={preference}
        onChange={(event) => setPreference(event.target.value as ThemePreference)}
      >
        {THEME_PREFERENCES.map((option) => (
          <option key={option} value={option}>
            {t(`theme.${option}`)}
          </option>
        ))}
      </Select>
    </div>
  )
}
