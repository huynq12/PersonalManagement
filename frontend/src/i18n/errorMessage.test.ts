import { describe, expect, it } from 'vitest'
import { ApiError } from '../lib/api'
import { errorMessageKey } from './errorMessage'

describe('errorMessageKey', () => {
  it('maps a known API error code to its translation key', () => {
    const error = new ApiError(503, 'service_unavailable', 'service temporarily unavailable')

    expect(errorMessageKey(error)).toBe('errors.service_unavailable')
  })

  it('falls back to the generic key for an unknown API error code', () => {
    const error = new ApiError(418, 'teapot', 'i am a teapot')

    expect(errorMessageKey(error)).toBe('errors.unknown')
  })

  it('falls back to the generic key for errors that are not API errors', () => {
    expect(errorMessageKey(new Error('boom'))).toBe('errors.unknown')
  })
})
