import { cleanup, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import viLocale from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import { addDays, formatLongDate, todayIn } from '../lib/dates'
import { FakeChecklist, habit, task } from '../test/fakeChecklist'
import { errorResponse, renderApp, signedIn, stubApi, USER } from '../test/testUtils'

const TODAY = todayIn(USER.timezone)
const YESTERDAY = addDays(TODAY, -1)
const TOMORROW = addDays(TODAY, 1)

let backend: FakeChecklist

/** Renders the Today screen against a fake backend and returns the requests it receives. */
function renderToday(path = '/') {
  const requests = stubApi(signedIn, backend.handle)
  renderApp(path)
  return requests
}

function habitsCard() {
  return within(screen.getByRole('region', { name: 'Habits' }))
}

function tasksCard() {
  return within(screen.getByRole('region', { name: 'To-dos' }))
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  backend = new FakeChecklist(
    [habit('habit-read', 'Read'), habit('habit-gym', 'Gym', { sort_order: 1 })],
    [task('task-milk', TODAY, 'Buy milk'), task('task-mom', TODAY, 'Call mom', true)],
  )
  backend.complete('habit-gym', TODAY)
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Today screen', () => {
  it('shows the date, the scheduled habits, and the to-dos of today', async () => {
    renderToday()

    expect(await screen.findByRole('checkbox', { name: 'Read' })).toHaveProperty('checked', false)
    expect(screen.getByRole('heading', { name: 'Today', level: 1 })).toBeTruthy()
    expect(screen.getByText(formatLongDate(TODAY, 'en'))).toBeTruthy()
    expect(screen.getByRole('checkbox', { name: 'Gym' })).toHaveProperty('checked', true)
    expect(screen.getByRole('checkbox', { name: 'Buy milk' })).toHaveProperty('checked', false)
    expect(screen.getByRole('checkbox', { name: 'Call mom' })).toHaveProperty('checked', true)
    expect(habitsCard().getByText('1 of 2 done')).toBeTruthy()
    expect(tasksCard().getByText('1 of 2 done')).toBeTruthy()
  })

  it('renders in Vietnamese', async () => {
    await i18n.changeLanguage('vi')

    renderToday()

    expect(
      await screen.findByRole('region', { name: viLocale.home.habits.title }),
    ).toBeTruthy()
    expect(screen.getByRole('region', { name: viLocale.home.tasks.title })).toBeTruthy()
    expect(screen.getByText(formatLongDate(TODAY, 'vi'))).toBeTruthy()
  })

  it('shows an error with a retry button when the day cannot be loaded', async () => {
    backend.failWhen = (request) => request.path.startsWith('/days/')
    renderToday()

    expect((await screen.findByRole('alert')).textContent).toBe(
      'The server ran into a problem. Please try again later.',
    )
    backend.failWhen = null
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }))

    expect(await screen.findByRole('checkbox', { name: 'Read' })).toBeTruthy()
  })

  it('points to the habits screen when no habit is scheduled', async () => {
    backend.habits = []

    renderToday()

    expect(await screen.findByText('No habits are scheduled for this day.')).toBeTruthy()
    expect(screen.getByRole('link', { name: 'Manage habits' }).getAttribute('href')).toBe('/habits')
  })
})

describe('day navigation', () => {
  it('loads the previous day and keeps it in the URL', async () => {
    backend.tasks.push(task('task-old', YESTERDAY, 'Old to-do'))
    const requests = renderToday()
    await screen.findByRole('checkbox', { name: 'Read' })

    await userEvent.click(screen.getByRole('link', { name: 'Previous day' }))

    expect(await screen.findByRole('checkbox', { name: 'Old to-do' })).toBeTruthy()
    expect(screen.getByText(formatLongDate(YESTERDAY, 'en'))).toBeTruthy()
    expect(requests.some((request) => request.path === `/days/${YESTERDAY}`)).toBe(true)
    expect(screen.queryByRole('checkbox', { name: 'Buy milk' })).toBeNull()
  })

  it('opens the day given in the URL and offers a way back to today', async () => {
    backend.tasks.push(task('task-plan', TOMORROW, 'Plan trip'))

    renderToday(`/?date=${TOMORROW}`)

    expect(await screen.findByRole('checkbox', { name: 'Plan trip' })).toBeTruthy()
    await userEvent.click(screen.getByRole('link', { name: 'Back to today' }))
    expect(await screen.findByRole('checkbox', { name: 'Buy milk' })).toBeTruthy()
    expect(screen.queryByRole('link', { name: 'Back to today' })).toBeNull()
  })

  it('falls back to today when the date in the URL is not a real date', async () => {
    const requests = renderToday('/?date=2026-02-30')

    expect(await screen.findByRole('checkbox', { name: 'Buy milk' })).toBeTruthy()
    expect(requests.some((request) => request.path === `/days/${TODAY}`)).toBe(true)
  })
})

