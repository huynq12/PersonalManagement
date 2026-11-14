import { cleanup, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import { setAccessToken } from '../lib/api'
import { addDays, mondayOf, todayIn } from '../lib/dates'
import type { DayStats, WeeklyStats, WeekTotals } from '../lib/stats'
import {
  errorResponse,
  jsonResponse,
  renderApp,
  signedIn,
  stubApi,
  USER,
  type RecordedRequest,
} from '../test/testUtils'

const CURRENT_WEEK = mondayOf(todayIn(USER.timezone))
const PREVIOUS_WEEK = addDays(CURRENT_WEEK, -7)

const NO_TOTALS: WeekTotals = { study_seconds: 0, pomodoros: 0, completion_rate: null }

function day(date: string, overrides: Partial<DayStats> = {}): DayStats {
  return {
    date,
    mode: 'normal',
    study_seconds: 0,
    pomodoros: 0,
    habits_scheduled: 0,
    habits_done: 0,
    tasks_total: 0,
    tasks_done: 0,
    completion_rate: null,
    ...overrides,
  }
}

function week(weekStart: string, overrides: Partial<WeeklyStats> = {}): WeeklyStats {
  return {
    week_start: weekStart,
    week_end: addDays(weekStart, 6),
    days: Array.from({ length: 7 }, (_, index) => day(addDays(weekStart, index))),
    subjects: [],
    habits: [],
    totals: NO_TOTALS,
    previous_totals: NO_TOTALS,
    ...overrides,
  }
}

let weeks: Record<string, WeeklyStats>
let requests: RecordedRequest[]
/** When set, `GET /stats/weekly` answers with this error code. */
let statsError: string | null

function renderStats(path = '/stats') {
  requests = stubApi({ ...signedIn }, (request) => {
    const url = new URL(request.path, 'http://localhost')
    if (request.method !== 'GET' || url.pathname !== '/stats/weekly') {
      return undefined
    }
    if (statsError) {
      return errorResponse(500, statsError)
    }
    const start = url.searchParams.get('week_start') ?? ''
    return jsonResponse(200, weeks[start] ?? week(start))
  })
  renderApp(path)
}

function weeklyRequests() {
  return requests.filter((request) => request.path.startsWith('/stats/weekly'))
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  weeks = {}
  statsError = null
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Weekly statistics screen', () => {
  it('asks for the current week by default', async () => {
    renderStats()

    await screen.findByRole('heading', { name: 'Weekly stats' })

    await vi.waitFor(() =>
      expect(weeklyRequests().map((request) => request.path)).toEqual([
        `/stats/weekly?week_start=${CURRENT_WEEK}`,
      ]),
    )
  })

  it('shows the totals with a comparison against the previous week', async () => {
    weeks[CURRENT_WEEK] = week(CURRENT_WEEK, {
      totals: { study_seconds: 5400, pomodoros: 4, completion_rate: 0.75 },
      previous_totals: { study_seconds: 3600, pomodoros: 6, completion_rate: 0.5 },
    })
    renderStats()

    const study = (await screen.findByText('Study time', { selector: 'dt' })).parentElement!
    expect(within(study).getByText('1 h 30 min')).toBeTruthy()
    expect(within(study).getByText('30 min more than last week')).toBeTruthy()

    const pomodoros = screen.getByText('Pomodoros', { selector: 'dt' }).parentElement!
    expect(within(pomodoros).getByText('2 less than last week')).toBeTruthy()

    const checklist = screen.getByText('Checklist completion', { selector: 'dt' }).parentElement!
    expect(within(checklist).getByText('75%')).toBeTruthy()
    expect(within(checklist).getByText('25 pts more than last week')).toBeTruthy()
  })

  it('says so when there is no previous checklist rate to compare with', async () => {
    weeks[CURRENT_WEEK] = week(CURRENT_WEEK, {
      totals: { study_seconds: 0, pomodoros: 0, completion_rate: 0.5 },
    })
    renderStats()

    const checklist = (await screen.findByText('Checklist completion', { selector: 'dt' }))
      .parentElement!
    expect(within(checklist).getByText('No data for last week')).toBeTruthy()
    const study = screen.getByText('Study time', { selector: 'dt' }).parentElement!
    expect(within(study).getByText('Same as last week')).toBeTruthy()
  })

  it('marks low-energy days in the day table', async () => {
    const days = Array.from({ length: 7 }, (_, index) =>
      day(addDays(CURRENT_WEEK, index), index === 1 ? { mode: 'low_energy' } : {}),
    )
    weeks[CURRENT_WEEK] = week(CURRENT_WEEK, { days })
    renderStats()

    const table = await screen.findByRole('table')
    const rows = within(table).getAllByRole('row').slice(1)

    expect(rows).toHaveLength(7)
    expect(within(rows[0]).getByText('Normal')).toBeTruthy()
    expect(within(rows[1]).getByText('Low energy')).toBeTruthy()
    expect(within(table).getAllByText('Low energy')).toHaveLength(1)
  })

  it('lists each day with its study time, Pomodoros, and checklist progress', async () => {
    const days = Array.from({ length: 7 }, (_, index) =>
      index === 2
        ? day(addDays(CURRENT_WEEK, index), {
            study_seconds: 3000,
            pomodoros: 2,
            habits_scheduled: 3,
            habits_done: 2,
            tasks_total: 1,
            tasks_done: 1,
            completion_rate: 0.75,
          })
        : day(addDays(CURRENT_WEEK, index)),
    )
    weeks[CURRENT_WEEK] = week(CURRENT_WEEK, { days })
    renderStats()

    const rows = within(await screen.findByRole('table')).getAllByRole('row').slice(1)

    expect(within(rows[2]).getByText('50 min')).toBeTruthy()
    expect(within(rows[2]).getByText('3 of 4')).toBeTruthy()
    expect(within(rows[3]).getByText('Nothing scheduled')).toBeTruthy()
  })

  it('lists study time per subject, including sessions without a subject', async () => {
    weeks[CURRENT_WEEK] = week(CURRENT_WEEK, {
      subjects: [
        { subject_id: 's1', name: 'Math', color: '#3B82F6', study_seconds: 3600, pomodoros: 2 },
        { subject_id: null, name: null, color: null, study_seconds: 600, pomodoros: 1 },
      ],
    })
    renderStats()

    const card = (await screen.findByRole('region', { name: 'Study time per subject' }))
    expect(within(card).getByText('Math')).toBeTruthy()
    expect(within(card).getByText('1 h')).toBeTruthy()
    expect(within(card).getByText('2 Pomodoros')).toBeTruthy()
    expect(within(card).getByText('No subject')).toBeTruthy()
    expect(within(card).getByText('10 min')).toBeTruthy()
  })

  it('shows empty states for a week without study sessions or habits', async () => {
    renderStats()

    expect(await screen.findByText('No study sessions this week.')).toBeTruthy()
    expect(screen.getByText('You have no active habits.')).toBeTruthy()
  })

  it('shows the current and the longest streak of each habit', async () => {
    weeks[CURRENT_WEEK] = week(CURRENT_WEEK, {
      habits: [{ id: 'h1', name: 'Read', current_streak: 1, longest_streak: 12 }],
    })
    renderStats()

    const card = await screen.findByRole('region', { name: 'Habit streaks' })
    expect(within(card).getByText('Read')).toBeTruthy()
    expect(within(card).getByText('1 day')).toBeTruthy()
    expect(within(card).getByText('12 days')).toBeTruthy()
  })

  it('moves between weeks and cannot go past the current one', async () => {
    const user = userEvent.setup()
    renderStats()
    const next = await screen.findByRole('button', { name: 'Next week' })
    expect((next as HTMLButtonElement).disabled).toBe(true)

    await user.click(screen.getByRole('button', { name: 'Previous week' }))

    expect(
      await screen.findByRole('button', { name: 'Next week' }).then(
        (button) => (button as HTMLButtonElement).disabled,
      ),
    ).toBe(false)
    expect(weeklyRequests().map((request) => request.path)).toContain(
      `/stats/weekly?week_start=${PREVIOUS_WEEK}`,
    )

    await user.click(screen.getByRole('button', { name: 'Next week' }))

    expect(
      (screen.getByRole('button', { name: 'Next week' }) as HTMLButtonElement).disabled,
    ).toBe(true)
  })

  it('shows the week from the URL when it is a valid Monday', async () => {
    renderStats(`/stats?week=${PREVIOUS_WEEK}`)

    await screen.findByRole('heading', { name: 'Weekly stats' })

    await vi.waitFor(() =>
      expect(weeklyRequests().map((request) => request.path)).toEqual([
        `/stats/weekly?week_start=${PREVIOUS_WEEK}`,
      ]),
    )
  })

  it('falls back to the current week for a date that is not a Monday or is in the future', async () => {
    renderStats(`/stats?week=${addDays(PREVIOUS_WEEK, 2)}`)
    await screen.findByRole('heading', { name: 'Weekly stats' })
    await vi.waitFor(() =>
      expect(weeklyRequests().map((request) => request.path)).toEqual([
        `/stats/weekly?week_start=${CURRENT_WEEK}`,
      ]),
    )
    cleanup()

    renderStats(`/stats?week=${addDays(CURRENT_WEEK, 7)}`)
    await screen.findByRole('heading', { name: 'Weekly stats' })
    await vi.waitFor(() =>
      expect(weeklyRequests().map((request) => request.path)).toEqual([
        `/stats/weekly?week_start=${CURRENT_WEEK}`,
      ]),
    )
  })

  it('shows a translated error with a retry button when loading fails', async () => {
    statsError = 'internal_error'
    renderStats()

    expect(await screen.findByRole('alert')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy()
  })

  it('draws a chart for study time and one for checklist completion', async () => {
    renderStats()

    expect(await screen.findByRole('img', { name: 'Study time per day, in minutes' })).toBeTruthy()
    expect(screen.getByRole('img', { name: 'Checklist completion per day, in percent' })).toBeTruthy()
  })
})
