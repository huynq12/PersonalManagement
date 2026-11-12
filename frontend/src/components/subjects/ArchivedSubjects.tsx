import { ArchiveRestore, ChevronDown, ChevronRight, Trash2 } from 'lucide-react'
import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { Subject } from '../../lib/subjects'
import IconButton from '../ui/IconButton'
import ColorDot from './ColorDot'

interface ArchivedSubjectsProps {
  subjects: Subject[]
  onRestore: (subject: Subject) => void
  onDelete: (subject: Subject) => void
}

/** The archived subjects, collapsed until the user asks to see them. */
export default function ArchivedSubjects({
  subjects,
  onRestore,
  onDelete,
}: ArchivedSubjectsProps) {
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
        {t('subjects.archived', { count: subjects.length })}
      </button>

      {isOpen ? (
        <ul id={listId} className="flex flex-col gap-2">
          {subjects.map((subject) => (
            <li
              key={subject.id}
              className="flex items-center gap-2 rounded-xl border border-dashed border-border p-3"
            >
              <ColorDot color={subject.color} />
              <span className="min-w-0 flex-1 truncate text-sm text-muted-foreground">
                {subject.name}
              </span>
              <IconButton
                icon={ArchiveRestore}
                label={t('subjects.restore', { name: subject.name })}
                onClick={() => onRestore(subject)}
              />
              <IconButton
                icon={Trash2}
                label={t('subjects.delete', { name: subject.name })}
                onClick={() => onDelete(subject)}
              />
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  )
}
