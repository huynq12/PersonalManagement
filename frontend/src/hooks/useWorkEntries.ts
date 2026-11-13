import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { IsoDate, IsoMonth } from '../lib/dates'
import {
  deleteWorkEntry,
  getWorkMonth,
  putWorkEntry,
  WORK_ENTRIES_QUERY_KEY,
  workMonthQueryKey,
} from '../lib/workEntries'

/** The entries and totals of one month. */
export function useWorkEntries(month: IsoMonth) {
  return useQuery({ queryKey: workMonthQueryKey(month), queryFn: () => getWorkMonth(month) })
}

/** Creates or updates a day's entry, then refreshes every month's list and totals. */
export function usePutWorkEntry() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ date, hours, note }: { date: IsoDate; hours: number; note: string | null }) =>
      putWorkEntry(date, { hours, note }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: WORK_ENTRIES_QUERY_KEY }),
  })
}

/** Deletes a day's entry, then refreshes every month's list and totals. */
export function useDeleteWorkEntry() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (date: IsoDate) => deleteWorkEntry(date),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: WORK_ENTRIES_QUERY_KEY }),
  })
}
