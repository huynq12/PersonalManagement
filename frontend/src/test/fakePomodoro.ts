import { todayIn } from '../lib/dates'
import { FakeSubjects } from './fakeSubjects'
import { errorResponse, jsonResponse, type RecordedRequest } from './testUtils'

export interface FakeSession {
  id: string
  subject_id: string | null
  status: 'running' | 'completed' | 'cancelled'
  planned_minutes: number
  started_at: string
  ended_at: string | null
  duration_seconds: number | null
  local_date: string
}

export interface FakeSettings {
  focus_minutes: number
  short_break_minutes: number
  long_break_minutes: number
  long_break_every: number
  hourly_wage: number | null
  base_work_hours: number | null
  monthly_budget: number | null
}

export const DEFAULT_SETTINGS: FakeSettings = {
  focus_minutes: 25,
  short_break_minutes: 5,
  long_break_minutes: 15,
  long_break_every: 4,
  hourly_wage: null,
  base_work_hours: null,
  monthly_budget: null,
}

const TIME_ZONE = 'Asia/Ho_Chi_Minh'

/** An ISO timestamp `milliseconds` before the (possibly faked) current time. */
export function ago(milliseconds: number): string {
  return new Date(Date.now() - milliseconds).toISOString()
}

const MINUTE = 60_000

/** A session that is still running, started `minutesAgo` minutes ago. */
export function runningSession(
  id: string,
  minutesAgo: number,
  overrides: Partial<FakeSession> = {},
): FakeSession {
  return {
    id,
    subject_id: null,
    status: 'running',
    planned_minutes: 25,
    started_at: ago(minutesAgo * MINUTE),
    ended_at: null,
    duration_seconds: null,
    local_date: todayIn(TIME_ZONE),
    ...overrides,
  }
}

/** A completed session that ended `endedMinutesAgo` minutes ago after a full 25 minutes. */
export function completedSession(
  id: string,
  endedMinutesAgo: number,
  overrides: Partial<FakeSession> = {},
): FakeSession {
  return {
    id,
    subject_id: null,
    status: 'completed',
    planned_minutes: 25,
    started_at: ago((endedMinutesAgo + 25) * MINUTE),
    ended_at: ago(endedMinutesAgo * MINUTE),
    duration_seconds: 25 * 60,
    local_date: todayIn(TIME_ZONE),
    ...overrides,
  }
}

/**
 * An in-memory stand-in for the Pomodoro API: settings, subjects, and study sessions. Stateful,
 * so a test can act and see the refetched result. Pass `handle` as the dynamic handler of
 * `stubApi`. Time comes from `Date.now()`, which tests fix with fake system time.
 */
export class FakePomodoro {
  readonly subjects: FakeSubjects
  settings: FakeSettings = { ...DEFAULT_SETTINGS }
  sessions: FakeSession[]
  /** When set, requests it returns true for fail with a 500. */
  failWhen: ((request: RecordedRequest) => boolean) | null = null
  private nextId = 1

  constructor(subjects: FakeSubjects, sessions: FakeSession[] = []) {
    this.subjects = subjects
    this.sessions = sessions
  }

  handle = (request: RecordedRequest): Response | undefined => {
    if (this.failWhen?.(request)) {
      return errorResponse(500, 'internal_error')
    }
    const [path, query] = request.path.split('?')
    const segments = path.split('/').filter(Boolean)

    if (segments[0] === 'subjects') {
      return this.subjects.handle(request)
    }
    if (segments[0] === 'settings' && request.method === 'GET') {
      return jsonResponse(200, this.settings)
    }
    if (segments[0] === 'study-sessions') {
      return this.sessionRoute(request, segments, query)
    }
    return undefined
  }

  private running(): FakeSession | undefined {
    return this.sessions.find((item) => item.status === 'running')
  }

  private sessionRoute(
    request: RecordedRequest,
    segments: string[],
    query: string | undefined,
  ): Response | undefined {
    const [, id, action] = segments
    const body = (request.body ?? {}) as Record<string, unknown>

    if (!id) {
      if (request.method === 'GET') {
        const params = new URLSearchParams(query)
        const from = params.get('from') ?? ''
        const to = params.get('to') ?? ''
        const listed = this.sessions
          .filter((item) => item.local_date >= from && item.local_date <= to)
          .sort((a, b) => b.started_at.localeCompare(a.started_at))
        return jsonResponse(200, listed)
      }
      if (request.method === 'POST') {
        if (this.running()) {
          return errorResponse(409, 'study_session_already_running')
        }
        const created = runningSession(`session-new-${this.nextId++}`, 0, {
          subject_id: typeof body.subject_id === 'string' ? body.subject_id : null,
          planned_minutes: this.settings.focus_minutes,
        })
        this.sessions.push(created)
        return jsonResponse(201, created)
      }
      return undefined
    }

    if (id === 'current') {
      const current = this.running()
      return current ? jsonResponse(200, current) : new Response(null, { status: 204 })
    }

    const found = this.sessions.find((item) => item.id === id)
    if (!found) {
      return errorResponse(404, 'not_found')
    }
    if (request.method === 'DELETE') {
      this.sessions = this.sessions.filter((item) => item.id !== id)
      return new Response(null, { status: 204 })
    }
    if (request.method === 'POST' && (action === 'complete' || action === 'cancel')) {
      if (found.status !== 'running') {
        return errorResponse(409, 'study_session_not_running')
      }
      const now = Date.now()
      found.ended_at = new Date(now).toISOString()
      if (action === 'complete') {
        const elapsed = Math.floor((now - Date.parse(found.started_at)) / 1000)
        found.status = 'completed'
        found.duration_seconds = Math.min(elapsed, found.planned_minutes * 60)
      } else {
        found.status = 'cancelled'
      }
      return jsonResponse(200, found)
    }
    return undefined
  }
}
