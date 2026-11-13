import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query'
import {
  createTask,
  DAYS_QUERY_KEY,
  dayQueryKey,
  deleteTask,
  getDay,
  HABITS_QUERY_KEY,
  setDayMode,
  setHabitCompleted,
  updateTask,
  type DayMode,
  type DayView,
} from '../lib/checklist'
import type { IsoDate } from '../lib/dates'

/** The scheduled habits and the to-dos of one date. */
export function useDay(date: IsoDate) {
  return useQuery({ queryKey: dayQueryKey(date), queryFn: () => getDay(date) })
}

/**
 * Applies `change` to the cached day view right away and returns the previous value, so the
 * screen reacts before the server answers and the change can be undone if saving fails.
 */
async function updateDayOptimistically(
  queryClient: QueryClient,
  date: IsoDate,
  change: (day: DayView) => DayView,
): Promise<DayView | undefined> {
  const key = dayQueryKey(date)
  // A refetch that is already on its way would overwrite the optimistic value.
  await queryClient.cancelQueries({ queryKey: key })
  const previous = queryClient.getQueryData<DayView>(key)
  if (previous) {
    queryClient.setQueryData(key, change(previous))
  }
  return previous
}

interface ToggleHabitInput {
  habitId: string
  completed: boolean
}

/** Checks or unchecks a habit on `date`, optimistically. */
export function useToggleHabit(date: IsoDate) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ habitId, completed }: ToggleHabitInput) =>
      setHabitCompleted(habitId, date, completed),
    onMutate: ({ habitId, completed }) =>
      updateDayOptimistically(queryClient, date, (day) => ({
        ...day,
        habits: day.habits.map((habit) =>
          habit.id === habitId ? { ...habit, completed } : habit,
        ),
        // Checking a habit locks the day mode right away; unchecking is settled by the refetch,
        // because the server also counts habits that are not shown.
        mode_locked: day.mode_locked || completed,
      })),
    onError: (_error, _input, previous) => {
      queryClient.setQueryData(dayQueryKey(date), previous)
    },
    // Whatever happened, load the server's version; a completion also changes the streak.
    onSettled: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: dayQueryKey(date) }),
        queryClient.invalidateQueries({ queryKey: HABITS_QUERY_KEY }),
      ]),
  })
}

interface ToggleTaskInput {
  taskId: string
  isDone: boolean
}

/** Checks or unchecks a to-do on `date`, optimistically. */
export function useToggleTask(date: IsoDate) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ taskId, isDone }: ToggleTaskInput) => updateTask(taskId, { is_done: isDone }),
    onMutate: ({ taskId, isDone }) =>
      updateDayOptimistically(queryClient, date, (day) => ({
        ...day,
        tasks: day.tasks.map((task) => (task.id === taskId ? { ...task, is_done: isDone } : task)),
      })),
    onError: (_error, _input, previous) => {
      queryClient.setQueryData(dayQueryKey(date), previous)
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: dayQueryKey(date) }),
  })
}

/** Adds a to-do to `date`. */
export function useCreateTask(date: IsoDate) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (title: string) => createTask(date, title),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: dayQueryKey(date) }),
  })
}

interface RenameTaskInput {
  taskId: string
  title: string
}

export function useRenameTask() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ taskId, title }: RenameTaskInput) => updateTask(taskId, { title }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: DAYS_QUERY_KEY }),
  })
}

export function useDeleteTask() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: deleteTask,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: DAYS_QUERY_KEY }),
  })
}

/**
 * Switches `date` between a normal and a low-energy day.
 *
 * The result always comes from the server: the list of habits changes with the mode, and a
 * refused change (another tab checked a habit) must show the real, locked state.
 */
export function useSetDayMode(date: IsoDate) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (mode: DayMode) => setDayMode(date, mode),
    onSettled: () => queryClient.invalidateQueries({ queryKey: dayQueryKey(date) }),
  })
}
