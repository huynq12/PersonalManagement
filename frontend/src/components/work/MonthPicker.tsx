import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { addMonths, formatMonth, type IsoMonth } from '../../lib/dates'
import IconButton from '../ui/IconButton'

interface MonthPickerProps {
  month: IsoMonth
  /** The newest month that can be shown; "next" is disabled there. */
  latest: IsoMonth
  onChange: (month: IsoMonth) => void
}

/** Previous and next buttons around the name of the month being shown. */
export default function MonthPicker({ month, latest, onChange }: MonthPickerProps) {
  const { t, i18n } = useTranslation()

  return (
    <div className="flex items-center gap-2">
      <IconButton
        icon={ChevronLeft}
        label={t('workLog.month.previous')}
        onClick={() => onChange(addMonths(month, -1))}
      />
      <h2 className="min-w-40 text-center text-lg font-semibold capitalize" aria-live="polite">
        {formatMonth(month, i18n.language)}
      </h2>
      <IconButton
        icon={ChevronRight}
        label={t('workLog.month.next')}
        disabled={month >= latest}
        onClick={() => onChange(addMonths(month, 1))}
      />
    </div>
  )
}