describe('habits on the Today screen', () => {
  it('checks a habit immediately and saves the completion', async () => {
    const requests = renderToday()

    await userEvent.click(await screen.findByRole('checkbox', { name: 'Read' }))

    expect(screen.getByRole('checkbox', { name: 'Read' })).toHaveProperty('checked', true)
    await waitFor(() =>
      expect(
        requests.some(
          (request) =>
            request.method === 'PUT' &&
            request.path === `/habits/habit-read/completions/${TODAY}`,
        ),
      ).toBe(true),
    )
    expect(await habitsCard().findByText('2 of 2 done')).toBeTruthy()
  })

  it('unchecks a habit and removes the completion', async () => {
    const requests = renderToday()

    await userEvent.click(await screen.findByRole('checkbox', { name: 'Gym' }))

    expect(screen.getByRole('checkbox', { name: 'Gym' })).toHaveProperty('checked', false)
    await waitFor(() =>
      expect(
        requests.some(
          (request) =>
            request.method === 'DELETE' &&
            request.path === `/habits/habit-gym/completions/${TODAY}`,
        ),
      ).toBe(true),
    )
  })

  it('restores the previous state and reports the error when saving fails', async () => {
    backend.failWhen = (request) => request.method === 'PUT'
    renderToday()

    await userEvent.click(await screen.findByRole('checkbox', { name: 'Read' }))

    expect((await screen.findByRole('alert')).textContent).toBe(
      'The server ran into a problem. Please try again later.',
    )
    expect(screen.getByRole('checkbox', { name: 'Read' })).toHaveProperty('checked', false)
  })

  it('lists habits for a future day but does not let them be checked', async () => {
    const requests = renderToday(`/?date=${TOMORROW}`)

    const checkbox = await screen.findByRole('checkbox', { name: 'Read' })

    expect(checkbox).toHaveProperty('disabled', true)
    expect(screen.getByText('Habits cannot be checked off ahead of time.')).toBeTruthy()
    await userEvent.click(checkbox)
    expect(requests.some((request) => request.method === 'PUT')).toBe(false)
  })
})

