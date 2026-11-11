import { ApiError } from '../lib/api'
import en from './locales/en.json'

type ErrorCode = keyof typeof en.errors
export type ErrorMessageKey = `errors.${ErrorCode}`

function isKnownErrorCode(code: string): code is ErrorCode {
  return Object.hasOwn(en.errors, code)
}

/**
 * Returns the translation key for an error shown to the user.
 *
 * The backend's own `message` is developer-facing English and is never displayed; the stable
 * error `code` selects the localized text, and unknown codes get a generic message.
 */
export function errorMessageKey(error: unknown): ErrorMessageKey {
  if (error instanceof ApiError && isKnownErrorCode(error.code)) {
    return `errors.${error.code}`
  }
  return 'errors.unknown'
}
