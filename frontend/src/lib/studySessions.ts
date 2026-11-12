import { z } from "zod";
import { apiRequest } from "./api";
import type { IsoDate } from "./dates";

export const studySessionSchema = z.object({
  id: z.string(),
  /** `null` for a session without a subject, or once its subject is deleted. */
  subject_id: z.string().nullable(),
  status: z.enum(["running", "completed", "cancelled"]),
  planned_minutes: z.number(),
  started_at: z.string(),
  /** When it was completed or cancelled; `null` while running. */
  ended_at: z.string().nullable(),
  /** Time studied; only set on completed sessions. */
  duration_seconds: z.number().nullable(),
  local_date: z.string(),
});

export type StudySession = z.infer<typeof studySessionSchema>;

/** Body of a response with no content (204). */
const noContent = z.null();

/** The running session, or `null` when there is none (the API answers 204). */
export const CURRENT_SESSION_QUERY_KEY = ["study-sessions", "current"] as const;
/** Prefix of every history query; invalidate it when any session changes. */
export const SESSIONS_QUERY_KEY = ["study-sessions", "history"] as const;
export const sessionsQueryKey = (date: IsoDate) =>
  [...SESSIONS_QUERY_KEY, date] as const;

export const getCurrentSession = () =>
  apiRequest("/study-sessions/current", studySessionSchema.nullable());

/** Sessions started on one local date, newest first, whatever their status. */
export const listSessionsOn = (date: IsoDate) =>
  apiRequest(
    `/study-sessions?from=${date}&to=${date}`,
    z.array(studySessionSchema),
  );

export interface StartSessionInput {
  /** Leave out to study without a subject. */
  subject_id?: string;
}

/** Starts a session; its length comes from the user's settings. */
export const startSession = (input: StartSessionInput) =>
  apiRequest("/study-sessions", studySessionSchema, {
    method: "POST",
    body: input,
  });

export const completeSession = (id: string) =>
  apiRequest(`/study-sessions/${id}/complete`, studySessionSchema, {
    method: "POST",
  });

export const cancelSession = (id: string) =>
  apiRequest(`/study-sessions/${id}/cancel`, studySessionSchema, {
    method: "POST",
  });

export const deleteSession = (id: string) =>
  apiRequest(`/study-sessions/${id}`, noContent, { method: "DELETE" });
