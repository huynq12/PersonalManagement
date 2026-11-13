import { useTranslation } from 'react-i18next'
import { CATEGORY_COLORS } from '../../lib/finance'

interface CategoryColorPickerProps {
  value: string
  onChange: (color: string) => void
}

/** The fixed palette as a radio group of color swatches. */
export default function CategoryColorPicker({ value, onChange }: CategoryColorPickerProps) {
  const { t } = useTranslation()

  return (
    <div
      role="radiogroup"
      aria-label={t('financeCategories.color')}
      className="flex flex-wrap gap-2"
    >
      {CATEGORY_COLORS.map((color) => (
        <button
          key={color}
          type="button"
          role="radio"
          aria-checked={color === value}
          aria-label={color}
          onClick={() => onChange(color)}
          style={{ backgroundColor: color }}
          className="size-7 rounded-full outline-offset-2 focus-visible:outline-2 focus-visible:outline-ring aria-checked:outline-2 aria-checked:outline-foreground"
        />
      ))}
    </div>
  )
}
