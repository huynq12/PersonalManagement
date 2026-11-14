import { useTranslation } from 'react-i18next'
import { remainingSeconds, type ActiveBreak } from '../../lib/pomodoro'
import Button from '../ui/Button'
import ClockFace from './ClockFace'

interface BreakPanelProps {
  activeBreak: ActiveBreak
  /** The current time in milliseconds. */
  now: number
  onSkip: () => void
}

/** A break after a focus session. It ends by itself; the user may also skip it. */
export default function BreakPanel({ activeBreak, now, onSkip }: BreakPanelProps) {
  const { t } = useTranslation()
  const label = t(
    activeBreak.kind === 'long' ? 'pomodoro.phase.longBreak' : 'pomodoro.phase.shortBreak',
  )

  return (
    <ClockFace label={label} seconds={remainingSeconds(activeBreak.endsAt, now)}>
      <Button variant="secondary" onClick={onSkip}>
        {t('pomodoro.skipBreak')}
      </Button>
    </ClockFace>
  )
}
