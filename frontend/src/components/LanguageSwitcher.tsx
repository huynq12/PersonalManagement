import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { SUPPORTED_LANGUAGES } from '../i18n/i18n'

export default function LanguageSwitcher() {
  const { t, i18n } = useTranslation()
  const selectId = useId()

  return (
    <div className="language-switcher">
      <label htmlFor={selectId}>{t('language.label')}</label>
      <select
        id={selectId}
        value={i18n.resolvedLanguage}
        onChange={(event) => void i18n.changeLanguage(event.target.value)}
      >
        {SUPPORTED_LANGUAGES.map((language) => (
          <option key={language} value={language}>
            {t(`language.${language}`)}
          </option>
        ))}
      </select>
    </div>
  )
}
