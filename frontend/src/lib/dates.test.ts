import { describe, expect, it } from "vitest";
import {
  addDays,
  formatLongDate,
  formatTime,
  isIsoDate,
  isoWeekday,
  todayIn,
  weekdayNames,
} from "./dates";

describe("todayIn", () => {
  it("returns the calendar date in the given time zone, not in UTC", () => {
    // 20:30 UTC on the 11th is already 03:30 on the 12th in Vietnam (UTC+7).
    const instant = new Date("2026-11-11T20:30:00Z");

    expect(todayIn("Asia/Ho_Chi_Minh", instant)).toBe("2026-11-12");
    expect(todayIn("UTC", instant)).toBe("2026-11-11");
    expect(todayIn("America/Los_Angeles", instant)).toBe("2026-11-11");
  });
});

describe("addDays", () => {
  it("moves forward and backward across month and year boundaries", () => {
    expect(addDays("2026-11-12", 1)).toBe("2026-11-13");
    expect(addDays("2026-11-30", 1)).toBe("2026-12-01");
    expect(addDays("2026-01-01", -1)).toBe("2025-12-31");
    expect(addDays("2028-02-28", 1)).toBe("2028-02-29");
  });
});

describe("isIsoDate", () => {
  it("accepts real calendar dates in YYYY-MM-DD form", () => {
    expect(isIsoDate("2026-11-12")).toBe(true);
  });

  it("rejects other shapes and dates that do not exist", () => {
    expect(isIsoDate("12/11/2026")).toBe(false);
    expect(isIsoDate("2026-13-01")).toBe(false);
    expect(isIsoDate("2026-02-30")).toBe(false);
    expect(isIsoDate("today")).toBe(false);
    expect(isIsoDate("")).toBe(false);
  });
});

describe("isoWeekday", () => {
  it("numbers Monday as 1 and Sunday as 7", () => {
    expect(isoWeekday("2026-11-09")).toBe(1);
    expect(isoWeekday("2026-11-12")).toBe(4);
    expect(isoWeekday("2026-11-15")).toBe(7);
  });
});

describe("formatLongDate", () => {
  it("formats the calendar date itself, whatever the local time zone", () => {
    const expected = new Intl.DateTimeFormat("en", {
      dateStyle: "full",
      timeZone: "UTC",
    }).format(new Date("2026-11-12T00:00:00Z"));

    expect(formatLongDate("2026-11-12", "en")).toBe(expected);
    expect(formatLongDate("2026-11-12", "en")).toContain("November 12, 2026");
  });
});

describe("weekdayNames", () => {
  it("lists seven short names starting on Monday", () => {
    expect(weekdayNames("en")).toEqual([
      "Mon",
      "Tue",
      "Wed",
      "Thu",
      "Fri",
      "Sat",
      "Sun",
    ]);
  });
});

describe("formatTime", () => {
  it("shows the time of day in the given time zone, not in UTC", () => {
    // 03:30 UTC is 10:30 in Ho Chi Minh City and 12:30 in Sydney.
    expect(
      formatTime("2026-11-12T03:30:00Z", "en-GB", "Asia/Ho_Chi_Minh"),
    ).toBe("10:30");
    expect(
      formatTime("2026-11-12T03:30:00Z", "en-GB", "Australia/Sydney"),
    ).toBe("14:30");
  });
});
