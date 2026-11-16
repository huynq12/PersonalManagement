import { cleanup, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import vi_ from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import { addMonths, formatLongDate, formatShortDate, todayIn } from '../lib/dates'
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
    {
      ...signedIn,
      'GET /settings': () => jsonResponse(200, settings),
      'PATCH /settings': (request) => {
        settings = { ...settings, ...(request.body as Partial<Settings>) }
        return jsonResponse(200, settings)
      },
    },
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

/** The calendar button of a day, found by its accessible name that starts with the short date. */
async function day(date: string) {
  const prefix = `${formatShortDate(date, 'en')}:`
  return screen.findByRole('button', {
    name: (name) => name.startsWith(prefix),
  })
}

/** The Save button of the open dialog; the Work settings card has a Save button of its own. */
function saveButton() {
  return within(screen.getByRole('dialog')).getByRole('button', { name: 'Save' })
}

/** The Work settings card at the bottom of the page. */
async function workSettings() {
  return within(await screen.findByRole('region', { name: 'Work' }))
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
  it('shows the month as a calendar with a cell for every day', async () => {
    renderWorkLog()

    const grid = await screen.findByRole('grid')

    expect(within(grid).getAllByRole('columnheader')).toHaveLength(7)
    expect(await day(TODAY)).toBeTruthy()
    expect(
      screen.getByRole('heading', { name: new RegExp(String(Number(TODAY.slice(0, 4)))) }),
    ).toBeTruthy()
  })

  it('shows hours and pay on the day of an entry and the month totals', async () => {
    entries = [entry(`${THIS_MONTH}-01`, 9, { note: 'Busy day' }), entry(`${THIS_MONTH}-02`, 6.5)]
    renderWorkLog()

    const first = await day(`${THIS_MONTH}-01`)

    expect(first.getAttribute('data-status')).toBe('above')
    expect(first.textContent).toMatch(/270,000/)
    expect(first.textContent).toContain('Busy day')
    expect((await day(`${THIS_MONTH}-02`)).getAttribute('data-status')).toBe('below')
    const totals = within(screen.getByLabelText('Month totals'))
    expect(totals.getByText('15.5')).toBeTruthy()
    expect(totals.getByText(/465,000/)).toBeTruthy()
    expect(totals.getByText('2 days')).toBeTruthy()
  })

  it('shows the work settings card below the calendar and no settings link', async () => {
    renderWorkLog()

    const work = await workSettings()

    expect((work.getByLabelText('Hourly wage (VND)') as HTMLInputElement).value).toBe('30000')
    expect((work.getByLabelText('Base hours per day') as HTMLInputElement).value).toBe('8')
    expect(screen.queryByRole('link', { name: 'Open Settings' })).toBeNull()
  })

  describe('logging hours for a day', () => {
    it('opens a dialog for the clicked day and saves the hours for that date', async () => {
      renderWorkLog()
      await userEvent.click(await day(TODAY))

      const dialog = await screen.findByRole('dialog', { name: formatLongDate(TODAY, 'en') })
      expect(within(dialog).queryByLabelText('Date')).toBeNull()
      await userEvent.type(within(dialog).getByLabelText('Hours'), '7,5')
      await userEvent.type(within(dialog).getByLabelText('Note'), 'Inventory')
      await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))

      await vi.waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
      expect(sent('PUT', '/work-entries/').map((request) => [request.path, request.body])).toEqual([
        [`/work-entries/${TODAY}`, { hours: 7.5, note: 'Inventory' }],
      ])
      expect((await day(TODAY)).getAttribute('data-status')).toBe('below')
    })

    it('sends no note when the note is left empty', async () => {
      renderWorkLog()
      await userEvent.click(await day(TODAY))
      await userEvent.type(await screen.findByLabelText('Hours'), '8')

      await userEvent.click(saveButton())

      await vi.waitFor(() => expect(sent('PUT', '/work-entries/')).toHaveLength(1))
      expect(sent('PUT', '/work-entries/')[0].body).toEqual({ hours: 8, note: null })
    })

    it('does not send invalid hours and shows the validation message', async () => {
      renderWorkLog()
      await userEvent.click(await day(TODAY))
      await userEvent.type(await screen.findByLabelText('Hours'), '25')

      await userEvent.click(saveButton())

      expect(
        await screen.findByText(
          'Enter hours greater than 0 and at most 24, with up to two decimals.',
        ),
      ).toBeTruthy()
      expect(sent('PUT', '/work-entries/')).toEqual([])
      expect(screen.getByRole('dialog')).toBeTruthy()
    })

    it('closes the dialog with Cancel without saving', async () => {
      renderWorkLog()
      await userEvent.click(await day(TODAY))

      await userEvent.click(
        within(await screen.findByRole('dialog')).getByRole('button', { name: 'Cancel' }),
      )

      expect(screen.queryByRole('dialog')).toBeNull()
      expect(sent('PUT', '/work-entries/')).toEqual([])
    })

    it('shows the translated error inside the dialog when saving fails', async () => {
      putError = 'service_unavailable'
      renderWorkLog()
      await userEvent.click(await day(TODAY))
      await userEvent.type(await screen.findByLabelText('Hours'), '8')

      await userEvent.click(saveButton())

      expect(
        await within(screen.getByRole('dialog')).findByText(
          'The service is temporarily unavailable. Please try again shortly.',
        ),
      ).toBeTruthy()
    })

    it('does not offer a delete button for a day without an entry', async () => {
      renderWorkLog()
      await userEvent.click(await day(TODAY))

      const dialog = await screen.findByRole('dialog')

      expect(within(dialog).queryByRole('button', { name: 'Delete' })).toBeNull()
    })

    it('does not open anything when a future day is clicked', async () => {
      renderWorkLog()
      const tomorrow = new Date(`${TODAY}T00:00:00Z`)
      tomorrow.setUTCDate(tomorrow.getUTCDate() + 1)
      const next = tomorrow.toISOString().slice(0, 10)
      if (!next.startsWith(THIS_MONTH)) {
        // Today is the last day of the month: there is no future day in view.
        return
      }

      await userEvent.click(await day(next))

      expect(screen.queryByRole('dialog')).toBeNull()
    })
  })

  describe('editing and deleting an entry', () => {
    it('opens the existing entry prefilled and saves it for the same date', async () => {
      entries = [entry(TODAY, 6, { note: 'Short day' })]
      renderWorkLog()
      await userEvent.click(await day(TODAY))

      const dialog = await screen.findByRole('dialog')
      expect((within(dialog).getByLabelText('Hours') as HTMLInputElement).value).toBe('6')
      expect((within(dialog).getByLabelText('Note') as HTMLInputElement).value).toBe('Short day')
      await userEvent.clear(within(dialog).getByLabelText('Hours'))
      await userEvent.type(within(dialog).getByLabelText('Hours'), '9')
      await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))

      await vi.waitFor(async () =>
        expect((await day(TODAY)).getAttribute('data-status')).toBe('above'),
      )
      expect(sent('PUT', '/work-entries/').map((request) => request.path)).toEqual([
        `/work-entries/${TODAY}`,
      ])
    })

    it('shows the wage and base hours kept with the entry', async () => {
      entries = [entry(TODAY, 6)]
      renderWorkLog()
      await userEvent.click(await day(TODAY))

      const dialog = await screen.findByRole('dialog')

      expect(within(dialog).getByText(/30,000/)).toBeTruthy()
      expect(within(dialog).getByText('Below base')).toBeTruthy()
    })

    it('asks for confirmation before deleting an entry', async () => {
      entries = [entry(TODAY, 6)]
      renderWorkLog()
      await userEvent.click(await day(TODAY))
      await userEvent.click(
        within(await screen.findByRole('dialog')).getByRole('button', { name: 'Delete' }),
      )

      const confirm = await screen.findByRole('dialog', { name: 'Delete this entry?' })
      expect(sent('DELETE', '/work-entries/')).toEqual([])
      await userEvent.click(within(confirm).getByRole('button', { name: 'Delete' }))

      await vi.waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
      expect(sent('DELETE', '/work-entries/').map((request) => request.path)).toEqual([
        `/work-entries/${TODAY}`,
      ])
      expect((await day(TODAY)).getAttribute('data-status')).toBe('none')
    })

    it('keeps the entry when the delete confirmation is cancelled', async () => {
      entries = [entry(TODAY, 6)]
      renderWorkLog()
      await userEvent.click(await day(TODAY))
      await userEvent.click(
        within(await screen.findByRole('dialog')).getByRole('button', { name: 'Delete' }),
      )

      await userEvent.click(
        within(await screen.findByRole('dialog')).getByRole('button', { name: 'Cancel' }),
      )

      expect(screen.queryByRole('dialog')).toBeNull()
      expect(sent('DELETE', '/work-entries/')).toEqual([])
      expect((await day(TODAY)).getAttribute('data-status')).toBe('below')
    })
  })

  describe('without a wage or base hours', () => {
    it('explains where to set them and links to the settings card on the same page', async () => {
      settings = { ...SETTINGS, hourly_wage: null }
      renderWorkLog()

      const link = await screen.findByRole('link', { name: 'Go to work settings' })

      expect(link.getAttribute('href')).toBe('#work-settings')
      expect(document.getElementById('work-settings')).toBeTruthy()
    })

    it('shows the notice when only the base hours are missing', async () => {
      settings = { ...SETTINGS, base_work_hours: null }
      renderWorkLog()

      expect(await screen.findByRole('link', { name: 'Go to work settings' })).toBeTruthy()
    })

    it('scrolls to the settings card instead of opening the entry dialog', async () => {
      settings = { ...SETTINGS, hourly_wage: null }
      const scrollIntoView = vi.fn()
      Element.prototype.scrollIntoView = scrollIntoView
      renderWorkLog()
      await screen.findByRole('link', { name: 'Go to work settings' })

      await userEvent.click(await day(TODAY))

      expect(screen.queryByRole('dialog')).toBeNull()
      expect(scrollIntoView).toHaveBeenCalled()
    })

    it('switches to the notice when the server reports missing work settings', async () => {
      putError = 'work_settings_missing'
      renderWorkLog()
      await userEvent.click(await day(TODAY))
      await userEvent.type(await screen.findByLabelText('Hours'), '8')

      await userEvent.click(saveButton())

      expect(await screen.findByRole('link', { name: 'Go to work settings' })).toBeTruthy()
      expect(screen.queryByRole('dialog')).toBeNull()
    })
  })

  describe('work settings card', () => {
    it('saves the work settings in minor units and then allows logging without a reload', async () => {
      settings = { ...SETTINGS, hourly_wage: null, base_work_hours: null }
      renderWorkLog()
      const work = await workSettings()
      await screen.findByRole('link', { name: 'Go to work settings' })

      await userEvent.type(work.getByLabelText('Hourly wage (VND)'), '30000')
      await userEvent.type(work.getByLabelText('Base hours per day'), '7.5')
      await userEvent.click(work.getByRole('button', { name: 'Save' }))

      expect(await work.findByText('Saved.')).toBeTruthy()
      expect(
        requests.filter((request) => request.method === 'PATCH').map((request) => request.body),
      ).toEqual([{ hourly_wage: 30000, base_work_hours: 7.5 }])
      await vi.waitFor(() =>
        expect(screen.queryByRole('link', { name: 'Go to work settings' })).toBeNull(),
      )
      await userEvent.click(await day(TODAY))
      expect(await screen.findByRole('dialog')).toBeTruthy()
    })

    it('rejects more decimals than the currency has without calling the server', async () => {
      renderWorkLog()
      const work = await workSettings()

      await userEvent.clear(work.getByLabelText('Hourly wage (VND)'))
      await userEvent.type(work.getByLabelText('Hourly wage (VND)'), '100.5')
      await userEvent.click(work.getByRole('button', { name: 'Save' }))

      expect(await work.findByText(/Enter an amount greater than 0/)).toBeTruthy()
      expect(requests.filter((request) => request.method === 'PATCH')).toHaveLength(0)
    })

    it('rejects base hours above 24', async () => {
      renderWorkLog()
      const work = await workSettings()

      await userEvent.clear(work.getByLabelText('Base hours per day'))
      await userEvent.type(work.getByLabelText('Base hours per day'), '25')
      await userEvent.click(work.getByRole('button', { name: 'Save' }))

      expect(await work.findByText(/Enter hours greater than 0/)).toBeTruthy()
      expect(requests.filter((request) => request.method === 'PATCH')).toHaveLength(0)
    })

    it('shows an error when saving fails', async () => {
      renderWorkLog()
      const work = await workSettings()
      await screen.findByRole('grid')
      stubApi({}, () => errorResponse(422, 'validation_failed'))

      await userEvent.clear(work.getByLabelText('Hourly wage (VND)'))
      await userEvent.type(work.getByLabelText('Hourly wage (VND)'), '100')
      await userEvent.click(work.getByRole('button', { name: 'Save' }))

      expect((await work.findByRole('alert')).textContent).toBe(
        'Some fields are not valid. Please check them and try again.',
      )
    })
  })

  describe('months', () => {
    it('moves between months with the previous and next buttons', async () => {
      renderWorkLog()
      await screen.findByRole('grid')
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
      await screen.findByRole('grid')

      expect(
        (screen.getByRole('button', { name: 'Next month' }) as HTMLButtonElement).disabled,
      ).toBe(true)
    })

    it('makes every day of a past month clickable', async () => {
      entries = [entry('2026-01-15', 8)]
      renderWorkLog('/work-log?month=2026-01')

      const grid = await screen.findByRole('grid')

      expect(within(grid).getAllByRole('button')).toHaveLength(31)
      expect(within(grid).getAllByRole('button').some((cell) => cell.hasAttribute('disabled'))).toBe(
        false,
      )
    })

    it('opens the month given in the URL', async () => {
      entries = [entry('2026-01-15', 8)]
      renderWorkLog('/work-log?month=2026-01')

      expect((await day('2026-01-15')).getAttribute('data-status')).toBe('equal')
      expect(screen.getByRole('heading', { name: /January 2026/ })).toBeTruthy()
    })
  })

  it('formats pay in the account currency', async () => {
    entries = [entry(`${THIS_MONTH}-01`, 1)]
    renderWorkLog()

    // Vietnamese dong has no decimals and is shown with its symbol.
    expect((await day(`${THIS_MONTH}-01`)).textContent).toMatch(/30,000/)
  })

  it('renders in Vietnamese', async () => {
    await i18n.changeLanguage('vi')
    renderWorkLog()

    expect(await screen.findByRole('heading', { name: vi_.workLog.title, level: 1 })).toBeTruthy()
    expect(await screen.findByRole('region', { name: vi_.settings.work.title })).toBeTruthy()
  })
})
