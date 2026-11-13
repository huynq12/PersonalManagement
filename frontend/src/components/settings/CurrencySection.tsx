import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useUpdateCurrency } from '../../hooks/useSettings'
import { SUPPORTED_CURRENCIES } from '../../lib/money'
import Button from '../ui/Button'
import Card from '../ui/Card'
import Select from '../ui/Select'
import SaveStatus from './SaveStatus'

interface CurrencySectionProps {
  /** The account's current currency. */
  currency: string
}

/** Chooses the currency amounts are entered and shown in. Existing amounts are not converted. */
export default function CurrencySection({ currency }: CurrencySectionProps) {
  const { t, i18n } = useTranslation()
  const id = useId()
  const hintId = `${id}-hint`
  const [selected, setSelected] = useState(currency)
  const save = useUpdateCurrency()

  const names = new Intl.DisplayNames(i18n.resolvedLanguage, { type: 'currency' })

  return (
    <Card title={t('settings.currency.title')}>
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault()
          save.mutate(selected)
        }}
      >
        <div className="flex flex-col gap-2">
          <label htmlFor={id} className="text-sm font-medium">
            {t('settings.currency.label')}
          </label>
          <Select
            id={id}
            aria-describedby={hintId}
            className="h-10 max-w-sm"
            value={selected}
            onChange={(event) => setSelected(event.target.value)}
          >
            {SUPPORTED_CURRENCIES.map((code) => (
              <option key={code} value={code}>
                {`${code} — ${names.of(code) ?? code}`}
              </option>
            ))}
          </Select>
          <p id={hintId} className="text-sm text-muted-foreground">
            {t('settings.currency.warning')}
          </p>
        </div>

        <SaveStatus isError={save.isError} isSuccess={save.isSuccess} error={save.error} />

        <div>
          <Button type="submit" disabled={save.isPending || selected === currency}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Card>
  )
}
