import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import DeleteWorkEntryDialog from '../components/work/DeleteWorkEntryDialog'
import MonthPicker from '../components/work/MonthPicker'
import WorkEntryFormCard from '../components/work/WorkEntryForm'
import WorkEntryList from '../components/work/WorkEntryList'
import WorkSetupNotice from '../components/work/WorkSetupNotice'
import { useSession } from '../hooks/useSession'
import { useSettings } from '../hooks/useSettings'
import { usePutWorkEntry, useWorkEntries } from '../hooks/useWorkEntries'
import { errorMessageKey } from '../i18n/errorMessage'
import { ApiError } from '../lib/api'
import { isIsoMonth, monthOf, todayIn, type IsoMonth } from '../lib/dates'
import { WORK_SETTINGS_MISSING, type WorkEntry } from '../lib/workEntries'

/**
 * The Work log screen: hours per day with the pay they earn, for one month at a time.
 *
 * The month comes from the URL (`/work-log?month=YYYY-MM`); without it, or with a value that
 * is invalid or in the future, the screen shows the current month.
 */
export default function WorkLogPage() {
  const { t } = useTranslation()
  const [searchParams, setSearchParams] = useSearchParams()
  const session = useSession()
  const settings = useSettings()
  const save = usePutWorkEntry()
  const [editing, setEditing] = useState<WorkEntry | null>(null)
  const [deleting, setDeleting] = useState<WorkEntry | null>(null)

  // "Today" is the date in the user's time zone, not the browser's. Read the clock once per
  // visit so the page does not jump to another day while it is open.
  const timeZone = session.data?.timezone ?? Intl.DateTimeFormat().resolvedOptions().timeZone
  const [today] = useState(() => todayIn(timeZone))
  const currentMonth = monthOf(today)

  const requested = searchParams.get('month')
  const month: IsoMonth =
    requested && isIsoMonth(requested) && requested <= currentMonth ? requested : currentMonth
  const entries = useWorkEntries(month)
  const currency = session.data?.currency

  const changeMonth = (next: IsoMonth) => {
    setEditing(null)
    setSearchParams(next === currentMonth ? {} : { month: next })
  }

  const canLogHours =
    settings.data !== undefined &&
    settings.data.hourly_wage !== null &&
    settings.data.base_work_hours !== null &&
    // Another tab may have cleared the settings after this page loaded them.
    !(save.error instanceof ApiError && save.error.code === WORK_SETTINGS_MISSING)

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t('workLog.title')}</h1>
        <p className="text-sm text-muted-foreground">{t('workLog.description')}</p>
      </header>

      {settings.isPending || !currency ? (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
      ) : settings.isError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(settings.error))}</Alert>
          <Button variant="secondary" onClick={() => void settings.refetch()}>
            {t('common.retry')}
          </Button>
        </div>
      ) : (
        <>
          {canLogHours ? (
            <WorkEntryFormCard
              // A different entry (or none) starts the form over with its own values.
              key={editing?.date ?? 'new'}
              today={today}
              editing={editing}
              save={save}
              onCancelEdit={() => setEditing(null)}
              onSaved={() => setEditing(null)}
            />
          ) : (
            <WorkSetupNotice />
          )}

          <MonthPicker month={month} latest={currentMonth} onChange={changeMonth} />

          {entries.isPending ? (
            <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
          ) : entries.isError ? (
            <div className="flex flex-col items-start gap-3">
              <Alert variant="error">{t(errorMessageKey(entries.error))}</Alert>
              <Button variant="secondary" onClick={() => void entries.refetch()}>
                {t('common.retry')}
              </Button>
            </div>
          ) : (
            <WorkEntryList
              month={entries.data}
              currency={currency}
              onEdit={canLogHours ? setEditing : undefined}
              onDelete={setDeleting}
            />
          )}
        </>
      )}

      {deleting ? (
        <DeleteWorkEntryDialog entry={deleting} onClose={() => setDeleting(null)} />
      ) : null}
    </main>
  )
}
