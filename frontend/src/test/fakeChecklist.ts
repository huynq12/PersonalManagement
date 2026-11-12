import { isoWeekday } from '../lib/dates'
import { errorResponse, jsonResponse, type RecordedRequest } from './testUtils'

export interface FakeHabit {
  id: string
  name: string
  weekdays: number[]
  sort_order: number
  archived_at: string | null
  created_at: string
  current_streak: number
}

export interface FakeTask {
  id: string
  date: string
  title: string
  is_done: boolean
  done_at: string | null
  sort_order: number
  created_at: string
}

const CREATED_AT = '2026-11-01T00:00:00Z'
const EVERY_DAY = [1, 2, 3, 4, 5, 6, 7]

export function habit(id: string, name: string, overrides: Partial<FakeHabit> = {}): FakeHabit {
  return {
    id,
    name,
    weekdays: EVERY_DAY,
    sort_order: 0,
    archived_at: null,
    created_at: CREATED_AT,
    current_streak: 0,
    ...overrides,
  }
}

export function task(id: string, date: string, title: string, isDone = false): FakeTask {
  return {
    id,
    date,
    title,
    is_done: isDone,
    done_at: isDone ? CREATED_AT : null,
    sort_order: 0,
    created_at: CREATED_AT,
  }
}

/**
 * An in-memory stand-in for the checklist API, stateful so a test can change something and see
 * the app show the refetched result. Pass `handle` as the dynamic handler of `stubApi`.
 */
export class FakeChecklist {
  habits: FakeHabit[]
  tasks: FakeTask[]
  /** Completed habit days as `"<habit id>|<date>"`. */
  completions = new Set<string>()
  /** When set, requests it returns true for fail with a 500. */
  failWhen: ((request: RecordedRequest) => boolean) | null = null
  private nextId = 1

  constructor(habits: FakeHabit[] = [], tasks: FakeTask[] = []) {
    this.habits = habits
    this.tasks = tasks
  }

  complete(habitId: string, date: string) {
    this.completions.add(`${habitId}|${date}`)
  }

  handle = (request: RecordedRequest): Response | undefined => {
    if (this.failWhen?.(request)) {
      return errorResponse(500, 'internal_error')
    }

    const [path] = request.path.split('?')
    const segments = path.split('/').filter(Boolean)
    const body = (request.body ?? {}) as Record<string, unknown>

    if (segments[0] === 'days' && request.method === 'GET') {
      return this.day(segments[1])
    }
    if (segments[0] === 'habits') {
      return this.habitRoute(request.method, segments, body)
    }
    if (segments[0] === 'tasks') {
      return this.taskRoute(request.method, segments, body)
    }
    return undefined
  }

  private day(date: string): Response {
    const weekday = isoWeekday(date)
    return jsonResponse(200, {
      date,
      habits: this.sortedHabits()
        .filter((item) => item.archived_at === null && item.weekdays.includes(weekday))
        .map((item) => ({
          id: item.id,
          name: item.name,
          completed: this.completions.has(`${item.id}|${date}`),
        })),
      tasks: this.tasks.filter((item) => item.date === date),
    })
  }

  private sortedHabits(): FakeHabit[] {
    return [...this.habits].sort((a, b) => a.sort_order - b.sort_order)
  }

  private habitRoute(
    method: string,
    segments: string[],
    body: Record<string, unknown>,
  ): Response | undefined {
    const [, id, sub, date] = segments

    if (!id) {
      if (method === 'GET') {
        return jsonResponse(200, this.sortedHabits())
      }
      if (method === 'POST') {
        const created = habit(`habit-new-${this.nextId++}`, String(body.name), {
          weekdays: (body.weekdays as number[] | undefined) ?? EVERY_DAY,
          sort_order: this.habits.length,
        })
        this.habits.push(created)
        return jsonResponse(201, created)
      }
      return undefined
    }

    const found = this.habits.find((item) => item.id === id)
    if (!found) {
      return errorResponse(404, 'not_found')
    }

    if (sub === 'completions') {
      if (method === 'PUT') {
        this.completions.add(`${id}|${date}`)
      } else {
        this.completions.delete(`${id}|${date}`)
      }
      return new Response(null, { status: 204 })
    }

    if (method === 'PATCH') {
      if (typeof body.name === 'string') found.name = body.name
      if (Array.isArray(body.weekdays)) found.weekdays = body.weekdays as number[]
      if (typeof body.sort_order === 'number') found.sort_order = body.sort_order
      if (typeof body.archived === 'boolean') {
        found.archived_at = body.archived ? CREATED_AT : null
      }
      return jsonResponse(200, found)
    }
    if (method === 'DELETE') {
      this.habits = this.habits.filter((item) => item.id !== id)
      return new Response(null, { status: 204 })
    }
    return undefined
  }

  private taskRoute(
    method: string,
    segments: string[],
    body: Record<string, unknown>,
  ): Response | undefined {
    const [, id] = segments

    if (!id) {
      if (method === 'POST') {
        const created = task(`task-new-${this.nextId++}`, String(body.date), String(body.title))
        this.tasks.push(created)
        return jsonResponse(201, created)
      }
      return undefined
    }

    const found = this.tasks.find((item) => item.id === id)
    if (!found) {
      return errorResponse(404, 'not_found')
    }
    if (method === 'PATCH') {
      if (typeof body.title === 'string') found.title = body.title
      if (typeof body.is_done === 'boolean') {
        found.is_done = body.is_done
        found.done_at = body.is_done ? CREATED_AT : null
      }
      return jsonResponse(200, found)
    }
    if (method === 'DELETE') {
      this.tasks = this.tasks.filter((item) => item.id !== id)
      return new Response(null, { status: 204 })
    }
    return undefined
  }
}
