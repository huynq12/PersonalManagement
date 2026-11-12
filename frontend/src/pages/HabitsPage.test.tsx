import { cleanup, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import viLocale from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import { FakeChecklist, habit } from '../test/fakeChecklist'
import { renderApp, signedIn, stubApi, type RecordedRequest } from '../test/testUtils'

let backend: FakeChecklist
let requests: RecordedRequest[]

function renderHabits() {
  requests = stubApi(signedIn, backend.handle)
  renderApp('/habits')
}

/** Requests sent to the habits endpoints with the given method. */
function habitRequests(method: string) {
  return requests.filter(
    (request) => request.method === method && request.path.startsWith('/habits'),
  )
}

function activeList() {
  return within(screen.getByRole('list', { name: 'Habits' }))
}

async function openDialog(buttonName: string, dialogName: string) {
  await userEvent.click(await screen.findByRole('button', { name: buttonName }))
  return within(screen.getByRole('dialog', { name: dialogName }))
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  backend = new FakeChecklist([
    habit('habit-read', 'Read', { current_streak: 5 }),
    habit('habit-gym', 'Gym', { sort_order: 1, weekdays: [1, 3, 5], current_streak: 1 }),
    habit('habit-old', 'Journal', { sort_order: 2, archived_at: '2026-11-01T00:00:00Z' }),
  ])
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Habits screen', () => {
  it('lists active habits in order with their schedule and streak', async () => {
    renderHabits()

    expect(await screen.findByRole('heading', { name: 'Habits', level: 1 })).toBeTruthy()
    const list = await screen.findByRole('list', { name: 'Habits' })
    const rows = within(list).getAllByRole('listitem')
    expect(rows).toHaveLength(2)
    expect(within(rows[0]).getByText('Read')).toBeTruthy()
    expect(within(rows[0]).getByText('Every day')).toBeTruthy()
    expect(within(rows[0]).getByText('5-day streak')).toBeTruthy()
    expect(within(rows[1]).getByText('Gym')).toBeTruthy()
    expect(within(rows[1]).getByText('Mon, Wed, Fri')).toBeTruthy()
    expect(within(rows[1]).getByText('1-day streak')).toBeTruthy()
    expect(activeList().queryByText('Journal')).toBeNull()
  })

  it('invites the user to add a habit when there are none', async () => {
    backend.habits = []

    renderHabits()

    expect(
      await screen.findByText('You have no habits yet. Add the first one to start a streak.'),
    ).toBeTruthy()
  })

  it('shows an error with a retry button when the habits cannot be loaded', async () => {
    backend.failWhen = (request) => request.path.startsWith('/habits')
    renderHabits()

    expect((await screen.findByRole('alert')).textContent).toBe(
      'The server ran into a problem. Please try again later.',
    )
    backend.failWhen = null
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }))

    expect(await activeList().findByText('Read')).toBeTruthy()
  })

  it('renders in Vietnamese', async () => {
    await i18n.changeLanguage('vi')

    renderHabits()

    expect(
      await screen.findByRole('heading', { name: viLocale.habits.title, level: 1 }),
    ).toBeTruthy()
    expect(await screen.findByText(viLocale.habits.everyDay)).toBeTruthy()
    expect(screen.getByRole('button', { name: viLocale.habits.add })).toBeTruthy()
  })
})

