import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { SUPPORTED_LANGUAGES } from '../i18n/i18n'
import Select from './ui/Select'

export default function LanguageSwitcher() {
  const { t, i18n } = useTranslation()
  const selectId = useId()

  return (
    <div className="flex items-center justify-between gap-2 text-sm">
      <label htmlFor={selectId}>{t('language.label')}</label>
      <Select
        id={selectId}
        value={i18n.resolvedLanguage}
        onChange={(event) => void i18n.changeLanguage(event.target.value)}
      >
        {SUPPORTED_LANGUAGES.map((language) => (
          <option key={language} value={language}>
            {t(`language.${language}`)}
          </option>
        ))}
      </Select>
    </div>
  )
}
