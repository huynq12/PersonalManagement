import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import DeleteWorkEntryDialog from '../components/work/DeleteWorkEntryDialog'
import MonthPicker from '../components/work/MonthPicker'
import WorkCalendar from '../components/work/WorkCalendar'
import WorkEntryDialog from '../components/work/WorkEntryDialog'
import WorkLegend from '../components/work/WorkLegend'
import WorkSettingsCard from '../components/work/WorkSettingsCard'
import WorkSetupNotice from '../components/work/WorkSetupNotice'
import WorkSummary from '../components/work/WorkSummary'
import { useSession } from '../hooks/useSession'
import { useSettings } from '../hooks/useSettings'
import { usePutWorkEntry, useWorkEntries } from '../hooks/useWorkEntries'
import { errorMessageKey } from '../i18n/errorMessage'
import { ApiError } from '../lib/api'
import { isIsoMonth, monthOf, todayIn, type IsoDate, type IsoMonth } from '../lib/dates'
import { WORK_SETTINGS_MISSING, type WorkEntry } from '../lib/workEntries'

/** The day whose dialog is open, with its entry when there is one. */
interface SelectedDay {
  date: IsoDate
  entry: WorkEntry | null
}

/**
 * The Work log screen: a calendar of the month's days with the hours and pay of each, a summary
 * of the month, and the wage and base hours the pay is computed from.
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
  const [selected, setSelected] = useState<SelectedDay | null>(null)
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
    setSelected(null)
    setSearchParams(next === currentMonth ? {} : { month: next })
  }

  const canLogHours =
    settings.data !== undefined &&
    settings.data.hourly_wage !== null &&
    settings.data.base_work_hours !== null &&
    // Another tab may have cleared the settings after this page loaded them.
    !(save.error instanceof ApiError && save.error.code === WORK_SETTINGS_MISSING)

  const selectDay = (date: IsoDate, entry: WorkEntry | null) => {
    if (!canLogHours) {
      // Without a wage there is nothing to log yet: take the user to the card that sets it.
      document.getElementById('work-settings')?.scrollIntoView?.({ behavior: 'smooth' })
      return
    }
    // An error from the last save belongs to that day, not to this one.
    save.reset()
    setSelected({ date, entry })
  }

  return (
    <main className="mx-auto flex w-full max-w-4xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
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
          {canLogHours ? null : <WorkSetupNotice />}

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
            <>
              <WorkSummary month={entries.data} currency={currency} />
              <section aria-label={t('workLog.calendar.section')} className="flex flex-col gap-3">
                <WorkLegend />
                <WorkCalendar
                  month={entries.data}
                  monthKey={month}
                  today={today}
                  currency={currency}
                  onSelectDay={selectDay}
                />
              </section>
            </>
          )}

          <WorkSettingsCard
            // A new currency changes how the saved wage is written, so start the form over.
            key={`work-${currency}`}
            settings={settings.data}
            currency={currency}
          />
        </>
      )}

      {selected && canLogHours && currency ? (
        <WorkEntryDialog
          // A different day starts the form over with its own values.
          key={selected.date}
          date={selected.date}
          today={today}
          entry={selected.entry}
          currency={currency}
          save={save}
          onClose={() => setSelected(null)}
          onDelete={(entry) => {
            setSelected(null)
            setDeleting(entry)
          }}
        />
      ) : null}
      {deleting ? (
        <DeleteWorkEntryDialog entry={deleting} onClose={() => setDeleting(null)} />
      ) : null}
    </main>
  )
}
