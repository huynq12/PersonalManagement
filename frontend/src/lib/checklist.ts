import { z } from 'zod'
import { apiRequest } from './api'
import type { IsoDate } from './dates'

export const habitSchema = z.object({
  id: z.string(),
  name: z.string(),
  /** ISO weekdays the habit applies to: 1 (Monday) to 7 (Sunday). */
  weekdays: z.array(z.number()),
  /** Whether the habit stays on low-energy days. */
  low_energy: z.boolean(),
  /** Easier version shown on low-energy days; always null when `low_energy` is false. */
  low_energy_name: z.string().nullable(),
  sort_order: z.number(),
  archived_at: z.string().nullable(),
  created_at: z.string(),
  current_streak: z.number(),
})

export type Habit = z.infer<typeof habitSchema>

export const taskSchema = z.object({
  id: z.string(),
  date: z.string(),
  title: z.string(),
  is_done: z.boolean(),
  done_at: z.string().nullable(),
  sort_order: z.number(),
  created_at: z.string(),
})

export type Task = z.infer<typeof taskSchema>

const dayHabitSchema = z.object({
  id: z.string(),
  name: z.string(),
  completed: z.boolean(),
})

export type DayHabit = z.infer<typeof dayHabitSchema>

export const dayModeSchema = z.enum(['normal', 'low_energy'])

export type DayMode = z.infer<typeof dayModeSchema>

export const dayViewSchema = z.object({
  date: z.string(),
  mode: dayModeSchema,
  /** The mode cannot change: the date is not today, or a habit is already checked. */
  mode_locked: z.boolean(),
  habits: z.array(dayHabitSchema),
  tasks: z.array(taskSchema),
  /** The day's work entry; `null` when no hours were logged. */
  work: z.object({ hours: z.number(), pay: z.number().int() }).nullable(),
  /** Total expenses of the day in minor units. */
  expense_total: z.number().int(),
  /** Time of the day's completed study sessions, in seconds. */
  study_seconds: z.number().int(),
})

export type DayView = z.infer<typeof dayViewSchema>

/** Body of a response with no content (204). */
const noContent = z.null()

// ---------------------------------------------------------------------------------------------
// Query keys
// ---------------------------------------------------------------------------------------------

/** Prefix of every day view; invalidate it when something shown on any day changes. */
export const DAYS_QUERY_KEY = ['days'] as const
export const dayQueryKey = (date: IsoDate) => [...DAYS_QUERY_KEY, date] as const
/** All of the user's habits, archived ones included. */
export const HABITS_QUERY_KEY = ['habits'] as const

// ---------------------------------------------------------------------------------------------
// Day view
// ---------------------------------------------------------------------------------------------

export const getDay = (date: IsoDate) => apiRequest(`/days/${date}`, dayViewSchema)

/** Marks a day as a normal or a low-energy day. The server accepts only today. */
export const setDayMode = (date: IsoDate, mode: DayMode) =>
  apiRequest(`/days/${date}/mode`, noContent, { method: 'PUT', body: { mode } })

// ---------------------------------------------------------------------------------------------
// Habits
// ---------------------------------------------------------------------------------------------

export const listHabits = () => apiRequest('/habits?include_archived=true', z.array(habitSchema))

export interface HabitInput {
  name: string
  weekdays: number[]
  low_energy: boolean
  /** Ignored by the server unless `low_energy` is true; blank means "use the normal name". */
  low_energy_name: string | null
}

export const createHabit = (input: HabitInput) =>
  apiRequest('/habits', habitSchema, { method: 'POST', body: input })

export interface HabitChanges {
  name?: string
  weekdays?: number[]
  low_energy?: boolean
  /** A blank string clears the easier name; `null` leaves it unchanged. */
  low_energy_name?: string | null
  sort_order?: number
  archived?: boolean
}

export const updateHabit = (id: string, changes: HabitChanges) =>
  apiRequest(`/habits/${id}`, habitSchema, { method: 'PATCH', body: changes })

export const deleteHabit = (id: string) =>
  apiRequest(`/habits/${id}`, noContent, { method: 'DELETE' })

/** Marks a habit done on a day, or removes the mark. Both are idempotent on the server. */
export const setHabitCompleted = (id: string, date: IsoDate, completed: boolean) =>
  apiRequest(`/habits/${id}/completions/${date}`, noContent, {
    method: completed ? 'PUT' : 'DELETE',
  })

// ---------------------------------------------------------------------------------------------
// To-dos
// ---------------------------------------------------------------------------------------------

export const createTask = (date: IsoDate, title: string) =>
  apiRequest('/tasks', taskSchema, { method: 'POST', body: { date, title } })

export interface TaskChanges {
  title?: string
  is_done?: boolean
}

export const updateTask = (id: string, changes: TaskChanges) =>
  apiRequest(`/tasks/${id}`, taskSchema, { method: 'PATCH', body: changes })

export const deleteTask = (id: string) =>
  apiRequest(`/tasks/${id}`, noContent, { method: 'DELETE' })
