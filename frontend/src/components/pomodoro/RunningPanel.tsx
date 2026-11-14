import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { useCancelSession, useCompleteSession } from '../../hooks/useStudySessions'
import { errorMessageKey } from '../../i18n/errorMessage'
import { remainingSeconds } from '../../lib/pomodoro'
import type { StudySession } from '../../lib/studySessions'
import type { Subject } from '../../lib/subjects'
import ColorDot from '../subjects/ColorDot'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import ClockFace from './ClockFace'

interface RunningPanelProps {
  session: StudySession
  /** The session's subject; `undefined` when it has none. */
  subject: Subject | undefined
  /** The current time in milliseconds. */
  now: number
}

/** A focus session is running: the countdown, which completes the session when it hits zero. */
export default function RunningPanel({ session, subject, now }: RunningPanelProps) {
  const { t } = useTranslation()
  const { mutate: complete, ...completion } = useCompleteSession()
  const cancel = useCancelSession()

  const endsAt = Date.parse(session.started_at) + session.planned_minutes * 60_000
  const remaining = remainingSeconds(endsAt, now)

  // Ask once per session: the clock keeps ticking while the request is in flight. If it fails,
  // the retry button below asks again.
  const requestedFor = useRef<string | null>(null)
  useEffect(() => {
    if (remaining === 0 && requestedFor.current !== session.id) {
      requestedFor.current = session.id
      complete(session.id)
    }
  }, [remaining, session.id, complete])

  const failure = [completion, cancel].find((mutation) => mutation.isError)

  return (
    <ClockFace label={t('pomodoro.phase.focus')} seconds={remaining}>
      <p className="flex items-center gap-2 text-sm">
        {subject ? <ColorDot color={subject.color} /> : null}
        {subject ? subject.name : t('pomodoro.noSubject')}
      </p>

      {failure ? <Alert variant="error">{t(errorMessageKey(failure.error))}</Alert> : null}

      {remaining === 0 ? (
        <>
          <p className="text-sm text-muted-foreground">{t('pomodoro.saving')}</p>
          {completion.isError ? (
            <Button variant="secondary" onClick={() => complete(session.id)}>
              {t('common.retry')}
            </Button>
          ) : null}
        </>
      ) : (
        <Button
          variant="secondary"
          disabled={cancel.isPending}
          onClick={() => cancel.mutate(session.id)}
        >
          {t('pomodoro.cancel')}
        </Button>
      )}
    </ClockFace>
  )
}
