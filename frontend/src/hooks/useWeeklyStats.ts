import { useQuery } from '@tanstack/react-query'
import type { IsoDate } from '../lib/dates'
import { getWeeklyStats, weeklyStatsQueryKey } from '../lib/stats'

/** The statistics of the week that starts on the given Monday. */
export function useWeeklyStats(weekStart: IsoDate) {
  return useQuery({
    queryKey: weeklyStatsQueryKey(weekStart),
    queryFn: () => getWeeklyStats(weekStart),
  })
}
