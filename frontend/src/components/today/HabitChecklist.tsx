import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { useToggleHabit } from '../../hooks/useDay'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { DayHabit, DayMode } from '../../lib/checklist'
import type { IsoDate } from '../../lib/dates'
import Alert from '../ui/Alert'
import Card from '../ui/Card'
import Checkbox from '../ui/Checkbox'

interface HabitChecklistProps {
  date: IsoDate
  habits: DayHabit[]
  mode: DayMode
  /** A future day: habits are listed but cannot be checked (the API rejects it too). */
  isFuture: boolean
}

/** The habits scheduled for one day, each with a checkbox. */
export default function HabitChecklist({ date, habits, mode, isFuture }: HabitChecklistProps) {
  const { t } = useTranslation()
  const toggle = useToggleHabit(date)
  const done = habits.filter((habit) => habit.completed).length

  return (
    <Card
      title={t('home.habits.title')}
      aside={habits.length > 0 ? t('home.progress', { done, total: habits.length }) : null}
    >
      {toggle.isError ? <Alert variant="error">{t(errorMessageKey(toggle.error))}</Alert> : null}

      {habits.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {t(mode === 'low_energy' ? 'home.habits.emptyLowEnergy' : 'home.habits.empty')}{' '}
          <Link className="font-medium text-foreground underline underline-offset-4" to="/habits">
            {t('home.habits.manage')}
          </Link>
        </p>
      ) : (
        <ul className="flex flex-col">
          {habits.map((habit) => (
            <li key={habit.id}>
              <label className="flex min-h-11 items-center gap-3 rounded-md px-1 hover:bg-accent has-disabled:hover:bg-transparent">
                <Checkbox
                  checked={habit.completed}
                  disabled={isFuture}
                  onChange={(event) =>
                    toggle.mutate({ habitId: habit.id, completed: event.target.checked })
                  }
                />
                <span
                  className={
                    habit.completed ? 'text-sm text-muted-foreground line-through' : 'text-sm'
                  }
                >
                  {habit.name}
                </span>
              </label>
            </li>
          ))}
        </ul>
      )}

      {isFuture && habits.length > 0 ? (
        <p className="text-sm text-muted-foreground">{t('home.habits.futureNote')}</p>
      ) : null}
    </Card>
  )
}
