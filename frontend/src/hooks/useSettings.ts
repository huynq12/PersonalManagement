import { useQuery } from "@tanstack/react-query";
import { getSettings, SETTINGS_QUERY_KEY } from "../lib/settings";

/** The user's Pomodoro settings. */
export function useSettings() {
  return useQuery({ queryKey: SETTINGS_QUERY_KEY, queryFn: getSettings });
}
