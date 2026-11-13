import { z } from 'zod'
import { apiRequest } from './api'
import type { IsoDate, IsoMonth } from './dates'

export const kindSchema = z.enum(['expense', 'income'])

export type Kind = z.infer<typeof kindSchema>

export const KINDS: readonly Kind[] = ['expense', 'income']

export const categorySchema = z.object({
  id: z.string(),
  kind: kindSchema,
  name: z.string(),
  color: z.string(),
  archived_at: z.string().nullable(),
})

export type Category = z.infer<typeof categorySchema>

const categoryRefSchema = z.object({
  id: z.string(),
  name: z.string(),
  color: z.string(),
})

export const transactionSchema = z.object({
  id: z.string(),
  kind: kindSchema,
  /** Minor units of the account currency. */
  amount: z.number(),
  date: z.string(),
  note: z.string().nullable(),
  category: categoryRefSchema,
})

export type Transaction = z.infer<typeof transactionSchema>

export const financeSummarySchema = z.object({
  income_total: z.number(),
  expense_total: z.number(),
  balance: z.number(),
  by_category: z.array(
    z.object({
      kind: kindSchema,
      category: categoryRefSchema,
      total: z.number(),
    }),
  ),
  /** `null` when no budget is set. */
  budget: z.object({ amount: z.number(), spent: z.number() }).nullable(),
})

export type FinanceSummary = z.infer<typeof financeSummarySchema>

/** Prefix of every category query. */
export const FINANCE_CATEGORIES_QUERY_KEY = ['finance-categories'] as const
/** Prefix of every transaction list query, to invalidate all months at once. */
export const TRANSACTIONS_QUERY_KEY = ['transactions'] as const
/** Prefix of every summary query, to invalidate all months at once. */
export const FINANCE_SUMMARY_QUERY_KEY = ['finance-summary'] as const

export const transactionsQueryKey = (month: IsoMonth) => [...TRANSACTIONS_QUERY_KEY, month] as const
export const financeSummaryQueryKey = (month: IsoMonth) =>
  [...FINANCE_SUMMARY_QUERY_KEY, month] as const

/** The API error code for a category name that the user already has within its kind. */
export const NAME_TAKEN = 'finance_category_name_taken'

/** The fixed palette categories pick their color from; the API only checks the format. */
export const CATEGORY_COLORS = [
  '#EF4444',
  '#F97316',
  '#EAB308',
  '#22C55E',
  '#14B8A6',
  '#3B82F6',
  '#8B5CF6',
  '#EC4899',
  '#71717A',
] as const

/** The starter categories, in order. Names come from `finances.suggestedCategories.<kind>.<id>`. */
export const SUGGESTED_CATEGORIES = {
  expense: [
    'food',
    'transport',
    'housing',
    'bills',
    'shopping',
    'health',
    'entertainment',
    'education',
    'other',
  ],
  income: ['salary', 'bonus', 'other'],
} as const

/** Body of a response with no content (204). */
const noContent = z.null()

export const getCategories = () =>
  apiRequest('/finance-categories?include_archived=true', z.array(categorySchema))

export const createCategory = (input: { kind: Kind; name: string; color: string }) =>
  apiRequest('/finance-categories', categorySchema, {
    method: 'POST',
    body: input,
  })

export const updateCategory = (
  id: string,
  changes: { name?: string; color?: string; archived?: boolean },
) =>
  apiRequest(`/finance-categories/${id}`, categorySchema, {
    method: 'PATCH',
    body: changes,
  })

export const getTransactions = (month: IsoMonth) =>
  apiRequest(`/transactions?month=${month}`, z.array(transactionSchema))

export interface TransactionInput {
  category_id: string
  amount: number
  date: IsoDate
  note: string | null
}

export const createTransaction = (input: TransactionInput) =>
  apiRequest('/transactions', transactionSchema, {
    method: 'POST',
    body: input,
  })

export const updateTransaction = (id: string, input: TransactionInput) =>
  apiRequest(`/transactions/${id}`, transactionSchema, {
    method: 'PATCH',
    body: input,
  })

export const deleteTransaction = (id: string) =>
  apiRequest(`/transactions/${id}`, noContent, { method: 'DELETE' })

export const getFinanceSummary = (month: IsoMonth) =>
  apiRequest(`/finances/summary?month=${month}`, financeSummarySchema)
