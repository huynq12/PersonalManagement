import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { setAccessToken } from '../lib/api'
import { HABITS_QUERY_KEY, type Habit } from '../lib/checklist'
import { FakeChecklist, habit } from '../test/fakeChecklist'
import { stubApi } from '../test/testUtils'
import { useReorderHabits } from './useHabits'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setAccessToken(null)
})

function setup(backend: FakeChecklist) {
  const requests = stubApi({}, backend.handle)
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  queryClient.setQueryData(HABITS_QUERY_KEY, backend.habits)
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  const { result } = renderHook(() => useReorderHabits(), { wrapper })
  const cachedNames = () =>
    queryClient.getQueryData<Habit[]>(HABITS_QUERY_KEY)?.map((item) => item.name)

  return { requests, result, cachedNames }
}

describe('useReorderHabits', () => {
  it('shows the new order at once and saves only the habits that moved', async () => {
    const backend = new FakeChecklist([
      habit('a', 'First', { sort_order: 0 }),
      habit('b', 'Second', { sort_order: 1 }),
      habit('c', 'Third', { sort_order: 2 }),
    ])
    const { requests, result, cachedNames } = setup(backend)
    const [first, second, third] = backend.habits

    result.current.mutate([first, third, second])

    await waitFor(() => expect(cachedNames()).toEqual(['First', 'Third', 'Second']))
    await waitFor(() => expect(result.current.isSuccess).toBe(true))
    const saved = requests.filter((request) => request.method === 'PATCH')
    expect(saved.map((request) => [request.path, request.body])).toEqual([
      ['/habits/c', { sort_order: 1 }],
      ['/habits/b', { sort_order: 2 }],
    ])
  })

  it('restores the previous order when saving fails', async () => {
    const backend = new FakeChecklist([
      habit('a', 'First', { sort_order: 0 }),
      habit('b', 'Second', { sort_order: 1 }),
    ])
    backend.failWhen = (request) => request.method === 'PATCH'
    const { result, cachedNames } = setup(backend)
    const [first, second] = backend.habits

    result.current.mutate([second, first])

    await waitFor(() => expect(result.current.isError).toBe(true))
    await waitFor(() => expect(cachedNames()).toEqual(['First', 'Second']))
  })
})
