import { useTranslation } from 'react-i18next'
import { formatDuration } from '../../lib/duration'
import type { SubjectStats } from '../../lib/stats'
import Card from '../ui/Card'

interface SubjectBreakdownProps {
  subjects: SubjectStats[]
}

/** The week's study time per subject as a list with proportional bars, longest first. */
export default function SubjectBreakdown({ subjects }: SubjectBreakdownProps) {
  const { t } = useTranslation()
  const longest = Math.max(...subjects.map((subject) => subject.study_seconds), 0)

  return (
    <Card title={t('stats.subjects.title')}>
      {subjects.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t('stats.subjects.empty')}</p>
      ) : (
        <ul className="flex flex-col gap-3">
          {subjects.map((subject) => (
            <li key={subject.subject_id ?? 'none'} className="flex flex-col gap-1">
              <div className="flex justify-between gap-3 text-sm">
                <span>
                  {subject.name ?? t('stats.subjects.none')}
                  <span className="ml-2 text-xs text-muted-foreground">
                    {t('stats.subjects.pomodoros', { count: subject.pomodoros })}
                  </span>
                </span>
                <span className="tabular-nums">{formatDuration(subject.study_seconds, t)}</span>
              </div>
              <div aria-hidden="true" className="h-2 overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full rounded-full bg-muted-foreground"
                  style={{
                    width: `${longest === 0 ? 0 : (subject.study_seconds / longest) * 100}%`,
                    ...(subject.color ? { backgroundColor: subject.color } : {}),
                  }}
                />
              </div>
            </li>
          ))}
        </ul>
      )}
    </Card>
  )
}
