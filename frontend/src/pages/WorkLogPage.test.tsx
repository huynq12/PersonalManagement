import { cleanup, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import { setAccessToken } from '../lib/api'
import { addMonths, todayIn } from '../lib/dates'
import type { Settings } from '../lib/settings'
import type { WorkEntry } from '../lib/workEntries'
import {
  errorResponse,
  jsonResponse,
  renderApp,
  signedIn,
  stubApi,
  USER,
  type RecordedRequest,
} from '../test/testUtils'

const TODAY = todayIn(USER.timezone)
const THIS_MONTH = TODAY.slice(0, 7)

const SETTINGS: Settings = {
  focus_minutes: 25,
  short_break_minutes: 5,
  long_break_minutes: 15,
  long_break_every: 4,
  hourly_wage: 30000,
  base_work_hours: 8,
  monthly_budget: null,
}

function entry(date: string, hours: number, overrides: Partial<WorkEntry> = {}): WorkEntry {
  const wage = 30000
  const base = 8
  return {
    date,
    hours,
    hourly_wage: wage,
    base_hours: base,
    pay: Math.round(hours * wage),
    comparison: hours > base ? 'above' : hours < base ? 'below' : 'equal',
    note: null,
    ...overrides,
  }
}

let settings: Settings
let entries: WorkEntry[]
let requests: RecordedRequest[]
/** When set, `PUT /work-entries/...` answers with this error code. */
let putError: string | null

function monthView(month: string) {
  const inMonth = entries.filter((item) => item.date.startsWith(month))
  return {
    entries: inMonth,
    summary: {
      total_hours: inMonth.reduce((sum, item) => sum + item.hours, 0),
      total_pay: inMonth.reduce((sum, item) => sum + item.pay, 0),
      days_worked: inMonth.length,
    },
  }
}

function renderWorkLog(path = '/work-log') {
  requests = stubApi(
    { ...signedIn, 'GET /settings': () => jsonResponse(200, settings) },
    (request) => {
      const url = new URL(request.path, 'http://localhost')
      if (request.method === 'GET' && url.pathname === '/work-entries') {
        return jsonResponse(200, monthView(url.searchParams.get('month') ?? ''))
      }
      const match = /^\/work-entries\/(\d{4}-\d{2}-\d{2})$/.exec(url.pathname)
      if (!match) {
        return undefined
      }
      const date = match[1]
      if (request.method === 'PUT') {
        if (putError) {
          return errorResponse(409, putError)
        }
        const body = request.body as { hours: number; note?: string | null }
        const saved = entry(date, body.hours, { note: body.note ?? null })
        entries = [...entries.filter((item) => item.date !== date), saved].sort((a, b) =>
          a.date.localeCompare(b.date),
        )
        return jsonResponse(200, saved)
      }
      if (request.method === 'DELETE') {
        entries = entries.filter((item) => item.date !== date)
        return new Response(null, { status: 204 })
      }
      return undefined
    },
  )
  renderApp(path)
}

function sent(method: string, prefix: string) {
  return requests.filter((request) => request.method === method && request.path.startsWith(prefix))
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  settings = { ...SETTINGS }
  entries = []
  putError = null
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Work log screen', () => {
  it('shows an empty state linking to Settings instead of the form when the wage is not set', async () => {
    settings = { ...SETTINGS, hourly_wage: null }
    renderWorkLog()

    const link = await screen.findByRole('link', { name: 'Open Settings' })

    expect(link.getAttribute('href')).toBe('/settings')
    expect(screen.queryByLabelText('Hours')).toBeNull()
  })

  it('shows the empty state when the base hours are not set', async () => {
    settings = { ...SETTINGS, base_work_hours: null }
    renderWorkLog()

    expect(await screen.findByRole('link', { name: 'Open Settings' })).toBeTruthy()
  })

  it('lists the month with hours, pay, badge, and totals', async () => {
    entries = [
      entry(`${THIS_MONTH}-01`, 9, { note: 'Busy day' }),
      entry(`${THIS_MONTH}-02`, 8),
      entry(`${THIS_MONTH}-03`, 6.5),
    ]
    renderWorkLog()

    const rows = await screen.findAllByRole('row')

    // Header, three days, and the totals row.
    expect(rows).toHaveLength(5)
    expect(within(rows[1]).getByText('Above base')).toBeTruthy()
    expect(within(rows[1]).getByText('Busy day')).toBeTruthy()
    expect(within(rows[2]).getByText('Equal to base')).toBeTruthy()
    expect(within(rows[3]).getByText('Below base')).toBeTruthy()
    expect(within(rows[3]).getByText(/195,000/)).toBeTruthy()
    const totals = within(rows[4])
    expect(totals.getByText('23.5')).toBeTruthy()
    expect(totals.getByText(/705,000/)).toBeTruthy()
    expect(totals.getByText('3 days')).toBeTruthy()
  })

  it('shows a hint when the month has no entries', async () => {
    renderWorkLog()

    expect(await screen.findByText('No hours logged this month.')).toBeTruthy()
  })

  it('saves an entry for the chosen date and shows it in the list', async () => {
    renderWorkLog()
    await userEvent.type(await screen.findByLabelText('Hours'), '7,5')
    await userEvent.type(screen.getByLabelText('Note'), 'Inventory')

    await userEvent.click(screen.getByRole('button', { name: 'Save entry' }))

    expect(await screen.findByText('Inventory')).toBeTruthy()
    expect(sent('PUT', '/work-entries/').map((request) => [request.path, request.body])).toEqual([
      [`/work-entries/${TODAY}`, { hours: 7.5, note: 'Inventory' }],
    ])
    expect(screen.getByText('Below base')).toBeTruthy()
  })

  it('sends no note when the note is left empty', async () => {
    renderWorkLog()
    await userEvent.type(await screen.findByLabelText('Hours'), '8')

    await userEvent.click(screen.getByRole('button', { name: 'Save entry' }))

    await screen.findByText('Equal to base')
    expect(sent('PUT', '/work-entries/')[0].body).toEqual({ hours: 8, note: null })
  })

  it('does not send invalid hours and shows the validation message', async () => {
    renderWorkLog()
    await userEvent.type(await screen.findByLabelText('Hours'), '25')

    await userEvent.click(screen.getByRole('button', { name: 'Save entry' }))

    expect(
      await screen.findByText(
        'Enter hours greater than 0 and at most 24, with up to two decimals.',
      ),
    ).toBeTruthy()
    expect(sent('PUT', '/work-entries/')).toEqual([])
  })

  it('loads a row into the form for editing and saves it for the same date', async () => {
    entries = [entry(`${THIS_MONTH}-01`, 6, { note: 'Short day' })]
    renderWorkLog()
    await userEvent.click(await screen.findByRole('button', { name: /Edit/ }))

    expect((screen.getByLabelText('Hours') as HTMLInputElement).value).toBe('6')
    expect((screen.getByLabelText('Note') as HTMLInputElement).value).toBe('Short day')
    expect((screen.getByLabelText('Date') as HTMLInputElement).value).toBe(`${THIS_MONTH}-01`)

    await userEvent.clear(screen.getByLabelText('Hours'))
    await userEvent.type(screen.getByLabelText('Hours'), '9')
    await userEvent.click(screen.getByRole('button', { name: 'Save entry' }))

    expect(await screen.findByText('Above base')).toBeTruthy()
    expect(sent('PUT', '/work-entries/').map((request) => request.path)).toEqual([
      `/work-entries/${THIS_MONTH}-01`,
    ])
  })

  it('asks for confirmation before deleting an entry', async () => {
    entries = [entry(`${THIS_MONTH}-01`, 6)]
    renderWorkLog()
    await userEvent.click(await screen.findByRole('button', { name: /Delete/ }))

    const dialog = await screen.findByRole('dialog')
    expect(sent('DELETE', '/work-entries/')).toEqual([])
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }))

    expect(await screen.findByText('No hours logged this month.')).toBeTruthy()
    expect(sent('DELETE', '/work-entries/').map((request) => request.path)).toEqual([
      `/work-entries/${THIS_MONTH}-01`,
    ])
  })

  it('keeps the entry when the delete dialog is cancelled', async () => {
    entries = [entry(`${THIS_MONTH}-01`, 6)]
    renderWorkLog()
    await userEvent.click(await screen.findByRole('button', { name: /Delete/ }))

    await userEvent.click(
      within(await screen.findByRole('dialog')).getByRole('button', { name: 'Cancel' }),
    )

    expect(screen.queryByRole('dialog')).toBeNull()
    expect(sent('DELETE', '/work-entries/')).toEqual([])
  })

  it('switches to the empty state when the server reports missing work settings', async () => {
    putError = 'work_settings_missing'
    renderWorkLog()
    await userEvent.type(await screen.findByLabelText('Hours'), '8')

    await userEvent.click(screen.getByRole('button', { name: 'Save entry' }))

    expect(await screen.findByRole('link', { name: 'Open Settings' })).toBeTruthy()
    expect(screen.queryByLabelText('Hours')).toBeNull()
  })

  it('shows the translated error when saving fails for another reason', async () => {
    putError = 'service_unavailable'
    renderWorkLog()
    await userEvent.type(await screen.findByLabelText('Hours'), '8')

    await userEvent.click(screen.getByRole('button', { name: 'Save entry' }))

    expect((await screen.findByRole('alert')).textContent).toBe(
      'The service is temporarily unavailable. Please try again shortly.',
    )
    expect(screen.getByLabelText('Hours')).toBeTruthy()
  })

  it('moves between months with the previous and next buttons', async () => {
    renderWorkLog()
    await screen.findByText('No hours logged this month.')
    const previousMonth = addMonths(THIS_MONTH, -1)

    await userEvent.click(screen.getByRole('button', { name: 'Previous month' }))

    await vi.waitFor(() => {
      expect(requests.map((request) => request.path)).toContain(
        `/work-entries?month=${previousMonth}`,
      )
    })
    expect(
      (screen.getByRole('button', { name: 'Next month' }) as HTMLButtonElement).disabled,
    ).toBe(false)

    await userEvent.click(screen.getByRole('button', { name: 'Next month' }))

    expect(
      (screen.getByRole('button', { name: 'Next month' }) as HTMLButtonElement).disabled,
    ).toBe(true)
  })

  it('does not allow moving past the current month', async () => {
    renderWorkLog()
    await screen.findByText('No hours logged this month.')

    expect(
      (screen.getByRole('button', { name: 'Next month' }) as HTMLButtonElement).disabled,
    ).toBe(true)
  })

  it('opens the month given in the URL', async () => {
    entries = [entry('2026-01-15', 8)]
    renderWorkLog('/work-log?month=2026-01')

    expect(await screen.findByText('Equal to base')).toBeTruthy()
    expect(screen.getByRole('heading', { name: /January 2026/ })).toBeTruthy()
  })

  it('formats pay in the account currency', async () => {
    entries = [entry(`${THIS_MONTH}-01`, 1)]
    renderWorkLog()

    const row = (await screen.findAllByRole('row'))[1]

    // Vietnamese dong has no decimals and is shown with its symbol.
    expect(within(row).getByText(/30,000/)).toBeTruthy()
  })
})
