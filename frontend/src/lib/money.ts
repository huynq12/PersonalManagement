/** Currencies a user can choose; the same list as the backend's `users_currency_supported`. */
export const SUPPORTED_CURRENCIES = [
  'VND',
  'USD',
  'EUR',
  'GBP',
  'JPY',
  'KRW',
  'CNY',
  'SGD',
  'THB',
  'AUD',
] as const

export type Currency = (typeof SUPPORTED_CURRENCIES)[number]

/** Largest wage or budget the backend accepts, in minor units. */
export const MAX_MONEY = 1_000_000_000_000

// Money is an integer in the currency's minor unit (dong for VND, cents for USD) everywhere
// except in text boxes, where people type major units. These helpers convert at that edge.

/** Number of decimals the currency shows: 0 for VND, JPY and KRW, 2 for USD and EUR. */
export function fractionDigits(currency: string): number {
  return (
    new Intl.NumberFormat('en', { style: 'currency', currency }).resolvedOptions()
      .maximumFractionDigits ?? 2
  )
}

/** Formats an amount in minor units for display, using the active language's conventions. */
export function formatMoney(amountMinor: number, currency: string, language: string): string {
  const major = amountMinor / 10 ** fractionDigits(currency)
  return new Intl.NumberFormat(language, { style: 'currency', currency }).format(major)
}

/**
 * Converts what the user typed (major units) to minor units. Returns `null` when the text is
 * not a plain non-negative number or has more decimals than the currency allows.
 *
 * Only digits and one decimal separator (`.` or `,`) are accepted. Grouping separators are
 * rejected rather than guessed at, because "1.000" means one thousand in Vietnamese and one
 * in English.
 */
export function parseMoneyInput(text: string, currency: string): number | null {
  const match = /^(\d+)(?:[.,](\d+))?$/.exec(text.trim())
  if (!match) {
    return null
  }

  const digits = fractionDigits(currency)
  const [, whole, fraction = ''] = match
  if (fraction.length > digits) {
    return null
  }

  const minor = Number(whole + fraction.padEnd(digits, '0'))
  return Number.isSafeInteger(minor) ? minor : null
}

/** Writes an amount in minor units as the text a user would type, such as `12.50`. */
export function toInputValue(amountMinor: number, currency: string): string {
  return (amountMinor / 10 ** fractionDigits(currency)).toFixed(fractionDigits(currency))
}
