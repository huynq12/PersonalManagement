import { z } from 'zod'
import type { ValidationMessageKey } from './authSchemas'

const required: ValidationMessageKey = 'validation.required'
const subjectNameLength: ValidationMessageKey = 'validation.subjectNameLength'

/**
 * The colors a subject can be given in the UI. The API accepts any `#RRGGBB`; the screen offers
 * a fixed palette. `key` selects the translated color name under `subjects.colors`.
 */
export const SUBJECT_COLORS = [
  { key: 'red', hex: '#EF4444' },
  { key: 'orange', hex: '#F97316' },
  { key: 'yellow', hex: '#EAB308' },
  { key: 'green', hex: '#22C55E' },
  { key: 'teal', hex: '#14B8A6' },
  { key: 'blue', hex: '#3B82F6' },
  { key: 'purple', hex: '#8B5CF6' },
  { key: 'pink', hex: '#EC4899' },
  { key: 'gray', hex: '#71717A' },
] as const

export const DEFAULT_SUBJECT_COLOR = '#3B82F6'

// Mirrors the backend rules: not blank, at most 100 characters.
export const subjectFormSchema = z.object({
  name: z.string().trim().min(1, required).max(100, subjectNameLength),
  color: z.string(),
})

export type SubjectForm = z.infer<typeof subjectFormSchema>
