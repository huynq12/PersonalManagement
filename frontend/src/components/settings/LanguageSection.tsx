import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { useUpdateAccount } from '../../hooks/useSettings'
import { SUPPORTED_LANGUAGES, type Language } from '../../i18n/i18n'
import Card from '../ui/Card'
import Select from '../ui/Select'
import SaveStatus from './SaveStatus'

/** Chooses the interface language. It applies at once and is also saved on the account. */
export default function LanguageSection() {
  const { t, i18n } = useTranslation()
  const id = useId()
  const hintId = `${id}-hint`
  const save = useUpdateAccount()

  const change = (language: Language) => {
    void i18n.changeLanguage(language)
    save.mutate({ locale: language })
  }

  return (
    <Card title={t('settings.language.title')}>
      <div className="flex flex-col gap-2">
        <label htmlFor={id} className="text-sm font-medium">
          {t('language.label')}
        </label>
        <Select
          id={id}
          aria-describedby={hintId}
          className="h-10 max-w-sm"
          value={i18n.resolvedLanguage}
          onChange={(event) => change(event.target.value as Language)}
        >
          {SUPPORTED_LANGUAGES.map((language) => (
            <option key={language} value={language}>
              {t(`language.${language}`)}
            </option>
          ))}
        </Select>
        <p id={hintId} className="text-sm text-muted-foreground">
          {t('settings.language.hint')}
        </p>
      </div>
      <SaveStatus isError={save.isError} isSuccess={save.isSuccess} error={save.error} />
    </Card>
  )
}
