import { describe, expect, it } from "vitest";
import {
  breakKind,
  breakMinutes,
  currentBreak,
  formatClock,
  formatDuration,
  remainingSeconds,
} from "./pomodoro";

describe("remainingSeconds", () => {
  it("rounds a partial second up, so the clock reads 00:00 only when time is up", () => {
    expect(remainingSeconds(10_000, 0)).toBe(10);
    expect(remainingSeconds(10_000, 100)).toBe(10);
    expect(remainingSeconds(10_000, 9_001)).toBe(1);
    expect(remainingSeconds(10_000, 10_000)).toBe(0);
  });

  it("never goes below zero", () => {
    expect(remainingSeconds(10_000, 60_000)).toBe(0);
  });
});

describe("formatClock", () => {
  it("shows minutes and seconds with two digits each", () => {
    expect(formatClock(0)).toBe("00:00");
    expect(formatClock(5)).toBe("00:05");
    expect(formatClock(25 * 60)).toBe("25:00");
    expect(formatClock(14 * 60 + 59)).toBe("14:59");
  });

  it("keeps counting minutes past an hour", () => {
    expect(formatClock(180 * 60)).toBe("180:00");
  });
});

describe("breakKind", () => {
  it("gives a long break after every Nth completed session", () => {
    expect(breakKind(1, 4)).toBe("short");
    expect(breakKind(3, 4)).toBe("short");
    expect(breakKind(4, 4)).toBe("long");
    expect(breakKind(5, 4)).toBe("short");
    expect(breakKind(8, 4)).toBe("long");
  });

  it("gives a long break every time when N is 1", () => {
    expect(breakKind(1, 1)).toBe("long");
  });
});

describe("breakMinutes", () => {
  const settings = { short_break_minutes: 5, long_break_minutes: 15 };

  it("picks the length for the kind of break", () => {
    expect(breakMinutes("short", settings)).toBe(5);
    expect(breakMinutes("long", settings)).toBe(15);
  });
});

describe("formatDuration", () => {
  it("formats short spans in seconds and longer ones in minutes", () => {
    expect(formatDuration(45, "en")).toBe("45 sec");
    expect(formatDuration(25 * 60, "en")).toBe("25 min");
    expect(formatDuration(25 * 60 + 40, "en")).toBe("26 min");
  });

  it("adds hours once there are 60 minutes or more", () => {
    expect(formatDuration(60 * 60, "en")).toBe("1 hr");
    expect(formatDuration(75 * 60, "en")).toBe("1 hr 15 min");
  });

  it("follows the active language", () => {
    expect(formatDuration(25 * 60, "vi")).toBe(
      new Intl.NumberFormat("vi", {
        style: "unit",
        unit: "minute",
        unitDisplay: "short",
      }).format(25),
    );
  });
});

describe("currentBreak", () => {
  const settings = {
    short_break_minutes: 5,
    long_break_minutes: 15,
    long_break_every: 2,
  };
  const NOW = Date.parse("2026-11-12T03:00:00Z");
  const endedMinutesAgo = (minutes: number) =>
    new Date(NOW - minutes * 60_000).toISOString();

  it("is none before any session is completed", () => {
    expect(currentBreak([], settings, NOW, null)).toBeNull();
  });

  it("is a short break that ends a break length after the newest session ended", () => {
    const sessions = [{ id: "a", ended_at: endedMinutesAgo(2) }];

    expect(currentBreak(sessions, settings, NOW, null)).toEqual({
      kind: "short",
      endsAt: NOW + 3 * 60_000,
      afterSessionId: "a",
    });
  });

  it("is a long break after the configured number of sessions", () => {
    const sessions = [
      { id: "b", ended_at: endedMinutesAgo(1) },
      { id: "a", ended_at: endedMinutesAgo(40) },
    ];

    expect(currentBreak(sessions, settings, NOW, null)?.kind).toBe("long");
  });

  it("is over once its time has passed", () => {
    const sessions = [{ id: "a", ended_at: endedMinutesAgo(5) }];

    expect(currentBreak(sessions, settings, NOW, null)).toBeNull();
  });

  it("is none when the user skipped the break after that session", () => {
    const sessions = [{ id: "a", ended_at: endedMinutesAgo(1) }];

    expect(currentBreak(sessions, settings, NOW, "a")).toBeNull();
  });
});
