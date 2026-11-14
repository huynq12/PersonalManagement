import { useTranslation } from 'react-i18next'
import { formatDuration } from '../../lib/duration'
import type { WeekTotals } from '../../lib/stats'

interface SummaryCardsProps {
  totals: WeekTotals
  previous: WeekTotals
}

/**
 * How this week compares with the week before, as text: "more", "less", "the same", or that
 * there is nothing to compare with. `format` writes the size of the difference.
 */
function comparisonText(
  t: ReturnType<typeof useTranslation>['t'],
  difference: number | null,
  format: (size: number) => string,
): string {
  if (difference === null) {
    return t('stats.comparison.none')
  }
  if (difference === 0) {
    return t('stats.comparison.same')
  }
  return t(difference > 0 ? 'stats.comparison.up' : 'stats.comparison.down', {
    value: format(Math.abs(difference)),
  })
}

interface FigureProps {
  label: string
  value: string
  comparison: string
}

function Figure({ label, value, comparison }: FigureProps) {
  return (
    <div className="flex flex-col gap-1 rounded-xl border border-border p-5">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="text-2xl font-semibold tabular-nums">{value}</dd>
      <dd className="text-xs text-muted-foreground">{comparison}</dd>
    </div>
  )
}

/** Study time, Pomodoros, and checklist completion of the week, each against the week before. */
export default function SummaryCards({ totals, previous }: SummaryCardsProps) {
  const { t, i18n } = useTranslation()
  const percent = new Intl.NumberFormat(i18n.language, {
    style: 'percent',
    maximumFractionDigits: 0,
  })
  const number = new Intl.NumberFormat(i18n.language)

  // Percentage points between two rates; unknown when either week had nothing scheduled.
  const rateDifference =
    totals.completion_rate === null || previous.completion_rate === null
      ? null
      : Math.round((totals.completion_rate - previous.completion_rate) * 100)

  return (
    <dl className="grid gap-4 sm:grid-cols-3">
      <Figure
        label={t('stats.summary.studyTime')}
        value={formatDuration(totals.study_seconds, t)}
        comparison={comparisonText(
          t,
          // A week without any study is a real zero, so there is always something to compare.
          totals.study_seconds - previous.study_seconds,
          (size) => formatDuration(size, t),
        )}
      />
      <Figure
        label={t('stats.summary.pomodoros')}
        value={number.format(totals.pomodoros)}
        comparison={comparisonText(t, totals.pomodoros - previous.pomodoros, (size) =>
          number.format(size),
        )}
      />
      <Figure
        label={t('stats.summary.checklist')}
        value={
          totals.completion_rate === null
            ? t('stats.summary.noData')
            : percent.format(totals.completion_rate)
        }
        comparison={comparisonText(t, rateDifference, (size) => t('stats.points', { count: size }))}
      />
    </dl>
  )
}
