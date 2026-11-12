import {
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
} from "@tanstack/react-query";
import type { IsoDate } from "../lib/dates";
import {
  cancelSession,
  completeSession,
  CURRENT_SESSION_QUERY_KEY,
  deleteSession,
  getCurrentSession,
  listSessionsOn,
  SESSIONS_QUERY_KEY,
  sessionsQueryKey,
  startSession,
} from "../lib/studySessions";

/** The running session, or `null`. Used to restore the timer after a reload. */
export function useCurrentSession() {
  return useQuery({
    queryKey: CURRENT_SESSION_QUERY_KEY,
    queryFn: getCurrentSession,
  });
}

/** The sessions started on one local date, newest first. */
export function useSessionsOn(date: IsoDate) {
  return useQuery({
    queryKey: sessionsQueryKey(date),
    queryFn: () => listSessionsOn(date),
  });
}

/** Any change to a session changes what is running and what the history shows. */
function invalidateSessions(queryClient: QueryClient) {
  return Promise.all([
    queryClient.invalidateQueries({ queryKey: CURRENT_SESSION_QUERY_KEY }),
    queryClient.invalidateQueries({ queryKey: SESSIONS_QUERY_KEY }),
  ]);
}

export function useStartSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: startSession,
    onSettled: () => invalidateSessions(queryClient),
  });
}

export function useCompleteSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: completeSession,
    onSettled: () => invalidateSessions(queryClient),
  });
}

export function useCancelSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: cancelSession,
    onSettled: () => invalidateSessions(queryClient),
  });
}

export function useDeleteSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: deleteSession,
    onSuccess: () => invalidateSessions(queryClient),
  });
}
