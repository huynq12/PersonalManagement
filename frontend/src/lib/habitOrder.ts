interface Sortable {
  id: string
  sort_order: number
}

/** A new `sort_order` to save for one item. */
export interface SortOrderChange {
  id: string
  sort_order: number
}

/**
 * Returns the list with the item `activeId` moved to the position of the item `overId`, the
 * way a drag-and-drop list behaves: the dragged item takes the slot of the one it is dropped on.
 */
export function moveItem<T extends Sortable>(items: T[], activeId: string, overId: string): T[] {
  const from = items.findIndex((item) => item.id === activeId)
  const to = items.findIndex((item) => item.id === overId)
  if (from === -1 || to === -1 || from === to) {
    return items
  }

  const next = [...items]
  const [moved] = next.splice(from, 1)
  next.splice(to, 0, moved)
  return next
}

/**
 * Numbers the items 0, 1, 2, ... in their current order and returns the ones whose stored
 * `sort_order` differs, so only those need to be saved.
 */
export function sortOrderChanges(items: Sortable[]): SortOrderChange[] {
  return items.flatMap((item, index) =>
    item.sort_order === index ? [] : [{ id: item.id, sort_order: index }],
  )
}
