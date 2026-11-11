import { afterEach, describe, expect, it, vi } from 'vitest'
import { z } from 'zod'
import { errorResponse, jsonResponse, stubApi } from '../test/testUtils'
import { ApiError, apiRequest, setAccessToken, setUnauthorizedHandler } from './api'

const itemSchema = z.object({ name: z.string() })

afterEach(() => {
  vi.unstubAllGlobals()
  setAccessToken(null)
  setUnauthorizedHandler(null)
})

describe('apiRequest', () => {
  it('sends the access token as a bearer token when one is set', async () => {
    const requests = stubApi({ 'GET /items': () => jsonResponse(200, { name: 'tea' }) })
    setAccessToken('token-1')

    await apiRequest('/items', itemSchema)

    expect(requests[0].authorization).toBe('Bearer token-1')
  })

  it('sends no Authorization header without an access token', async () => {
    const requests = stubApi({ 'GET /items': () => jsonResponse(200, { name: 'tea' }) })

    await apiRequest('/items', itemSchema)

    expect(requests[0].authorization).toBeNull()
  })

  it('sends a JSON body with the given method', async () => {
    const requests = stubApi({ 'POST /items': () => jsonResponse(201, { name: 'tea' }) })

    await apiRequest('/items', itemSchema, { method: 'POST', body: { name: 'tea' } })

    expect(requests[0].method).toBe('POST')
    expect(requests[0].body).toEqual({ name: 'tea' })
  })

  it('refreshes the session and retries once after an unauthorized response', async () => {
    const requests = stubApi({
      'GET /items': (request) =>
        request.authorization === 'Bearer fresh-token'
          ? jsonResponse(200, { name: 'tea' })
          : errorResponse(401, 'unauthorized'),
    })
    setAccessToken('expired-token')
    setUnauthorizedHandler(async () => {
      setAccessToken('fresh-token')
      return true
    })

    const item = await apiRequest('/items', itemSchema)

    expect(item).toEqual({ name: 'tea' })
    expect(requests.map((request) => request.authorization)).toEqual([
      'Bearer expired-token',
      'Bearer fresh-token',
    ])
  })

  it('gives up with the unauthorized error when the session cannot be refreshed', async () => {
    const requests = stubApi({ 'GET /items': () => errorResponse(401, 'unauthorized') })
    setUnauthorizedHandler(async () => false)

    const failure = await apiRequest('/items', itemSchema).catch((error: unknown) => error)

    expect(failure).toBeInstanceOf(ApiError)
    expect((failure as ApiError).code).toBe('unauthorized')
    expect(requests).toHaveLength(1)
  })

  it('does not retry more than once', async () => {
    const requests = stubApi({ 'GET /items': () => errorResponse(401, 'unauthorized') })
    setUnauthorizedHandler(async () => true)

    await apiRequest('/items', itemSchema).catch(() => null)

    expect(requests).toHaveLength(2)
  })

  it('rejects a response that does not match the schema', async () => {
    stubApi({ 'GET /items': () => jsonResponse(200, { name: 42 }) })

    await expect(apiRequest('/items', itemSchema)).rejects.toThrow()
  })
})
