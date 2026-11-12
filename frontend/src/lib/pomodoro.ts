// Pure helpers for the Pomodoro timer: clock arithmetic and break rules.

/** Whole seconds left until `endsAt`, rounded up so the clock shows 00:00 only at the end. */
export function remainingSeconds(endsAt: number, now: number): number {
  return Math.max(0, Math.ceil((endsAt - now) / 1000));
}

/** A countdown as `MM:SS`. Minutes keep counting past 99 (a session can last 180 minutes). */
export function formatClock(totalSeconds: number): string {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export type BreakKind = "short" | "long";

/**
 * The break that follows a focus session: a long one after every `longBreakEvery`-th session
 * completed today, a short one otherwise. `completedToday` includes the session just finished.
 */
export function breakKind(
  completedToday: number,
  longBreakEvery: number,
): BreakKind {
  return completedToday % longBreakEvery === 0 ? "long" : "short";
}

export function breakMinutes(
  kind: BreakKind,
  settings: { short_break_minutes: number; long_break_minutes: number },
): number {
  return kind === "long"
    ? settings.long_break_minutes
    : settings.short_break_minutes;
}

function formatUnit(
  value: number,
  unit: "second" | "minute" | "hour",
  language?: string,
) {
  return new Intl.NumberFormat(language, {
    style: "unit",
    unit,
    unitDisplay: "short",
  }).format(value);
}

/**
 * A span of time in the active language: seconds under a minute, otherwise whole minutes,
 * with hours once it reaches an hour ("45 sec", "25 min", "1 hr 15 min").
 */
export function formatDuration(
  totalSeconds: number,
  language: string | undefined,
): string {
  if (totalSeconds < 60) {
    return formatUnit(totalSeconds, "second", language);
  }
  const totalMinutes = Math.round(totalSeconds / 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours === 0) {
    return formatUnit(minutes, "minute", language);
  }
  return minutes === 0
    ? formatUnit(hours, "hour", language)
    : `${formatUnit(hours, "hour", language)} ${formatUnit(minutes, "minute", language)}`;
}

export interface ActiveBreak {
  kind: BreakKind;
  /** Epoch milliseconds when the break ends. */
  endsAt: number;
  /** The completed session this break follows; used to skip it. */
  afterSessionId: string;
}

/**
 * The break the user is in right now, if any. Breaks are not stored: they follow from the
 * newest completed session of the day, so a reload does not lose them.
 *
 * `completedNewestFirst` are today's completed sessions, newest first. The break is over once
 * its time has passed, or when the user skipped the one after `skippedAfterSessionId`.
 */
export function currentBreak(
  completedNewestFirst: { id: string; ended_at: string | null }[],
  settings: {
    short_break_minutes: number;
    long_break_minutes: number;
    long_break_every: number;
  },
  now: number,
  skippedAfterSessionId: string | null,
): ActiveBreak | null {
  const latest = completedNewestFirst[0];
  if (!latest?.ended_at || latest.id === skippedAfterSessionId) {
    return null;
  }

  const kind = breakKind(
    completedNewestFirst.length,
    settings.long_break_every,
  );
  const endsAt =
    Date.parse(latest.ended_at) + breakMinutes(kind, settings) * 60_000;
  return endsAt > now ? { kind, endsAt, afterSessionId: latest.id } : null;
}
