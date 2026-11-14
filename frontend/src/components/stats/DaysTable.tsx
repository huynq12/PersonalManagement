import { useTranslation } from 'react-i18next'
import { formatShortDate } from '../../lib/dates'
import { formatDuration } from '../../lib/duration'
import type { DayStats } from '../../lib/stats'
import Card from '../ui/Card'

interface DaysTableProps {
  days: DayStats[]
}

/** The week day by day, with the mode of each day. It carries the numbers the charts draw. */
export default function DaysTable({ days }: DaysTableProps) {
  const { t, i18n } = useTranslation()

  return (
    <Card title={t('stats.days.title')}>
      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm">
          <thead className="text-muted-foreground">
            <tr>
              <th scope="col" className="py-2 pr-4 font-medium">
                {t('stats.days.day')}
              </th>
              <th scope="col" className="py-2 pr-4 font-medium">
                {t('stats.days.mode')}
              </th>
              <th scope="col" className="py-2 pr-4 text-right font-medium">
                {t('stats.days.study')}
              </th>
              <th scope="col" className="py-2 pr-4 text-right font-medium">
                {t('stats.days.pomodoros')}
              </th>
              <th scope="col" className="py-2 text-right font-medium">
                {t('stats.days.checklist')}
              </th>
            </tr>
          </thead>
          <tbody>
            {days.map((day) => {
              const total = day.habits_scheduled + day.tasks_total
              const done = day.habits_done + day.tasks_done
              return (
                <tr key={day.date} className="border-t border-border">
                  <th scope="row" className="py-2 pr-4 font-medium">
                    {formatShortDate(day.date, i18n.language)}
                  </th>
                  <td className="py-2 pr-4">
                    {day.mode === 'low_energy' ? t('stats.days.lowEnergy') : t('stats.days.normal')}
                  </td>
                  <td className="py-2 pr-4 text-right tabular-nums">
                    {formatDuration(day.study_seconds, t)}
                  </td>
                  <td className="py-2 pr-4 text-right tabular-nums">{day.pomodoros}</td>
                  <td className="py-2 text-right tabular-nums">
                    {total === 0
                      ? t('stats.days.noChecklist')
                      : t('stats.days.checklistValue', { done, total })}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </Card>
  )
}
