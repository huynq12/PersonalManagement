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
  return error instanceof ApiError ? errorCodeMessageKey(error.code) : 'errors.unknown'
}

/**
 * Returns the translation key for a bare error code, such as the `?error=` parameter the
 * backend adds when it redirects back from Google. The code is untrusted input: anything that
 * is not a known code gets the generic message.
 */
export function errorCodeMessageKey(code: string): ErrorMessageKey {
  return isKnownErrorCode(code) ? `errors.${code}` : 'errors.unknown'
}
