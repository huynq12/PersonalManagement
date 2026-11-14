import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { addDays, formatWeekRange, type IsoDate } from '../../lib/dates'
import IconButton from '../ui/IconButton'

interface WeekPickerProps {
  /** The Monday of the week being shown. */
  weekStart: IsoDate
  /** The Monday of the newest week that can be shown; "next" is disabled there. */
  latest: IsoDate
  onChange: (weekStart: IsoDate) => void
}

/** Previous and next buttons around the date range of the week being shown. */
export default function WeekPicker({ weekStart, latest, onChange }: WeekPickerProps) {
  const { t, i18n } = useTranslation()

  return (
    <div className="flex items-center gap-2">
      <IconButton
        icon={ChevronLeft}
        label={t('stats.week.previous')}
        onClick={() => onChange(addDays(weekStart, -7))}
      />
      <h2 className="min-w-48 text-center text-lg font-semibold" aria-live="polite">
        {formatWeekRange(weekStart, i18n.language)}
      </h2>
      <IconButton
        icon={ChevronRight}
        label={t('stats.week.next')}
        disabled={weekStart >= latest}
        onClick={() => onChange(addDays(weekStart, 7))}
      />
    </div>
  )
}
