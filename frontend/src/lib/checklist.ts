import { z } from 'zod'
import { apiRequest } from './api'
import type { IsoDate } from './dates'

export const habitSchema = z.object({
  id: z.string(),
  name: z.string(),
  /** ISO weekdays the habit applies to: 1 (Monday) to 7 (Sunday). */
  weekdays: z.array(z.number()),
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

export const dayViewSchema = z.object({
  date: z.string(),
  habits: z.array(dayHabitSchema),
  tasks: z.array(taskSchema),
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

// ---------------------------------------------------------------------------------------------
// Habits
// ---------------------------------------------------------------------------------------------

export const listHabits = () => apiRequest('/habits?include_archived=true', z.array(habitSchema))

export interface HabitInput {
  name: string
  weekdays: number[]
}

export const createHabit = (input: HabitInput) =>
  apiRequest('/habits', habitSchema, { method: 'POST', body: input })

export interface HabitChanges {
  name?: string
  weekdays?: number[]
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
