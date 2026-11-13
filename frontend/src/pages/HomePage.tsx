import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router'
import DayModeControl from '../components/today/DayModeControl'
import DayNavigation from '../components/today/DayNavigation'
import HabitChecklist from '../components/today/HabitChecklist'
import TaskList from '../components/today/TaskList'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import { useDay } from '../hooks/useDay'
import { useSession } from '../hooks/useSession'
import { errorMessageKey } from '../i18n/errorMessage'
import { isIsoDate, todayIn } from '../lib/dates'

/**
 * The Today screen: the habits scheduled for a day and its to-dos.
 *
 * The day comes from the URL (`/?date=YYYY-MM-DD`); without it, or with an invalid value, the
 * screen shows today. The menu and the study time arrive with those features (SPEC.md, Today).
 */
export default function HomePage() {
  const { t } = useTranslation()
  const [searchParams] = useSearchParams()
  const session = useSession()

  // "Today" is the date in the user's time zone, not the browser's. Read the clock once per
  // visit so the page does not jump to another day while it is open.
  const timeZone = session.data?.timezone ?? Intl.DateTimeFormat().resolvedOptions().timeZone
  const [today] = useState(() => todayIn(timeZone))

  const requested = searchParams.get('date')
  const date = requested && isIsoDate(requested) ? requested : today
  const day = useDay(date)

  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-3">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t('home.title')}</h1>
        <DayNavigation date={date} today={today} />
        {day.isSuccess ? (
          <DayModeControl
            date={date}
            mode={day.data.mode}
            locked={day.data.mode_locked}
            isToday={date === today}
          />
        ) : null}
      </header>

      {day.isPending ? (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
      ) : day.isError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(day.error))}</Alert>
          <Button variant="secondary" onClick={() => void day.refetch()}>
            {t('common.retry')}
          </Button>
        </div>
      ) : (
        <div className="grid items-start gap-6 md:grid-cols-2">
          {/* ISO dates compare correctly as strings. */}
          <HabitChecklist
            date={date}
            habits={day.data.habits}
            mode={day.data.mode}
            isFuture={date > today}
          />
          <TaskList date={date} tasks={day.data.tasks} />
        </div>
      )}
    </main>
  )
}
