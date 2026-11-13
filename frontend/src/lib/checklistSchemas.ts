import { z } from 'zod'
import type { ValidationMessageKey } from './authSchemas'

// Validation messages are translation keys; the forms pass them through `t()`.
const required: ValidationMessageKey = 'validation.required'
const habitNameLength: ValidationMessageKey = 'validation.habitNameLength'
const taskTitleLength: ValidationMessageKey = 'validation.taskTitleLength'
const weekdaysRequired: ValidationMessageKey = 'validation.weekdaysRequired'

/** Every ISO weekday, Monday (1) to Sunday (7): the default schedule of a new habit. */
export const EVERY_WEEKDAY = [1, 2, 3, 4, 5, 6, 7]

// Mirrors the backend rules: not blank, at most 200 characters.
export const taskFormSchema = z.object({
  title: z.string().trim().min(1, required).max(200, taskTitleLength),
})

export type TaskForm = z.infer<typeof taskFormSchema>

// Mirrors the backend rules: a name of at most 100 characters and at least one weekday.
export const habitFormSchema = z.object({
  name: z.string().trim().min(1, required).max(100, habitNameLength),
  weekdays: z.array(z.number()).min(1, weekdaysRequired),
  low_energy: z.boolean(),
  // Optional, and only used while `low_energy` is on.
  low_energy_name: z.string().trim().max(100, habitNameLength),
})

export type HabitForm = z.infer<typeof habitFormSchema>
