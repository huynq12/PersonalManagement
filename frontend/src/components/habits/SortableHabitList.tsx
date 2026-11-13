import {
  closestCenter,
  DndContext,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type Announcements,
  type DragEndEvent,
  type UniqueIdentifier,
} from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { Archive, BatteryLow, Flame, GripVertical, Pencil, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { Habit } from '../../lib/checklist'
import { weekdayNames } from '../../lib/dates'
import { moveItem } from '../../lib/habitOrder'
import IconButton from '../ui/IconButton'

interface SortableHabitListProps {
  /** Active habits in display order. */
  habits: Habit[]
  /** Called with the habits in their new order after a drop that changed it. */
  onReorder: (ordered: Habit[]) => void
  onEdit: (habit: Habit) => void
  onArchive: (habit: Habit) => void
  onDelete: (habit: Habit) => void
}

/**
 * The active habits as a list that can be reordered by dragging a row's handle, with the
 * mouse, by touch, or from the keyboard (Space to pick up, arrows to move, Space to drop).
 */
export default function SortableHabitList({
  habits,
  onReorder,
  onEdit,
  onArchive,
  onDelete,
}: SortableHabitListProps) {
  const { t } = useTranslation()
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  )

  const nameOf = (id: UniqueIdentifier) => habits.find((habit) => habit.id === id)?.name ?? ''
  const positionOf = (id: UniqueIdentifier) => habits.findIndex((habit) => habit.id === id) + 1

  // The library announces drag steps to screen readers in English by default; these replace
  // its texts with translated ones.
  const announcements: Announcements = {
    onDragStart: ({ active }) => t('habits.drag.pickedUp', { name: nameOf(active.id) }),
    onDragOver: ({ active, over }) =>
      over
        ? t('habits.drag.movedOver', {
            name: nameOf(active.id),
            position: positionOf(over.id),
            total: habits.length,
          })
        : undefined,
    onDragEnd: ({ active, over }) =>
      over
        ? t('habits.drag.dropped', {
            name: nameOf(active.id),
            position: positionOf(over.id),
            total: habits.length,
          })
        : t('habits.drag.cancelled', { name: nameOf(active.id) }),
    onDragCancel: ({ active }) => t('habits.drag.cancelled', { name: nameOf(active.id) }),
  }

  const onDragEnd = ({ active, over }: DragEndEvent) => {
    if (over && active.id !== over.id) {
      onReorder(moveItem(habits, String(active.id), String(over.id)))
    }
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={onDragEnd}
      accessibility={{
        announcements,
        screenReaderInstructions: { draggable: t('habits.drag.instructions') },
      }}
    >
      <SortableContext items={habits.map((habit) => habit.id)} strategy={verticalListSortingStrategy}>
        <ul aria-label={t('habits.title')} className="flex flex-col gap-2">
          {habits.map((habit) => (
            <SortableHabitRow
              key={habit.id}
              habit={habit}
              onEdit={onEdit}
              onArchive={onArchive}
              onDelete={onDelete}
            />
          ))}
        </ul>
      </SortableContext>
    </DndContext>
  )
}

interface SortableHabitRowProps {
  habit: Habit
  onEdit: (habit: Habit) => void
  onArchive: (habit: Habit) => void
  onDelete: (habit: Habit) => void
}

function SortableHabitRow({ habit, onEdit, onArchive, onDelete }: SortableHabitRowProps) {
  const { t, i18n } = useTranslation()
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } =
    useSortable({ id: habit.id })

  return (
    <li
      ref={setNodeRef}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      className={`flex items-center gap-2 rounded-xl border border-border bg-background p-3 ${
        isDragging ? 'relative z-10 shadow-lg' : ''
      }`}
    >
      {/* Only the handle starts a drag, so the row's buttons and text stay clickable. */}
      <button
        ref={setActivatorNodeRef}
        type="button"
        aria-label={t('habits.reorder', { name: habit.name })}
        title={t('habits.reorder', { name: habit.name })}
        className="flex size-9 shrink-0 cursor-grab touch-none items-center justify-center rounded-md text-muted-foreground hover:bg-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring active:cursor-grabbing"
        {...attributes}
        {...listeners}
      >
        <GripVertical className="size-4" aria-hidden="true" />
      </button>

      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="truncate text-sm font-medium">{habit.name}</span>
        <span className="flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-muted-foreground">
          <span>{scheduleSummary(habit.weekdays, i18n.resolvedLanguage, t('habits.everyDay'))}</span>
          <span className="flex items-center gap-1">
            <Flame className="size-3.5" aria-hidden="true" />
            {t('habits.streak', { count: habit.current_streak })}
          </span>
          {habit.low_energy ? (
            <span className="flex items-center gap-1">
              <BatteryLow className="size-3.5" aria-hidden="true" />
              {t('habits.keptOnLowEnergy')}
            </span>
          ) : null}
        </span>
        {habit.low_energy && habit.low_energy_name ? (
          <span className="truncate text-xs text-muted-foreground">
            {t('habits.easierVersion', { name: habit.low_energy_name })}
          </span>
        ) : null}
      </div>

      <IconButton
        icon={Pencil}
        label={t('habits.edit', { name: habit.name })}
        onClick={() => onEdit(habit)}
      />
      <IconButton
        icon={Archive}
        label={t('habits.archive', { name: habit.name })}
        onClick={() => onArchive(habit)}
      />
      <IconButton
        icon={Trash2}
        label={t('habits.delete', { name: habit.name })}
        onClick={() => onDelete(habit)}
      />
    </li>
  )
}

/** "Every day", or the localized weekday names such as "Mon, Wed, Fri". */
function scheduleSummary(weekdays: number[], language: string | undefined, everyDay: string) {
  if (weekdays.length === 7) {
    return everyDay
  }
  const names = weekdayNames(language)
  return new Intl.ListFormat(language, { style: 'short', type: 'unit' }).format(
    weekdays.map((weekday) => names[weekday - 1]),
  )
}
