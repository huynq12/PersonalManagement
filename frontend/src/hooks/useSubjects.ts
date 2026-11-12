import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  createSubject,
  deleteSubject,
  listSubjects,
  SUBJECTS_QUERY_KEY,
  updateSubject,
  type SubjectChanges,
} from '../lib/subjects'

/** All of the user's subjects, archived ones included, by name. */
export function useSubjects() {
  return useQuery({ queryKey: SUBJECTS_QUERY_KEY, queryFn: listSubjects })
}

export function useCreateSubject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSubject,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SUBJECTS_QUERY_KEY }),
  })
}

interface UpdateSubjectInput {
  id: string
  changes: SubjectChanges
}

export function useUpdateSubject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, changes }: UpdateSubjectInput) => updateSubject(id, changes),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SUBJECTS_QUERY_KEY }),
  })
}

export function useDeleteSubject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: deleteSubject,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SUBJECTS_QUERY_KEY }),
  })
}