describe('day mode', () => {
  const modeGroup = () => within(screen.getByRole('group', { name: 'Day mode' }))
  const lowEnergyButton = () => modeGroup().getByRole('button', { name: 'Low energy' })
  const normalButton = () => modeGroup().getByRole('button', { name: 'Normal' })

  beforeEach(() => {
    backend.completions.clear()
    backend.habits = [
      habit('habit-hidden', 'Gym'),
      habit('habit-kept', 'Run 5 km', {
        sort_order: 1,
        low_energy: true,
        low_energy_name: 'Walk 5 minutes',
      }),
    ]
  })

  it('shows the current mode', async () => {
    backend.lowEnergyDays.add(TODAY)

    renderToday()

    await screen.findByRole('group', { name: 'Day mode' })
    expect(lowEnergyButton().getAttribute('aria-pressed')).toBe('true')
    expect(normalButton().getAttribute('aria-pressed')).toBe('false')
  })

  it('switches today to low energy and shows only the kept habits under their easier names', async () => {
    const requests = renderToday()
    expect(await screen.findByRole('checkbox', { name: 'Gym' })).toBeTruthy()
    expect(screen.getByRole('checkbox', { name: 'Run 5 km' })).toBeTruthy()

    await userEvent.click(lowEnergyButton())

    expect(await screen.findByRole('checkbox', { name: 'Walk 5 minutes' })).toBeTruthy()
    expect(screen.queryByRole('checkbox', { name: 'Gym' })).toBeNull()
    expect(lowEnergyButton().getAttribute('aria-pressed')).toBe('true')
    const change = requests.find((request) => request.method === 'PUT')
    expect(change?.path).toBe(`/days/${TODAY}/mode`)
    expect(change?.body).toEqual({ mode: 'low_energy' })
  })

  it('switches back to a normal day', async () => {
    backend.lowEnergyDays.add(TODAY)
    const requests = renderToday()
    await screen.findByRole('checkbox', { name: 'Walk 5 minutes' })

    await userEvent.click(normalButton())

    expect(await screen.findByRole('checkbox', { name: 'Gym' })).toBeTruthy()
    expect(requests.find((request) => request.method === 'PUT')?.body).toEqual({ mode: 'normal' })
  })

  it('is disabled with an explanation while a habit is checked', async () => {
    backend.complete('habit-hidden', TODAY)
    const requests = renderToday()
    await screen.findByRole('checkbox', { name: 'Gym' })

    expect(lowEnergyButton()).toHaveProperty('disabled', true)
    expect(normalButton()).toHaveProperty('disabled', true)
    expect(screen.getByText('Uncheck all habits to change the mode.')).toBeTruthy()
    await userEvent.click(lowEnergyButton())
    expect(requests.some((request) => request.method === 'PUT')).toBe(false)
  })

  it('is disabled with an explanation on a day other than today', async () => {
    renderToday(`/?date=${YESTERDAY}`)
    await screen.findByRole('checkbox', { name: 'Gym' })

    expect(lowEnergyButton()).toHaveProperty('disabled', true)
    expect(screen.getByText('The mode can only be changed for today.')).toBeTruthy()
  })

  it('locks after a habit is checked and unlocks after it is unchecked', async () => {
    renderToday()
    await userEvent.click(await screen.findByRole('checkbox', { name: 'Gym' }))

    await waitFor(() => expect(lowEnergyButton()).toHaveProperty('disabled', true))
    await userEvent.click(screen.getByRole('checkbox', { name: 'Gym' }))

    await waitFor(() => expect(lowEnergyButton()).toHaveProperty('disabled', false))
  })

  it('reports a conflict from another tab and reloads the locked state', async () => {
    renderToday()
    await screen.findByRole('checkbox', { name: 'Gym' })
    // Another tab checks a habit after this screen loaded.
    backend.complete('habit-hidden', TODAY)

    await userEvent.click(lowEnergyButton())

    expect((await screen.findByRole('alert')).textContent).toBe(
      'The mode cannot be changed while a habit is checked. Uncheck your habits first.',
    )
    await waitFor(() => expect(lowEnergyButton()).toHaveProperty('disabled', true))
    expect(normalButton().getAttribute('aria-pressed')).toBe('true')
  })

  it('says so when no habit is kept for low-energy days', async () => {
    backend.habits = [habit('habit-hidden', 'Gym')]
    backend.lowEnergyDays.add(TODAY)

    renderToday()

    expect(await screen.findByText('No habits are kept for low-energy days.')).toBeTruthy()
  })

  it('renders in Vietnamese', async () => {
    await i18n.changeLanguage('vi')

    renderToday()

    const group = await screen.findByRole('group', { name: viLocale.home.mode.label })
    expect(within(group).getByRole('button', { name: viLocale.home.mode.lowEnergy })).toBeTruthy()
  })
})

