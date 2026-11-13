import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { IsoMonth } from '../lib/dates'
import {
  createCategory,
  createTransaction,
  deleteTransaction,
  FINANCE_CATEGORIES_QUERY_KEY,
  FINANCE_SUMMARY_QUERY_KEY,
  financeSummaryQueryKey,
  getCategories,
  getFinanceSummary,
  getTransactions,
  NAME_TAKEN,
  TRANSACTIONS_QUERY_KEY,
  transactionsQueryKey,
  updateCategory,
  updateTransaction,
  type Category,
  type Kind,
  type TransactionInput,
} from '../lib/finance'
import { ApiError } from '../lib/api'

/** Every category of the user, archived ones included. Callers filter what they show. */
export function useFinanceCategories() {
  return useQuery({
    queryKey: FINANCE_CATEGORIES_QUERY_KEY,
    queryFn: getCategories,
  })
}

/** The transactions of one month, newest first. */
export function useTransactions(month: IsoMonth) {
  return useQuery({
    queryKey: transactionsQueryKey(month),
    queryFn: () => getTransactions(month),
  })
}

/** Totals, balance, per-category expenses, and budget use of one month. */
export function useFinanceSummary(month: IsoMonth) {
  return useQuery({
    queryKey: financeSummaryQueryKey(month),
    queryFn: () => getFinanceSummary(month),
  })
}

/** Creates a category, then refreshes the category list. */
export function useCreateCategory() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createCategory,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: FINANCE_CATEGORIES_QUERY_KEY }),
  })
}

/**
 * Creates the suggested categories one by one. A name the user already has (409) is skipped, so
 * a retry after a partial failure is safe.
 */
export function useCreateSuggestedCategories() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (categories: { kind: Kind; name: string; color: string }[]) => {
      for (const category of categories) {
        try {
          await createCategory(category)
        } catch (error) {
          if (!(error instanceof ApiError && error.code === NAME_TAKEN)) {
            throw error
          }
        }
      }
    },
    // Refresh even after a failure: some categories may have been created before it.
    onSettled: () => queryClient.invalidateQueries({ queryKey: FINANCE_CATEGORIES_QUERY_KEY }),
  })
}

/** Renames, recolors, archives, or restores a category, then refreshes the list. */
export function useUpdateCategory() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      id,
      ...changes
    }: { id: string } & Partial<Pick<Category, 'name' | 'color'>> & {
        archived?: boolean
      }) => updateCategory(id, changes),
    onSuccess: () => {
      // Transactions and summaries show the category's name and color too.
      void queryClient.invalidateQueries({ queryKey: TRANSACTIONS_QUERY_KEY })
      void queryClient.invalidateQueries({
        queryKey: FINANCE_SUMMARY_QUERY_KEY,
      })
      return queryClient.invalidateQueries({
        queryKey: FINANCE_CATEGORIES_QUERY_KEY,
      })
    },
  })
}

/** Refreshes every month's list and summary after a transaction changes. */
function useRefreshMonths() {
  const queryClient = useQueryClient()

  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: TRANSACTIONS_QUERY_KEY }),
      queryClient.invalidateQueries({ queryKey: FINANCE_SUMMARY_QUERY_KEY }),
    ])
}

/** Records a transaction, then refreshes the months' lists and summaries. */
export function useCreateTransaction() {
  const refresh = useRefreshMonths()

  return useMutation({ mutationFn: createTransaction, onSuccess: refresh })
}

/** Updates a transaction, then refreshes the months' lists and summaries. */
export function useUpdateTransaction() {
  const refresh = useRefreshMonths()

  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: TransactionInput }) =>
      updateTransaction(id, input),
    onSuccess: refresh,
  })
}

/** Deletes a transaction, then refreshes the months' lists and summaries. */
export function useDeleteTransaction() {
  const refresh = useRefreshMonths()

  return useMutation({ mutationFn: deleteTransaction, onSuccess: refresh })
}
