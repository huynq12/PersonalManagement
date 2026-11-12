import { describe, expect, it } from 'vitest'
import { moveItem, sortOrderChanges } from './habitOrder'

const habits = [
  { id: 'a', sort_order: 0 },
  { id: 'b', sort_order: 1 },
  { id: 'c', sort_order: 2 },
]

describe('moveItem', () => {
  it('moves an item down past the item it was dropped on', () => {
    expect(moveItem(habits, 'a', 'c').map((habit) => habit.id)).toEqual(['b', 'c', 'a'])
  })

  it('moves an item up before the item it was dropped on', () => {
    expect(moveItem(habits, 'c', 'a').map((habit) => habit.id)).toEqual(['c', 'a', 'b'])
  })

  it('returns the same order when dropped on itself or on an unknown item', () => {
    expect(moveItem(habits, 'b', 'b')).toEqual(habits)
    expect(moveItem(habits, 'b', 'missing')).toEqual(habits)
  })
})

describe('sortOrderChanges', () => {
  it('numbers the new order from zero and reports only the items whose position changed', () => {
    const reordered = moveItem(habits, 'c', 'b')

    expect(sortOrderChanges(reordered)).toEqual([
      { id: 'c', sort_order: 1 },
      { id: 'b', sort_order: 2 },
    ])
  })

  it('reports nothing when the order is unchanged', () => {
    expect(sortOrderChanges(habits)).toEqual([])
  })

  it('repairs gaps and duplicates left by earlier changes', () => {
    const messy = [
      { id: 'a', sort_order: 0 },
      { id: 'b', sort_order: 0 },
      { id: 'c', sort_order: 7 },
    ]

    expect(sortOrderChanges(messy)).toEqual([
      { id: 'b', sort_order: 1 },
      { id: 'c', sort_order: 2 },
    ])
  })
})
