import { z } from 'zod'
import { apiRequest } from './api'
import type { IsoDate } from './dates'

export const dayModeSchema = z.enum(['normal', 'low_energy'])

export type DayMode = z.infer<typeof dayModeSchema>

export const dayStatsSchema = z.object({
  date: z.string(),
  mode: dayModeSchema,
  /** Focus time of the completed sessions that started on the date. */
  study_seconds: z.number(),
  pomodoros: z.number(),
  habits_scheduled: z.number(),
  habits_done: z.number(),
  tasks_total: z.number(),
  tasks_done: z.number(),
  /** From 0 to 1; `null` when the day has no habits or to-dos. */
  completion_rate: z.number().nullable(),
})

export type DayStats = z.infer<typeof dayStatsSchema>

export const subjectStatsSchema = z.object({
  /** `null` for sessions without a subject. */
  subject_id: z.string().nullable(),
  name: z.string().nullable(),
  color: z.string().nullable(),
  study_seconds: z.number(),
  pomodoros: z.number(),
})

export type SubjectStats = z.infer<typeof subjectStatsSchema>

export const habitStreaksSchema = z.object({
  id: z.string(),
  name: z.string(),
  current_streak: z.number(),
  longest_streak: z.number(),
})

export type HabitStreaks = z.infer<typeof habitStreaksSchema>

export const weekTotalsSchema = z.object({
  study_seconds: z.number(),
  pomodoros: z.number(),
  /** From 0 to 1; `null` when the week has no habits or to-dos. */
  completion_rate: z.number().nullable(),
})

export type WeekTotals = z.infer<typeof weekTotalsSchema>

export const weeklyStatsSchema = z.object({
  week_start: z.string(),
  week_end: z.string(),
  days: z.array(dayStatsSchema),
  subjects: z.array(subjectStatsSchema),
  habits: z.array(habitStreaksSchema),
  totals: weekTotalsSchema,
  previous_totals: weekTotalsSchema,
})

export type WeeklyStats = z.infer<typeof weeklyStatsSchema>

export const weeklyStatsQueryKey = (weekStart: IsoDate) => ['stats', 'weekly', weekStart] as const

/** The statistics of the week that starts on `weekStart`, which must be a Monday. */
export const getWeeklyStats = (weekStart: IsoDate) =>
  apiRequest(`/stats/weekly?week_start=${weekStart}`, weeklyStatsSchema)
