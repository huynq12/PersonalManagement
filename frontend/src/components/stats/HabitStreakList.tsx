import { useTranslation } from 'react-i18next'
import type { HabitStreaks } from '../../lib/stats'
import Card from '../ui/Card'

interface HabitStreakListProps {
  habits: HabitStreaks[]
}

/** The current and the longest streak of every active habit. */
export default function HabitStreakList({ habits }: HabitStreakListProps) {
  const { t } = useTranslation()

  return (
    <Card title={t('stats.habits.title')}>
      {habits.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t('stats.habits.empty')}</p>
      ) : (
        <ul className="flex flex-col divide-y divide-border">
          {habits.map((habit) => (
            <li key={habit.id} className="flex flex-wrap items-baseline justify-between gap-x-4 py-2">
              <span className="text-sm font-medium">{habit.name}</span>
              <span className="flex gap-4 text-sm text-muted-foreground">
                <span>
                  {t('stats.habits.current')}:{' '}
                  <span className="tabular-nums text-foreground">
                    {t('stats.habits.days', { count: habit.current_streak })}
                  </span>
                </span>
                <span>
                  {t('stats.habits.longest')}:{' '}
                  <span className="tabular-nums text-foreground">
                    {t('stats.habits.days', { count: habit.longest_streak })}
                  </span>
                </span>
              </span>
            </li>
          ))}
        </ul>
      )}
    </Card>
  )
}
