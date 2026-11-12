import { z } from 'zod'
import { apiRequest } from './api'

export const subjectSchema = z.object({
  id: z.string(),
  name: z.string(),
  /** Hex color in the form `#RRGGBB`. */
  color: z.string(),
  archived_at: z.string().nullable(),
  created_at: z.string(),
})

export type Subject = z.infer<typeof subjectSchema>

/** Body of a response with no content (204). */
const noContent = z.null()

/** All of the user's subjects, archived ones included. */
export const SUBJECTS_QUERY_KEY = ['subjects'] as const

export const listSubjects = () =>
  apiRequest('/subjects?include_archived=true', z.array(subjectSchema))

export interface SubjectInput {
  name: string
  color: string
}

export const createSubject = (input: SubjectInput) =>
  apiRequest('/subjects', subjectSchema, { method: 'POST', body: input })

export interface SubjectChanges {
  name?: string
  color?: string
  archived?: boolean
}

export const updateSubject = (id: string, changes: SubjectChanges) =>
  apiRequest(`/subjects/${id}`, subjectSchema, { method: 'PATCH', body: changes })

export const deleteSubject = (id: string) =>
  apiRequest(`/subjects/${id}`, noContent, { method: 'DELETE' })
