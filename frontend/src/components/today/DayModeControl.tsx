import { useTranslation } from 'react-i18next'
import { useSetDayMode } from '../../hooks/useDay'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { DayMode } from '../../lib/checklist'
import type { IsoDate } from '../../lib/dates'
import Alert from '../ui/Alert'

interface DayModeControlProps {
  date: IsoDate
  mode: DayMode
  /** The server refuses a change: the day is not today, or a habit is already checked. */
  locked: boolean
  isToday: boolean
}

const MODES: DayMode[] = ['normal', 'low_energy']
const MODE_LABEL_KEYS = {
  normal: 'home.mode.normal',
  low_energy: 'home.mode.lowEnergy',
} as const

/**
 * Chooses between a normal and a low-energy day. Only today can be changed, and only while no
 * habit is checked, so the control is disabled with a short reason otherwise.
 */
export default function DayModeControl({ date, mode, locked, isToday }: DayModeControlProps) {
  const { t } = useTranslation()
  const setMode = useSetDayMode(date)

  return (
    <div className="flex flex-col items-start gap-2">
      <div
        role="group"
        aria-label={t('home.mode.label')}
        className="inline-flex rounded-lg border border-input p-0.5"
      >
        {MODES.map((option) => {
          const isSelected = option === mode
          return (
            <button
              key={option}
              type="button"
              aria-pressed={isSelected}
              disabled={locked || setMode.isPending}
              onClick={() => {
                if (!isSelected) {
                  setMode.mutate(option)
                }
              }}
              className={`h-9 rounded-md px-3 text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:cursor-not-allowed ${
                isSelected
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent disabled:hover:bg-transparent'
              } ${locked && !isSelected ? 'opacity-60' : ''}`}
            >
              {t(MODE_LABEL_KEYS[option])}
            </button>
          )
        })}
      </div>

      {locked ? (
        <p className="text-sm text-muted-foreground">
          {t(isToday ? 'home.mode.lockedChecked' : 'home.mode.lockedNotToday')}
        </p>
      ) : null}
      {setMode.isError ? <Alert variant="error">{t(errorMessageKey(setMode.error))}</Alert> : null}
    </div>
  )
}
