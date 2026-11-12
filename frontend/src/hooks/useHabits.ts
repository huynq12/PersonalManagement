import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query'
import {
  createHabit,
  DAYS_QUERY_KEY,
  deleteHabit,
  HABITS_QUERY_KEY,
  listHabits,
  updateHabit,
  type Habit,
  type HabitChanges,
} from '../lib/checklist'
import { sortOrderChanges } from '../lib/habitOrder'

/** All of the user's habits, archived ones included, in display order. */
export function useHabits() {
  return useQuery({ queryKey: HABITS_QUERY_KEY, queryFn: listHabits })
}

/** A change to a habit also changes what the day views show (name, schedule, order). */
function invalidateChecklist(queryClient: QueryClient) {
  return Promise.all([
    queryClient.invalidateQueries({ queryKey: HABITS_QUERY_KEY }),
    queryClient.invalidateQueries({ queryKey: DAYS_QUERY_KEY }),
  ])
}

export function useCreateHabit() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createHabit,
    onSuccess: () => invalidateChecklist(queryClient),
  })
}

interface UpdateHabitInput {
  id: string
  changes: HabitChanges
}

export function useUpdateHabit() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, changes }: UpdateHabitInput) => updateHabit(id, changes),
    onSuccess: () => invalidateChecklist(queryClient),
  })
}

export function useDeleteHabit() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: deleteHabit,
    onSuccess: () => invalidateChecklist(queryClient),
  })
}

/**
 * Saves a new order of the active habits. Pass them in the order they should appear.
 *
 * The list is renumbered 0, 1, 2, ... and only habits whose number changed are sent. The cache
 * shows the new order immediately, so a dropped row does not jump back while saving.
 */
export function useReorderHabits() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (ordered: Habit[]) =>
      Promise.all(
        sortOrderChanges(ordered).map((change) =>
          updateHabit(change.id, { sort_order: change.sort_order }),
        ),
      ),
    onMutate: async (ordered) => {
      await queryClient.cancelQueries({ queryKey: HABITS_QUERY_KEY })
      const previous = queryClient.getQueryData<Habit[]>(HABITS_QUERY_KEY)

      const reorderedIds = new Set(ordered.map((habit) => habit.id))
      queryClient.setQueryData<Habit[]>(HABITS_QUERY_KEY, [
        ...ordered.map((habit, index) => ({ ...habit, sort_order: index })),
        // Habits that were not part of the reordering (archived ones) keep their place after.
        ...(previous ?? []).filter((habit) => !reorderedIds.has(habit.id)),
      ])
      return previous
    },
    onError: (_error, _ordered, previous) => {
      queryClient.setQueryData(HABITS_QUERY_KEY, previous)
    },
    onSettled: () => invalidateChecklist(queryClient),
  })
}
