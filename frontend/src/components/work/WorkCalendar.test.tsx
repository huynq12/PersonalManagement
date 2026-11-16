import { cleanup, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../../i18n/i18n'
import type { WorkEntry, WorkMonth } from '../../lib/workEntries'
import WorkCalendar from './WorkCalendar'

function entry(date: string, hours: number, overrides: Partial<WorkEntry> = {}): WorkEntry {
  const base = 6
  return {
    date,
    hours,
    hourly_wage: 12400,
    base_hours: base,
    pay: Math.round(hours * 12400),
    comparison: hours > base ? 'above' : hours < base ? 'below' : 'equal',
    note: null,
    ...overrides,
  }
}

const BELOW = entry('2026-11-16', 5, { note: 'Morning shift' })
const EQUAL = entry('2026-11-14', 6)
const ABOVE = entry('2026-11-12', 7)

const MONTH: WorkMonth = {
  entries: [ABOVE, EQUAL, BELOW],
  summary: { total_hours: 18, total_pay: 223200, days_worked: 3 },
}

function renderCalendar(onSelectDay = vi.fn()) {
  render(
    <WorkCalendar
      month={MONTH}
      monthKey="2026-11"
      today="2026-11-17"
      currency="KRW"
      onSelectDay={onSelectDay}
    />,
  )
  return onSelectDay
}

const cell = (name: RegExp) => screen.getByRole('button', { name })

beforeEach(async () => {
  await i18n.changeLanguage('en')
})

afterEach(cleanup)

describe('WorkCalendar', () => {
  it('shows the weekday names Monday first', () => {
    renderCalendar()

    const headers = screen.getAllByRole('columnheader').map((header) => header.textContent)

    expect(headers).toEqual(['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'])
  })

  it('shows hours and pay of a day with an entry, below the base hours', () => {
    renderCalendar()

    const day = cell(/^Mon, Nov 16: 5 hours, .*62,000.*, Below base$/)

    expect(day.getAttribute('data-status')).toBe('below')
    expect(day.textContent).toContain('5')
    expect(day.textContent).toMatch(/62,000/)
    expect(day.textContent).toContain('Morning shift')
  })

  it('marks a day equal to the base hours and a day above it', () => {
    renderCalendar()

    expect(cell(/^Sat, Nov 14: 6 hours/).getAttribute('data-status')).toBe('equal')
    expect(cell(/^Thu, Nov 12: 7 hours/).getAttribute('data-status')).toBe('above')
  })

  it('shows zero pay on a past day without an entry', () => {
    renderCalendar()

    const day = cell(/^Sun, Nov 15: no hours logged$/)

    expect(day.getAttribute('data-status')).toBe('none')
    expect(day.textContent).toMatch(/0/)
  })

  it('shows today as a day without an entry that is highlighted', () => {
    renderCalendar()

    const today = cell(/^Tue, Nov 17: no hours logged$/)

    expect(today.getAttribute('aria-current')).toBe('date')
    expect(today.hasAttribute('disabled')).toBe(false)
  })

  it('disables future days', async () => {
    const onSelectDay = renderCalendar()

    const future = screen.getByRole('button', { name: /^Wed, Nov 18: not available yet$/ })
    await userEvent.click(future)

    expect(future.hasAttribute('disabled')).toBe(true)
    expect(future.getAttribute('data-status')).toBe('future')
    expect(onSelectDay).not.toHaveBeenCalled()
  })

  it('has one button per day of the month and none for the padding cells', () => {
    renderCalendar()

    // 30 days in November; the six leading cells are empty.
    expect(screen.getAllByRole('button')).toHaveLength(30)
  })

  it('reports the date and entry of a clicked day', async () => {
    const onSelectDay = renderCalendar()

    await userEvent.click(cell(/^Mon, Nov 16:/))

    expect(onSelectDay).toHaveBeenCalledWith('2026-11-16', BELOW)
  })

  it('reports no entry when a day without one is clicked', async () => {
    const onSelectDay = renderCalendar()

    await userEvent.click(cell(/^Sun, Nov 15:/))

    expect(onSelectDay).toHaveBeenCalledWith('2026-11-15', null)
  })
})
