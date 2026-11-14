import { useTranslation } from 'react-i18next'
import { Bar, BarChart, CartesianGrid, Cell, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'

export interface DayPoint {
  /** The short weekday name shown under the bar. */
  label: string
  value: number
  /** Drawn in a muted color so low-energy days stand out. */
  lowEnergy: boolean
}

interface DayBarChartProps {
  points: DayPoint[]
  /** What the chart shows, read by screen readers. The values are also in the day table. */
  label: string
  /** Writes a value for the tooltip. */
  formatValue: (value: number) => string
  /** The top of the value axis; the data decides it when omitted. */
  max?: number
}

const CHART_HEIGHT = 220

/** One bar per day of the week, with low-energy days drawn in a muted color and a legend. */
export default function DayBarChart({ points, label, formatValue, max }: DayBarChartProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col gap-3">
      <div role="img" aria-label={label}>
        <ResponsiveContainer
          width="100%"
          height={CHART_HEIGHT}
          // Used until the container is measured, and in tests where there is no layout.
          initialDimension={{ width: 480, height: CHART_HEIGHT }}
        >
          <BarChart data={points} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
            <CartesianGrid vertical={false} stroke="var(--border)" />
            <XAxis
              dataKey="label"
              tickLine={false}
              axisLine={false}
              tick={{ fill: 'var(--muted-foreground)', fontSize: 12 }}
            />
            <YAxis
              width={40}
              tickLine={false}
              axisLine={false}
              domain={[0, max ?? 'auto']}
              allowDecimals={false}
              tick={{ fill: 'var(--muted-foreground)', fontSize: 12 }}
            />
            <Tooltip
              cursor={{ fill: 'var(--muted)' }}
              formatter={(value) => formatValue(Number(value))}
              contentStyle={{
                background: 'var(--background)',
                border: '1px solid var(--border)',
                borderRadius: 8,
                color: 'var(--foreground)',
              }}
            />
            <Bar dataKey="value" radius={[4, 4, 0, 0]} isAnimationActive={false}>
              {points.map((point) => (
                <Cell
                  key={point.label}
                  fill={point.lowEnergy ? 'var(--muted-foreground)' : 'var(--primary)'}
                />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>
      <ul className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
        <li className="flex items-center gap-1.5">
          <span aria-hidden="true" className="size-2.5 rounded-sm bg-primary" />
          {t('stats.legend.normal')}
        </li>
        <li className="flex items-center gap-1.5">
          <span aria-hidden="true" className="size-2.5 rounded-sm bg-muted-foreground" />
          {t('stats.legend.lowEnergy')}
        </li>
      </ul>
    </div>
  )
}
