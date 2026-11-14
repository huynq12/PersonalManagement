import { describe, expect, it } from 'vitest'
import i18n from '../i18n/i18n'
import { formatDuration, splitDuration } from './duration'

describe('splitDuration', () => {
  it('rounds to the nearest minute and carries minutes into hours', () => {
    expect(splitDuration(0)).toEqual({ hours: 0, minutes: 0 })
    expect(splitDuration(29)).toEqual({ hours: 0, minutes: 0 })
    expect(splitDuration(90)).toEqual({ hours: 0, minutes: 2 })
    expect(splitDuration(3599)).toEqual({ hours: 1, minutes: 0 })
    expect(splitDuration(5100)).toEqual({ hours: 1, minutes: 25 })
  })
})

describe('formatDuration', () => {
  it('writes minutes, whole hours, or both in the active language', async () => {
    await i18n.changeLanguage('en')
    expect(formatDuration(2400, i18n.t)).toBe('40 min')
    expect(formatDuration(7200, i18n.t)).toBe('2 h')
    expect(formatDuration(5100, i18n.t)).toBe('1 h 25 min')
  })
})
