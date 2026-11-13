import { describe, expect, it } from 'vitest'
import { fractionDigits, formatMoney, parseMoneyInput, toInputValue } from './money'

// Intl output uses non-breaking spaces and locale-specific symbols, so compare digits only.
const digits = (text: string) => text.replace(/\D/g, '')

describe('fractionDigits', () => {
  it('follows the currency', () => {
    expect(fractionDigits('VND')).toBe(0)
    expect(fractionDigits('JPY')).toBe(0)
    expect(fractionDigits('KRW')).toBe(0)
    expect(fractionDigits('USD')).toBe(2)
    expect(fractionDigits('EUR')).toBe(2)
  })
})

describe('formatMoney', () => {
  it('shows whole dong without decimals', () => {
    const text = formatMoney(1500000, 'VND', 'en')

    expect(digits(text)).toBe('1500000')
    expect(text).toContain('₫')
  })

  it('treats the integer as cents for USD', () => {
    expect(formatMoney(123456, 'USD', 'en')).toBe('$1,234.56')
    expect(formatMoney(5, 'USD', 'en')).toBe('$0.05')
  })

  it('uses the grouping of the active language', () => {
    expect(formatMoney(123456, 'USD', 'vi')).toContain('1.234,56')
  })

  it('formats negative amounts', () => {
    expect(formatMoney(-250, 'USD', 'en')).toBe('-$2.50')
  })
})

describe('parseMoneyInput', () => {
  it('converts major units to minor units', () => {
    expect(parseMoneyInput('30000', 'VND')).toBe(30000)
    expect(parseMoneyInput('12.5', 'USD')).toBe(1250)
    expect(parseMoneyInput('12.34', 'USD')).toBe(1234)
    expect(parseMoneyInput('12', 'USD')).toBe(1200)
    expect(parseMoneyInput('0.05', 'USD')).toBe(5)
  })

  it('accepts a comma as the decimal separator and surrounding spaces', () => {
    expect(parseMoneyInput(' 12,5 ', 'USD')).toBe(1250)
  })

  it('rejects more decimals than the currency has', () => {
    expect(parseMoneyInput('12.345', 'USD')).toBeNull()
    expect(parseMoneyInput('100.5', 'VND')).toBeNull()
    expect(parseMoneyInput('100.0', 'JPY')).toBeNull()
  })

  it('rejects text that is not a plain number', () => {
    for (const text of ['', '  ', 'abc', '1e5', '-5', '+5', '1.2.3', '.', '1,000.50', '12 3']) {
      expect(parseMoneyInput(text, 'USD'), text).toBeNull()
    }
  })

  it('rejects amounts that are not safe integers in minor units', () => {
    expect(parseMoneyInput('99999999999999999999', 'VND')).toBeNull()
  })
})

describe('toInputValue', () => {
  it('writes minor units back as the text a user would type', () => {
    expect(toInputValue(30000, 'VND')).toBe('30000')
    expect(toInputValue(1250, 'USD')).toBe('12.50')
    expect(toInputValue(5, 'USD')).toBe('0.05')
  })

  it('round-trips through parseMoneyInput', () => {
    for (const [amount, currency] of [
      [1, 'USD'],
      [99, 'USD'],
      [100, 'USD'],
      [1234567, 'USD'],
      [30000, 'VND'],
    ] as const) {
      expect(parseMoneyInput(toInputValue(amount, currency), currency)).toBe(amount)
    }
  })
})
