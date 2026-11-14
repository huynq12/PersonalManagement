import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router'
import DayBarChart from '../components/stats/DayBarChart'
import DaysTable from '../components/stats/DaysTable'
import HabitStreakList from '../components/stats/HabitStreakList'
import SubjectBreakdown from '../components/stats/SubjectBreakdown'
import SummaryCards from '../components/stats/SummaryCards'
import WeekPicker from '../components/stats/WeekPicker'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import Card from '../components/ui/Card'
import { useSession } from '../hooks/useSession'
import { useWeeklyStats } from '../hooks/useWeeklyStats'
import { errorMessageKey } from '../i18n/errorMessage'
import {
  isIsoDate,
  isoWeekday,
  mondayOf,
  todayIn,
  weekdayNames,
  type IsoDate,
} from '../lib/dates'
import { formatDuration } from '../lib/duration'
import type { WeeklyStats } from '../lib/stats'

/** The week's figures as chart points, one per day, Monday first. */
function dayPoints(stats: WeeklyStats, names: string[], value: (day: WeeklyStats['days'][number]) => number) {
  return stats.days.map((day) => ({
    label: names[isoWeekday(day.date) - 1],
    value: value(day),
    lowEnergy: day.mode === 'low_energy',
  }))
}

/**
 * The Weekly statistics screen: study time, checklist completion, low-energy days, and habit
 * streaks for one week, compared with the week before.
 *
 * The week comes from the URL (`/stats?week=YYYY-MM-DD`, a Monday); without it, or with a value
 * that is invalid, not a Monday, or in the future, the screen shows the current week.
 */
export default function StatsPage() {
  const { t, i18n } = useTranslation()
  const [searchParams, setSearchParams] = useSearchParams()
  const session = useSession()

  // "Today" is the date in the user's time zone, not the browser's. Read the clock once per
  // visit so the page does not jump to another week while it is open.
  const timeZone = session.data?.timezone ?? Intl.DateTimeFormat().resolvedOptions().timeZone
  const [today] = useState(() => todayIn(timeZone))
  const currentWeek = mondayOf(today)

  const requested = searchParams.get('week')
  const weekStart: IsoDate =
    requested &&
    isIsoDate(requested) &&
    mondayOf(requested) === requested &&
    requested <= currentWeek
      ? requested
      : currentWeek
  const stats = useWeeklyStats(weekStart)

  const changeWeek = (next: IsoDate) => setSearchParams(next === currentWeek ? {} : { week: next })
  const names = weekdayNames(i18n.resolvedLanguage)
  const percent = new Intl.NumberFormat(i18n.language, { style: 'percent', maximumFractionDigits: 0 })

  return (
    <main className="mx-auto flex w-full max-w-4xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t('stats.title')}</h1>
        <p className="text-sm text-muted-foreground">{t('stats.description')}</p>
      </header>

      <WeekPicker weekStart={weekStart} latest={currentWeek} onChange={changeWeek} />

      {stats.isPending ? (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
      ) : stats.isError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(stats.error))}</Alert>
          <Button variant="secondary" onClick={() => void stats.refetch()}>
            {t('common.retry')}
          </Button>
        </div>
      ) : (
        <>
          <SummaryCards totals={stats.data.totals} previous={stats.data.previous_totals} />

          <Card title={t('stats.studyPerDay.title')}>
            <DayBarChart
              label={t('stats.studyPerDay.chart')}
              // Minutes keep the axis readable; the tooltip writes the full duration.
              points={dayPoints(stats.data, names, (day) => Math.round(day.study_seconds / 60))}
              formatValue={(minutes) => formatDuration(minutes * 60, t)}
            />
          </Card>

          <SubjectBreakdown subjects={stats.data.subjects} />

          <Card title={t('stats.rate.title')}>
            <DayBarChart
              label={t('stats.rate.chart')}
              points={dayPoints(stats.data, names, (day) =>
                Math.round((day.completion_rate ?? 0) * 100),
              )}
              formatValue={(value) => percent.format(value / 100)}
              max={100}
            />
          </Card>

          <DaysTable days={stats.data.days} />
          <HabitStreakList habits={stats.data.habits} />
        </>
      )}
    </main>
  )
}
