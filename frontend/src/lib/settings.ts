import { z } from "zod";
import { apiRequest } from "./api";

/** The user's Pomodoro settings, in minutes. */
export const settingsSchema = z.object({
  focus_minutes: z.number(),
  short_break_minutes: z.number(),
  long_break_minutes: z.number(),
  /** A long break follows every this many focus sessions. */
  long_break_every: z.number(),
});

export type Settings = z.infer<typeof settingsSchema>;

export const SETTINGS_QUERY_KEY = ["settings"] as const;

export const getSettings = () => apiRequest("/settings", settingsSchema);
