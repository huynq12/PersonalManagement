import { describe, expect, it } from 'vitest'
import { workEntryFormSchema } from './workEntrySchemas'

const TODAY = '2026-11-14'
const schema = workEntryFormSchema(TODAY)

function messages(values: { date: string; hours: string; note: string }): string[] {
  const result = schema.safeParse(values)
  return result.success ? [] : result.error.issues.map((issue) => issue.message)
}

describe('workEntryFormSchema', () => {
  it('accepts a valid entry, with comma or dot decimals', () => {
    expect(messages({ date: TODAY, hours: '7.5', note: '' })).toEqual([])
    expect(messages({ date: '2026-01-01', hours: '7,25', note: 'Inventory' })).toEqual([])
    expect(messages({ date: TODAY, hours: '24', note: '' })).toEqual([])
  })

  it('requires the hours', () => {
    expect(messages({ date: TODAY, hours: '  ', note: '' })).toEqual(['validation.required'])
  })

  it('rejects hours out of range or with too many decimals', () => {
    for (const hours of ['0', '24.01', '7.255', 'abc', '-1']) {
      expect(messages({ date: TODAY, hours, note: '' }), hours).toEqual(['validation.hoursInvalid'])
    }
  })

  it('rejects a missing, impossible, or future date', () => {
    expect(messages({ date: '', hours: '8', note: '' })).toEqual(['validation.dateInvalid'])
    expect(messages({ date: '2026-02-30', hours: '8', note: '' })).toEqual([
      'validation.dateInvalid',
    ])
    expect(messages({ date: '2026-11-15', hours: '8', note: '' })).toEqual([
      'validation.dateInFuture',
    ])
  })

  it('rejects a note longer than 500 characters', () => {
    expect(messages({ date: TODAY, hours: '8', note: 'x'.repeat(501) })).toEqual([
      'validation.noteLength',
    ])
    expect(messages({ date: TODAY, hours: '8', note: 'x'.repeat(500) })).toEqual([])
  })
})
