import { cleanup, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import viLocale from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import { FakeSubjects, subject } from '../test/fakeSubjects'
import { renderApp, signedIn, stubApi, type RecordedRequest } from '../test/testUtils'

let backend: FakeSubjects
let requests: RecordedRequest[]

function renderSubjects() {
  requests = stubApi(signedIn, backend.handle)
  renderApp('/subjects')
}

/** Requests sent to the subjects endpoints with the given method. */
function subjectRequests(method: string) {
  return requests.filter(
    (request) => request.method === method && request.path.startsWith('/subjects'),
  )
}

function activeList() {
  return within(screen.getByRole('list', { name: 'Subjects' }))
}

async function openDialog(buttonName: string, dialogName: string) {
  await userEvent.click(await screen.findByRole('button', { name: buttonName }))
  return within(screen.getByRole('dialog', { name: dialogName }))
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  backend = new FakeSubjects([
    subject('subject-math', 'Math', { color: '#3B82F6' }),
    subject('subject-art', 'art', { color: '#EC4899' }),
    subject('subject-old', 'Latin', { archived_at: '2026-11-01T00:00:00Z' }),
  ])
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Subjects screen', () => {
  it('lists active subjects by name and keeps archived ones out of the list', async () => {
    renderSubjects()

    expect(await screen.findByRole('heading', { name: 'Subjects', level: 1 })).toBeTruthy()
    const list = await screen.findByRole('list', { name: 'Subjects' })
    const rows = within(list).getAllByRole('listitem')
    expect(rows.map((row) => row.textContent)).toEqual(['art', 'Math'])
    expect(activeList().queryByText('Latin')).toBeNull()
  })

  it('invites the user to add a subject when there are none', async () => {
    backend.subjects = []

    renderSubjects()

    expect(
      await screen.findByText('You have no subjects yet. Add one to start tracking study time.'),
    ).toBeTruthy()
  })

  it('shows an error with a retry button when the subjects cannot be loaded', async () => {
    backend.failWhen = (request) => request.path.startsWith('/subjects')
    renderSubjects()

    expect((await screen.findByRole('alert')).textContent).toBe(
      'The server ran into a problem. Please try again later.',
    )
    backend.failWhen = null
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }))

    expect(await activeList().findByText('Math')).toBeTruthy()
  })

  it('renders in Vietnamese', async () => {
    await i18n.changeLanguage('vi')

    renderSubjects()

    expect(
      await screen.findByRole('heading', { name: viLocale.subjects.title, level: 1 }),
    ).toBeTruthy()
    expect(screen.getByRole('button', { name: viLocale.subjects.add })).toBeTruthy()
  })
})

describe('adding a subject', () => {
  it('creates a subject with the chosen color and closes the dialog', async () => {
    renderSubjects()
    const dialog = await openDialog('Add subject', 'Add subject')

    await userEvent.type(dialog.getByLabelText('Name'), 'Physics')
    await userEvent.click(dialog.getByRole('radio', { name: 'Green' }))
    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect(await activeList().findByText('Physics')).toBeTruthy()
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(subjectRequests('POST')[0].body).toEqual({ name: 'Physics', color: '#22C55E' })
  })

  it('requires a name', async () => {
    renderSubjects()
    const dialog = await openDialog('Add subject', 'Add subject')

    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect(await dialog.findByText('This field is required.')).toBeTruthy()
    expect(subjectRequests('POST')).toHaveLength(0)
  })

  it('tells the user when the name is already taken', async () => {
    renderSubjects()
    const dialog = await openDialog('Add subject', 'Add subject')

    await userEvent.type(dialog.getByLabelText('Name'), 'MATH')
    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect((await dialog.findByRole('alert')).textContent).toBe(
      'You already have a subject with this name.',
    )
    expect(screen.getByRole('dialog')).toBeTruthy()
  })
})

describe('editing a subject', () => {
  it('shows the current values and saves the changes', async () => {
    renderSubjects()
    await userEvent.click(await screen.findByRole('button', { name: 'Edit Math' }))
    const dialog = within(screen.getByRole('dialog', { name: 'Edit subject' }))

    expect((dialog.getByLabelText('Name') as HTMLInputElement).value).toBe('Math')
    expect((dialog.getByRole('radio', { name: 'Blue' }) as HTMLInputElement).checked).toBe(true)
    await userEvent.clear(dialog.getByLabelText('Name'))
    await userEvent.type(dialog.getByLabelText('Name'), 'Calculus')
    await userEvent.click(dialog.getByRole('button', { name: 'Save' }))

    expect(await activeList().findByText('Calculus')).toBeTruthy()
    expect(subjectRequests('PATCH')[0].body).toEqual({ name: 'Calculus', color: '#3B82F6' })
  })
})

describe('archiving a subject', () => {
  it('moves it to the archived section and back', async () => {
    renderSubjects()
    await userEvent.click(await screen.findByRole('button', { name: 'Archive Math' }))

    await waitFor(() => expect(activeList().queryByText('Math')).toBeNull())
    expect(subjectRequests('PATCH')[0].body).toEqual({ archived: true })

    await userEvent.click(screen.getByRole('button', { name: 'Archived (2)' }))
    await userEvent.click(screen.getByRole('button', { name: 'Restore Math' }))

    expect(await activeList().findByText('Math')).toBeTruthy()
    expect(subjectRequests('PATCH')[1].body).toEqual({ archived: false })
  })
})

describe('deleting a subject', () => {
  it('asks first, then deletes it', async () => {
    renderSubjects()
    await userEvent.click(await screen.findByRole('button', { name: 'Delete Math' }))
    const dialog = within(screen.getByRole('dialog', { name: 'Delete this subject?' }))
    expect(subjectRequests('DELETE')).toHaveLength(0)

    await userEvent.click(dialog.getByRole('button', { name: 'Delete' }))

    await waitFor(() => expect(activeList().queryByText('Math')).toBeNull())
    expect(subjectRequests('DELETE')[0].path).toBe('/subjects/subject-math')
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('keeps the subject when the user cancels', async () => {
    renderSubjects()
    await userEvent.click(await screen.findByRole('button', { name: 'Delete Math' }))

    await userEvent.click(
      within(screen.getByRole('dialog')).getByRole('button', { name: 'Cancel' }),
    )

    expect(screen.queryByRole('dialog')).toBeNull()
    expect(subjectRequests('DELETE')).toHaveLength(0)
    expect(activeList().getByText('Math')).toBeTruthy()
  })
})
