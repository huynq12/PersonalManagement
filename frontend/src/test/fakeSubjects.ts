import { errorResponse, jsonResponse, type RecordedRequest } from './testUtils'

export interface FakeSubject {
  id: string
  name: string
  color: string
  archived_at: string | null
  created_at: string
}

const CREATED_AT = '2026-11-01T00:00:00Z'

export function subject(
  id: string,
  name: string,
  overrides: Partial<FakeSubject> = {},
): FakeSubject {
  return { id, name, color: '#3B82F6', archived_at: null, created_at: CREATED_AT, ...overrides }
}

/**
 * An in-memory stand-in for the subjects API, stateful so a test can change something and see
 * the app show the refetched result. Pass `handle` as the dynamic handler of `stubApi`.
 */
export class FakeSubjects {
  subjects: FakeSubject[]
  /** When set, requests it returns true for fail with a 500. */
  failWhen: ((request: RecordedRequest) => boolean) | null = null
  private nextId = 1

  constructor(subjects: FakeSubject[] = []) {
    this.subjects = subjects
  }

  handle = (request: RecordedRequest): Response | undefined => {
    if (this.failWhen?.(request)) {
      return errorResponse(500, 'internal_error')
    }

    const [path, query] = request.path.split('?')
    const [root, id] = path.split('/').filter(Boolean)
    if (root !== 'subjects') {
      return undefined
    }
    const body = (request.body ?? {}) as Record<string, unknown>

    if (!id) {
      if (request.method === 'GET') {
        const includeArchived = new URLSearchParams(query).get('include_archived') === 'true'
        return jsonResponse(
          200,
          this.sorted().filter((item) => includeArchived || item.archived_at === null),
        )
      }
      if (request.method === 'POST') {
        if (this.nameTaken(String(body.name))) {
          return errorResponse(409, 'subject_name_taken')
        }
        const created = subject(`subject-new-${this.nextId++}`, String(body.name), {
          color: String(body.color),
        })
        this.subjects.push(created)
        return jsonResponse(201, created)
      }
      return undefined
    }

    const found = this.subjects.find((item) => item.id === id)
    if (!found) {
      return errorResponse(404, 'not_found')
    }
    if (request.method === 'PATCH') {
      if (typeof body.name === 'string') found.name = body.name
      if (typeof body.color === 'string') found.color = body.color
      if (typeof body.archived === 'boolean') {
        found.archived_at = body.archived ? CREATED_AT : null
      }
      return jsonResponse(200, found)
    }
    if (request.method === 'DELETE') {
      this.subjects = this.subjects.filter((item) => item.id !== id)
      return new Response(null, { status: 204 })
    }
    return undefined
  }

  private sorted(): FakeSubject[] {
    return [...this.subjects].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
  }

  private nameTaken(name: string): boolean {
    return this.subjects.some((item) => item.name.toLowerCase() === name.toLowerCase())
  }
}
