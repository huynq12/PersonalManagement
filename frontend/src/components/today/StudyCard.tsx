import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { formatDuration } from '../../lib/duration'
import Card from '../ui/Card'

interface StudyCardProps {
  /** Time of the day's completed study sessions, in seconds. */
  studySeconds: number
}

/** A read-only summary of the day's study time, with a shortcut to the Pomodoro screen. */
export default function StudyCard({ studySeconds }: StudyCardProps) {
  const { t } = useTranslation()

  return (
    <Card title={t('home.study.title')}>
      {studySeconds > 0 ? (
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 text-sm">
          <dt className="text-muted-foreground">{t('home.study.timeStudied')}</dt>
          <dd className="text-right font-medium">{formatDuration(studySeconds, t)}</dd>
        </dl>
      ) : (
        <p className="text-sm text-muted-foreground">{t('home.study.none')}</p>
      )}
      <Link
        to="/pomodoro"
        className="text-sm font-medium text-foreground underline underline-offset-4"
      >
        {t('home.study.openPomodoro')}
      </Link>
    </Card>
  )
}