describe('to-dos on the Today screen', () => {
  it('adds a to-do for the day being viewed', async () => {
    const requests = renderToday()
    const input = await screen.findByLabelText('New to-do')

    await userEvent.type(input, '  Water plants ')
    await userEvent.click(screen.getByRole('button', { name: 'Add' }))

    expect(await screen.findByRole('checkbox', { name: 'Water plants' })).toBeTruthy()
    await waitFor(() => expect((input as HTMLInputElement).value).toBe(''))
    const created = requests.find(
      (request) => request.method === 'POST' && request.path === '/tasks',
    )
    expect(created?.body).toEqual({ date: TODAY, title: 'Water plants' })
  })

  it('does not send a blank to-do', async () => {
    const requests = renderToday()
    const input = await screen.findByLabelText('New to-do')

    await userEvent.type(input, '   {Enter}')

    expect(requests.some((request) => request.path === '/tasks')).toBe(false)
  })

  it('rejects a title longer than 200 characters', async () => {
    const requests = renderToday()
    const input = await screen.findByLabelText('New to-do')

    await userEvent.click(input)
    await userEvent.paste('a'.repeat(201))
    await userEvent.keyboard('{Enter}')

    expect(await screen.findByText('Use at most 200 characters.')).toBeTruthy()
    expect(requests.some((request) => request.path === '/tasks')).toBe(false)
  })

  it('checks a to-do immediately and saves it', async () => {
    const requests = renderToday()

    await userEvent.click(await screen.findByRole('checkbox', { name: 'Buy milk' }))

    expect(screen.getByRole('checkbox', { name: 'Buy milk' })).toHaveProperty('checked', true)
    await waitFor(() => {
      const update = requests.find(
        (request) => request.method === 'PATCH' && request.path === '/tasks/task-milk',
      )
      expect(update?.body).toEqual({ is_done: true })
    })
  })

  it('restores a to-do when saving the check fails', async () => {
    backend.failWhen = (request) => request.method === 'PATCH'
    renderToday()

    await userEvent.click(await screen.findByRole('checkbox', { name: 'Buy milk' }))

    expect(await screen.findByRole('alert')).toBeTruthy()
    expect(screen.getByRole('checkbox', { name: 'Buy milk' })).toHaveProperty('checked', false)
  })

  it('renames a to-do in place', async () => {
    const requests = renderToday()

    await userEvent.click(await screen.findByRole('button', { name: 'Edit Buy milk' }))
    const input = screen.getByLabelText('To-do title')
    await userEvent.clear(input)
    await userEvent.type(input, 'Buy oat milk{Enter}')

    expect(await screen.findByRole('checkbox', { name: 'Buy oat milk' })).toBeTruthy()
    const update = requests.find(
      (request) => request.method === 'PATCH' && request.path === '/tasks/task-milk',
    )
    expect(update?.body).toEqual({ title: 'Buy oat milk' })
  })

  it('cancels a rename with Escape', async () => {
    const requests = renderToday()

    await userEvent.click(await screen.findByRole('button', { name: 'Edit Buy milk' }))
    await userEvent.type(screen.getByLabelText('To-do title'), ' and eggs{Escape}')

    expect(screen.getByRole('checkbox', { name: 'Buy milk' })).toBeTruthy()
    expect(requests.some((request) => request.method === 'PATCH')).toBe(false)
  })

  it('deletes a to-do', async () => {
    const requests = renderToday()

    await userEvent.click(await screen.findByRole('button', { name: 'Delete Buy milk' }))

    await waitFor(() => expect(screen.queryByRole('checkbox', { name: 'Buy milk' })).toBeNull())
    expect(
      requests.some(
        (request) => request.method === 'DELETE' && request.path === '/tasks/task-milk',
      ),
    ).toBe(true)
  })

  it('lets to-dos be planned for a future day', async () => {
    const requests = renderToday(`/?date=${TOMORROW}`)

    await userEvent.type(await screen.findByLabelText('New to-do'), 'Plan trip{Enter}')

    expect(await screen.findByRole('checkbox', { name: 'Plan trip' })).toBeTruthy()
    const created = requests.find((request) => request.path === '/tasks')
    expect(created?.body).toEqual({ date: TOMORROW, title: 'Plan trip' })
  })
})

describe('error handling', () => {
  it('translates an API error by its code instead of showing the backend message', async () => {
    stubApi({
      ...signedIn,
      [`GET /days/${TODAY}`]: () => errorResponse(503, 'service_unavailable'),
    })
    await i18n.changeLanguage('vi')

    renderApp('/')

    expect((await screen.findByRole('alert')).textContent).toBe(
      viLocale.errors.service_unavailable,
    )
    expect(screen.queryByText(/developer message/)).toBeNull()
  })
})
