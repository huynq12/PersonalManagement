import { useId, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useUpdateAccount } from '../../hooks/useSettings'
import Button from '../ui/Button'
import Card from '../ui/Card'
import Select from '../ui/Select'
import SaveStatus from './SaveStatus'

interface TimezoneSectionProps {
  /** The account's current IANA time zone. */
  timezone: string
}

/** Chooses the time zone that decides which calendar day it is for the user. */
export default function TimezoneSection({ timezone }: TimezoneSectionProps) {
  const { t } = useTranslation()
  const id = useId()
  const hintId = `${id}-hint`
  const [selected, setSelected] = useState(timezone)
  const save = useUpdateAccount()

  const zones = useMemo(() => {
    const listed = Intl.supportedValuesOf('timeZone')
    // The saved zone may be missing from the list of this browser; keep it selectable.
    return listed.includes(timezone) ? listed : [timezone, ...listed]
  }, [timezone])

  return (
    <Card title={t('settings.timezone.title')}>
      <form
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault()
          save.mutate({ timezone: selected })
        }}
      >
        <div className="flex flex-col gap-2">
          <label htmlFor={id} className="text-sm font-medium">
            {t('settings.timezone.label')}
          </label>
          <Select
            id={id}
            aria-describedby={hintId}
            className="h-10 max-w-sm"
            value={selected}
            onChange={(event) => setSelected(event.target.value)}
          >
            {zones.map((zone) => (
              <option key={zone} value={zone}>
                {zone}
              </option>
            ))}
          </Select>
          <p id={hintId} className="text-sm text-muted-foreground">
            {t('settings.timezone.hint')}
          </p>
        </div>

        <SaveStatus isError={save.isError} isSuccess={save.isSuccess} error={save.error} />

        <div>
          <Button type="submit" disabled={save.isPending || selected === timezone}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Card>
  )
}
