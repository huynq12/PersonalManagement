import { ArchiveRestore, ChevronDown, ChevronRight, Trash2 } from 'lucide-react'
import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { Habit } from '../../lib/checklist'
import IconButton from '../ui/IconButton'

interface ArchivedHabitsProps {
  habits: Habit[]
  onRestore: (habit: Habit) => void
  onDelete: (habit: Habit) => void
}

/** The archived habits, collapsed until the user asks to see them. */
export default function ArchivedHabits({ habits, onRestore, onDelete }: ArchivedHabitsProps) {
  const { t } = useTranslation()
  const [isOpen, setIsOpen] = useState(false)
  const listId = useId()
  const Chevron = isOpen ? ChevronDown : ChevronRight

  return (
    <section className="flex flex-col gap-2">
      <button
        type="button"
        aria-expanded={isOpen}
        aria-controls={listId}
        onClick={() => setIsOpen((open) => !open)}
        className="flex h-9 items-center gap-1.5 self-start rounded-md px-2 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
      >
        <Chevron className="size-4" aria-hidden="true" />
        {t('habits.archived', { count: habits.length })}
      </button>

      {isOpen ? (
        <ul id={listId} className="flex flex-col gap-2">
          {habits.map((habit) => (
            <li
              key={habit.id}
              className="flex items-center gap-2 rounded-xl border border-dashed border-border p-3"
            >
              <span className="min-w-0 flex-1 truncate text-sm text-muted-foreground">
                {habit.name}
              </span>
              <IconButton
                icon={ArchiveRestore}
                label={t('habits.restore', { name: habit.name })}
                onClick={() => onRestore(habit)}
              />
              <IconButton
                icon={Trash2}
                label={t('habits.delete', { name: habit.name })}
                onClick={() => onDelete(habit)}
              />
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  )
}
