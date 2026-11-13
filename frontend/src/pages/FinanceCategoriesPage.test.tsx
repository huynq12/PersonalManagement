import { cleanup, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import vi_ from '../i18n/locales/vi.json'
import { setAccessToken } from '../lib/api'
import type { Category } from '../lib/finance'
import {
  errorResponse,
  jsonResponse,
  renderApp,
  signedIn,
  stubApi,
  type RecordedRequest,
} from '../test/testUtils'

let categories: Category[]
let requests: RecordedRequest[]
let nextId: number
/** Names for which `POST /finance-categories` answers 409, like a retry after a partial run. */
let takenNames: string[]

function category(kind: Category['kind'], name: string, archived = false): Category {
  nextId += 1
  return {
    id: `category-${nextId}`,
    kind,
    name,
    color: '#3B82F6',
    archived_at: archived ? '2026-01-01T00:00:00Z' : null,
  }
}

function renderCategories() {
  requests = stubApi({ ...signedIn }, (request) => {
    const url = new URL(request.path, 'http://localhost')
    if (url.pathname === '/finance-categories' && request.method === 'GET') {
      return jsonResponse(200, categories)
    }
    if (url.pathname === '/finance-categories' && request.method === 'POST') {
      const body = request.body as { kind: Category['kind']; name: string; color: string }
      if (takenNames.includes(body.name)) {
        return errorResponse(409, 'finance_category_name_taken')
      }
      const created = { ...category(body.kind, body.name), color: body.color }
      categories = [...categories, created]
      return jsonResponse(201, created)
    }
    const match = /^\/finance-categories\/(.+)$/.exec(url.pathname)
    if (match && request.method === 'PATCH') {
      const changes = request.body as { name?: string; color?: string; archived?: boolean }
      categories = categories.map((item) =>
        item.id === match[1]
          ? {
              ...item,
              name: changes.name ?? item.name,
              color: changes.color ?? item.color,
              archived_at:
                changes.archived === undefined
                  ? item.archived_at
                  : changes.archived
                    ? '2026-01-01T00:00:00Z'
                    : null,
            }
          : item,
      )
      return jsonResponse(
        200,
        categories.find((item) => item.id === match[1]),
      )
    }
    return undefined
  })
  renderApp('/finance-categories')
}

function posted() {
  return requests
    .filter((request) => request.method === 'POST' && request.path === '/finance-categories')
    .map((request) => request.body as { kind: string; name: string })
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  categories = []
  nextId = 0
  takenNames = []
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Finance categories screen', () => {
  it('offers the suggested set while the user has no category', async () => {
    renderCategories()

    expect(await screen.findByRole('button', { name: 'Add suggested categories' })).toBeTruthy()
  })

  it('hides the suggested set once any category exists, archived ones included', async () => {
    categories = [category('expense', 'Food', true)]
    renderCategories()

    await screen.findByRole('heading', { name: 'Expense categories' })

    expect(screen.queryByRole('button', { name: 'Add suggested categories' })).toBeNull()
  })

  it('creates the suggested categories in the active language', async () => {
    await i18n.changeLanguage('vi')
    renderCategories()

    await userEvent.click(
      await screen.findByRole('button', { name: vi_.financeCategories.suggested.button }),
    )

    expect(await screen.findByText(vi_.finances.suggestedCategories.income.salary)).toBeTruthy()
    const names = posted().map((body) => `${body.kind}:${body.name}`)
    expect(names).toHaveLength(12)
    expect(names[0]).toBe(`expense:${vi_.finances.suggestedCategories.expense.food}`)
    expect(names).toContain(`income:${vi_.finances.suggestedCategories.income.bonus}`)
  })

  it('ignores a name that already exists so a retry is safe', async () => {
    takenNames = ['Food']
    renderCategories()

    await userEvent.click(await screen.findByRole('button', { name: 'Add suggested categories' }))

    expect(await screen.findByText('Transport')).toBeTruthy()
    expect(screen.queryByText('You already have a category with this name.')).toBeNull()
    expect(posted()).toHaveLength(12)
  })

  it('lists expense and income categories separately and hides archived ones by default', async () => {
    categories = [
      category('expense', 'Food'),
      category('income', 'Salary'),
      category('expense', 'Old', true),
    ]
    renderCategories()

    expect(await screen.findByText('Food')).toBeTruthy()
    expect(screen.getByText('Salary')).toBeTruthy()
    expect(screen.queryByText('Old')).toBeNull()

    await userEvent.click(screen.getByLabelText('Show archived categories'))

    expect(screen.getByText('Old')).toBeTruthy()
  })

  it('adds a category to the kind whose form was used', async () => {
    renderCategories()
    await screen.findByRole('heading', { name: 'Income categories' })
    const form = screen.getByRole('form', { name: 'Add to Income categories' })

    await userEvent.type(form.querySelector('input[type="text"], input:not([type])')!, 'Gift')
    await userEvent.click(form.querySelector('button[type="submit"]')!)

    expect(await screen.findByText('Gift')).toBeTruthy()
    expect(posted()).toEqual([expect.objectContaining({ kind: 'income', name: 'Gift' })])
  })

  it('archives and restores a category', async () => {
    categories = [category('expense', 'Food')]
    renderCategories()

    await userEvent.click(await screen.findByRole('button', { name: 'Archive Food' }))
    await waitFor(() => expect(screen.queryByText('Food')).toBeNull())

    await userEvent.click(screen.getByLabelText('Show archived categories'))
    await userEvent.click(await screen.findByRole('button', { name: 'Restore Food' }))

    expect(await screen.findByRole('button', { name: 'Archive Food' })).toBeTruthy()
  })
})
