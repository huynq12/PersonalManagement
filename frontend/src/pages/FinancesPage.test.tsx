import { cleanup, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n/i18n'
import { setAccessToken } from '../lib/api'
import { todayIn } from '../lib/dates'
import type { Category, FinanceSummary, Transaction } from '../lib/finance'
import {
  jsonResponse,
  renderApp,
  signedIn,
  stubApi,
  USER,
  type RecordedRequest,
} from '../test/testUtils'

const TODAY = todayIn(USER.timezone)
const THIS_MONTH = TODAY.slice(0, 7)

function category(id: string, kind: Category['kind'], name: string, archived = false): Category {
  return { id, kind, name, color: '#22C55E', archived_at: archived ? '2026-01-01T00:00:00Z' : null }
}

const CATEGORIES: Category[] = [
  category('food', 'expense', 'Food'),
  category('old', 'expense', 'Old expense', true),
  category('salary', 'income', 'Salary'),
]

function transaction(
  id: string,
  amount: number,
  cat: Category,
  note: string | null = null,
): Transaction {
  return {
    id,
    kind: cat.kind,
    amount,
    date: `${THIS_MONTH}-02`,
    note,
    category: { id: cat.id, name: cat.name, color: cat.color },
  }
}

let transactions: Transaction[]
let budget: number | null
let requests: RecordedRequest[]

function summary(): FinanceSummary {
  const total = (kind: Transaction['kind']) =>
    transactions.filter((item) => item.kind === kind).reduce((sum, item) => sum + item.amount, 0)
  const expense = total('expense')
  return {
    income_total: total('income'),
    expense_total: expense,
    balance: total('income') - expense,
    by_category: transactions
      .filter((item) => item.kind === 'expense')
      .map((item) => ({ kind: item.kind, category: item.category, total: item.amount })),
    budget: budget === null ? null : { amount: budget, spent: expense },
  }
}

function renderFinances(path = '/finances') {
  requests = stubApi({ ...signedIn }, (request) => {
    const url = new URL(request.path, 'http://localhost')
    if (request.method === 'GET' && url.pathname === '/finance-categories') {
      return jsonResponse(200, CATEGORIES)
    }
    if (request.method === 'GET' && url.pathname === '/transactions') {
      return jsonResponse(200, transactions)
    }
    if (request.method === 'GET' && url.pathname === '/finances/summary') {
      return jsonResponse(200, summary())
    }
    if (request.method === 'POST' && url.pathname === '/transactions') {
      const body = request.body as {
        category_id: string
        amount: number
        date: string
        note: string | null
      }
      const cat = CATEGORIES.find((item) => item.id === body.category_id)!
      const created = { ...transaction('new', body.amount, cat, body.note), date: body.date }
      transactions = [created, ...transactions]
      return jsonResponse(201, created)
    }
    return undefined
  })
  renderApp(path)
}

function posted() {
  return requests
    .filter((request) => request.method === 'POST' && request.path === '/transactions')
    .map((request) => request.body)
}

function categoryOptions() {
  return within(screen.getByLabelText('Category'))
    .getAllByRole('option')
    .map((option) => option.textContent)
}

beforeEach(async () => {
  await i18n.changeLanguage('en')
  transactions = []
  budget = null
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

describe('Finances screen', () => {
  it('shows income, expenses, balance, the breakdown, and the list for the month', async () => {
    transactions = [
      transaction('1', 45000, CATEGORIES[0], 'Lunch'),
      transaction('2', 2000000, CATEGORIES[2]),
    ]
    renderFinances()

    expect(await screen.findByText('Lunch', { exact: false })).toBeTruthy()
    expect(
      within(screen.getByText('Income', { selector: 'dt' }).closest('div')!).getByText(/2,000,000/),
    ).toBeTruthy()
    expect(
      within(screen.getByText('Balance', { selector: 'dt' }).closest('div')!).getByText(
        /1,955,000/,
      ),
    ).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'Expenses by category' })).toBeTruthy()
    expect(screen.getByText('+', { exact: false })).toBeTruthy()
  })

  it('offers only active categories of the chosen kind and follows the kind toggle', async () => {
    renderFinances()
    await screen.findByLabelText('Category')

    expect(categoryOptions()).toEqual(['Choose a category', 'Food'])

    await userEvent.click(screen.getByLabelText('Income'))

    expect(categoryOptions()).toEqual(['Choose a category', 'Salary'])
  })

  it('sends the amount in minor units of the account currency', async () => {
    renderFinances()
    await userEvent.type(await screen.findByLabelText('Amount (VND)'), '45000')
    await userEvent.selectOptions(screen.getByLabelText('Category'), 'Food')

    await userEvent.click(screen.getByRole('button', { name: 'Save transaction' }))

    await waitFor(() => expect(posted()).toHaveLength(1))
    // The form starts over once the transaction is saved.
    await waitFor(() =>
      expect((screen.getByLabelText('Amount (VND)') as HTMLInputElement).value).toBe(''),
    )
    expect(posted()).toEqual([{ category_id: 'food', amount: 45000, date: TODAY, note: null }])
  })

  it('rejects more decimals than the currency has', async () => {
    renderFinances()
    await userEvent.type(await screen.findByLabelText('Amount (VND)'), '45.5')
    await userEvent.selectOptions(screen.getByLabelText('Category'), 'Food')

    await userEvent.click(screen.getByRole('button', { name: 'Save transaction' }))

    expect(await screen.findByText(/Enter an amount greater than 0/)).toBeTruthy()
    expect(posted()).toEqual([])
  })

  it('requires a category', async () => {
    renderFinances()
    await userEvent.type(await screen.findByLabelText('Amount (VND)'), '100')

    await userEvent.click(screen.getByRole('button', { name: 'Save transaction' }))

    expect(await screen.findByText('This field is required.')).toBeTruthy()
    expect(posted()).toEqual([])
  })

  it('hides the budget bar and links to Settings when no budget is set', async () => {
    renderFinances()

    const link = await screen.findByRole('link', { name: 'Set a budget in Settings' })

    expect(link.getAttribute('href')).toBe('/settings')
    expect(screen.queryByRole('progressbar')).toBeNull()
  })

  it('shows the budget used while within the budget', async () => {
    budget = 100000
    transactions = [transaction('1', 40000, CATEGORIES[0])]
    renderFinances()

    const bar = await screen.findByRole('progressbar')

    expect(bar.getAttribute('aria-valuenow')).toBe('40')
    expect(bar.getAttribute('data-state')).toBe('within')
    expect(screen.getByText(/60,000.* left this month/)).toBeTruthy()
  })

  it('shows a distinct state when the month is over budget', async () => {
    budget = 100000
    transactions = [transaction('1', 130000, CATEGORIES[0])]
    renderFinances()

    const bar = await screen.findByRole('progressbar')

    expect(bar.getAttribute('aria-valuenow')).toBe('100')
    expect(bar.getAttribute('data-state')).toBe('over')
    expect(screen.getByText(/Over budget by .*30,000/)).toBeTruthy()
  })
})
