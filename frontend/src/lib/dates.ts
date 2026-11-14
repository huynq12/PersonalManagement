// Calendar dates travel as `YYYY-MM-DD` strings, the same form the API uses. A "day" is always
// the calendar date in the user's time zone (SPEC.md), never the browser's.

/** A calendar date as `YYYY-MM-DD`. */
export type IsoDate = string

/** A calendar month as `YYYY-MM`, the form the work log and finance endpoints take. */
export type IsoMonth = string

const ISO_DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/
const MILLISECONDS_PER_DAY = 24 * 60 * 60 * 1000
// 2024-01-01 was a Monday; used to produce weekday names in week order.
const A_MONDAY = Date.UTC(2024, 0, 1)

/** Midnight UTC of the date. UTC keeps the arithmetic free of daylight saving surprises. */
function toUtc(date: IsoDate): Date {
  return new Date(`${date}T00:00:00Z`)
}

function fromUtc(date: Date): IsoDate {
  return date.toISOString().slice(0, 10)
}

/** The calendar date it is right now in `timeZone` (an IANA name such as `Asia/Ho_Chi_Minh`). */
export function todayIn(timeZone: string, now: Date = new Date()): IsoDate {
  // The `en-CA` locale formats dates as YYYY-MM-DD.
  return new Intl.DateTimeFormat('en-CA', {
    timeZone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(now)
}

export function addDays(date: IsoDate, days: number): IsoDate {
  return fromUtc(new Date(toUtc(date).getTime() + days * MILLISECONDS_PER_DAY))
}

/** Whether `value` is a real calendar date written as `YYYY-MM-DD`. */
export function isIsoDate(value: string): value is IsoDate {
  if (!ISO_DATE_PATTERN.test(value)) {
    return false
  }
  const parsed = toUtc(value)
  // `2026-02-30` parses to an invalid date or rolls over to March; both fail the round trip.
  return !Number.isNaN(parsed.getTime()) && fromUtc(parsed) === value
}

/** ISO weekday number: 1 (Monday) to 7 (Sunday), as stored in `habits.weekdays`. */
export function isoWeekday(date: IsoDate): number {
  return toUtc(date).getUTCDay() || 7
}

/** A full, localized date such as "Thursday, November 12, 2026". */
export function formatLongDate(date: IsoDate, language: string | undefined): string {
  return new Intl.DateTimeFormat(language, { dateStyle: 'full', timeZone: 'UTC' }).format(
    toUtc(date),
  )
}

/** Short localized weekday names, Monday first: index 0 is ISO weekday 1. */
export function weekdayNames(language: string | undefined): string[] {
  const format = new Intl.DateTimeFormat(language, { weekday: 'short', timeZone: 'UTC' })
  return Array.from({ length: 7 }, (_, index) =>
    format.format(new Date(A_MONDAY + index * MILLISECONDS_PER_DAY)),
  )
}

const ISO_MONTH_PATTERN = /^\d{4}-(0[1-9]|1[0-2])$/

/** Whether `value` is a real month written as `YYYY-MM`. */
export function isIsoMonth(value: string): value is IsoMonth {
  return ISO_MONTH_PATTERN.test(value)
}

/** The month a date belongs to. */
export function monthOf(date: IsoDate): IsoMonth {
  return date.slice(0, 7)
}

/** The month `delta` months after `month` (negative for before). */
export function addMonths(month: IsoMonth, delta: number): IsoMonth {
  const [year, monthNumber] = month.split('-').map(Number)
  const index = year * 12 + (monthNumber - 1) + delta
  const newYear = Math.floor(index / 12)
  const newMonth = (index % 12) + 1
  return `${String(newYear).padStart(4, '0')}-${String(newMonth).padStart(2, '0')}`
}

/** A localized month and year such as "November 2026". */
export function formatMonth(month: IsoMonth, language: string | undefined): string {
  return new Intl.DateTimeFormat(language, {
    month: 'long',
    year: 'numeric',
    timeZone: 'UTC',
  }).format(toUtc(`${month}-01`))
}

/** A short localized date with the weekday, such as "Sun, Nov 1". */
export function formatShortDate(date: IsoDate, language: string | undefined): string {
  return new Intl.DateTimeFormat(language, {
    weekday: 'short',
    day: 'numeric',
    month: 'short',
    timeZone: 'UTC',
  }).format(toUtc(date))
}

/** The Monday of the week that contains `date`; weeks start on Monday (SPEC.md). */
export function mondayOf(date: IsoDate): IsoDate {
  return addDays(date, 1 - isoWeekday(date))
}

/** A localized range for the week that starts on `start`, such as "Nov 9 – 15, 2026". */
export function formatWeekRange(start: IsoDate, language: string | undefined): string {
  return new Intl.DateTimeFormat(language, {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    timeZone: 'UTC',
  }).formatRange(toUtc(start), toUtc(addDays(start, 6)))
}

/** The time of day of an instant in `timeZone`, localized, such as "10:30 AM". */
export function formatTime(
  instant: string,
  language: string | undefined,
  timeZone: string,
): string {
  return new Intl.DateTimeFormat(language, { timeStyle: 'short', timeZone }).format(
    new Date(instant),
  )
}