describe('adding a habit', () => {
  it('creates a habit for the chosen weekdays and closes the dialog', async () => {
    renderHabits()
    const dialog = await openDialog('Add habit', 'Add habit')

    await userEvent.type(dialog.getByLabelText('Name'), 'Meditate')
    await userEvent.click(dialog.getByRole('button', { name: 'Sat' }))
    await userEvent.click(dialog.getByRole('button', { name: 'Sun' }))
    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect(await activeList().findByText('Meditate')).toBeTruthy()
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(habitRequests('POST')[0].body).toEqual({ name: 'Meditate', weekdays: [1, 2, 3, 4, 5] })
  })

  it('starts with every weekday selected', async () => {
    renderHabits()
    const dialog = await openDialog('Add habit', 'Add habit')

    const days = within(dialog.getByRole('group', { name: 'Repeat on' })).getAllByRole('button')

    expect(days.map((day) => day.textContent)).toEqual([
      'Mon',
      'Tue',
      'Wed',
      'Thu',
      'Fri',
      'Sat',
      'Sun',
    ])
    expect(days.every((day) => day.getAttribute('aria-pressed') === 'true')).toBe(true)
  })

  it('requires a name', async () => {
    renderHabits()
    const dialog = await openDialog('Add habit', 'Add habit')

    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect(await dialog.findByText('This field is required.')).toBeTruthy()
    expect(habitRequests('POST')).toHaveLength(0)
  })

  it('requires at least one weekday', async () => {
    renderHabits()
    const dialog = await openDialog('Add habit', 'Add habit')
    await userEvent.type(dialog.getByLabelText('Name'), 'Meditate')

    for (const day of ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']) {
      await userEvent.click(dialog.getByRole('button', { name: day }))
    }
    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect(await dialog.findByText('Choose at least one day.')).toBeTruthy()
    expect(habitRequests('POST')).toHaveLength(0)
  })

  it('closes without saving on Escape and on Cancel', async () => {
    renderHabits()
    const first = await openDialog('Add habit', 'Add habit')
    await userEvent.type(first.getByLabelText('Name'), 'Meditate{Escape}')
    expect(screen.queryByRole('dialog')).toBeNull()

    const second = await openDialog('Add habit', 'Add habit')
    expect((second.getByLabelText('Name') as HTMLInputElement).value).toBe('')
    await userEvent.click(second.getByRole('button', { name: 'Cancel' }))

    expect(screen.queryByRole('dialog')).toBeNull()
    expect(habitRequests('POST')).toHaveLength(0)
  })

  it('keeps the dialog open and reports the error when saving fails', async () => {
    renderHabits()
    const dialog = await openDialog('Add habit', 'Add habit')
    backend.failWhen = (request) => request.method === 'POST' && request.path === '/habits'

    await userEvent.type(dialog.getByLabelText('Name'), 'Meditate')
    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect((await dialog.findByRole('alert')).textContent).toBe(
      'The server ran into a problem. Please try again later.',
    )
    expect((dialog.getByLabelText('Name') as HTMLInputElement).value).toBe('Meditate')
  })
})

describe('editing a habit', () => {
  it('opens with the current values and saves the changes', async () => {
    renderHabits()
    const dialog = await openDialog('Edit Gym', 'Edit habit')
    const name = dialog.getByLabelText('Name') as HTMLInputElement

    expect(name.value).toBe('Gym')
    expect(dialog.getByRole('button', { name: 'Mon' }).getAttribute('aria-pressed')).toBe('true')
    expect(dialog.getByRole('button', { name: 'Tue' }).getAttribute('aria-pressed')).toBe('false')

    await userEvent.clear(name)
    await userEvent.type(name, 'Climbing')
    await userEvent.click(dialog.getByRole('button', { name: 'Tue' }))
    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect(await activeList().findByText('Climbing')).toBeTruthy()
    expect(await activeList().findByText('Mon, Tue, Wed, Fri')).toBeTruthy()
    const [update] = habitRequests('PATCH')
    expect(update.path).toBe('/habits/habit-gym')
    expect(update.body).toEqual({ name: 'Climbing', weekdays: [1, 2, 3, 5] })
  })
})

describe('archiving', () => {
  it('moves a habit to the archived section', async () => {
    renderHabits()

    await userEvent.click(await screen.findByRole('button', { name: 'Archive Read' }))

    expect(await screen.findByRole('button', { name: 'Archived (2)' })).toBeTruthy()
    expect(activeList().queryByText('Read')).toBeNull()
    expect(habitRequests('PATCH')[0].body).toEqual({ archived: true })
  })

  it('keeps archived habits collapsed until asked and restores one', async () => {
    renderHabits()
    const toggle = await screen.findByRole('button', { name: 'Archived (1)' })
    expect(screen.queryByText('Journal')).toBeNull()

    await userEvent.click(toggle)
    await userEvent.click(screen.getByRole('button', { name: 'Restore Journal' }))

    expect(await activeList().findByText('Journal')).toBeTruthy()
    const [update] = habitRequests('PATCH')
    expect(update.path).toBe('/habits/habit-old')
    expect(update.body).toEqual({ archived: false })
  })
})

describe('deleting', () => {
  it('asks for confirmation and does nothing when cancelled', async () => {
    renderHabits()
    const dialog = await openDialog('Delete Read', 'Delete this habit?')

    expect(dialog.getByText(/"Read" and its whole history will be deleted permanently/)).toBeTruthy()
    await userEvent.click(dialog.getByRole('button', { name: 'Cancel' }))

    expect(screen.queryByRole('dialog')).toBeNull()
    expect(activeList().getByText('Read')).toBeTruthy()
    expect(habitRequests('DELETE')).toHaveLength(0)
  })

  it('deletes the habit after confirmation', async () => {
    renderHabits()
    const dialog = await openDialog('Delete Read', 'Delete this habit?')

    await userEvent.click(dialog.getByRole('button', { name: 'Delete permanently' }))

    await waitFor(() => expect(activeList().queryByText('Read')).toBeNull())
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(habitRequests('DELETE')[0].path).toBe('/habits/habit-read')
  })
})
