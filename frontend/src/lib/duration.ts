import type { TFunction } from 'i18next'

/** Splits a length of time into whole hours and the minutes left, rounding to the minute. */
export function splitDuration(seconds: number): { hours: number; minutes: number } {
  const totalMinutes = Math.round(Math.abs(seconds) / 60)
  return { hours: Math.floor(totalMinutes / 60), minutes: totalMinutes % 60 }
}

/** A localized length of time such as "1 h 25 min", "2 h", or "40 min". */
export function formatDuration(seconds: number, t: TFunction): string {
  const { hours, minutes } = splitDuration(seconds)
  if (hours === 0) {
    return t('stats.duration.minutes', { count: minutes })
  }
  return minutes === 0
    ? t('stats.duration.hours', { hours })
    : t('stats.duration.hoursMinutes', { hours, minutes })
}
