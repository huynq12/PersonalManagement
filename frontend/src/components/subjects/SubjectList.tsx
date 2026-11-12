import { Archive, Pencil, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { Subject } from '../../lib/subjects'
import IconButton from '../ui/IconButton'
import ColorDot from './ColorDot'

interface SubjectListProps {
  subjects: Subject[]
  onEdit: (subject: Subject) => void
  onArchive: (subject: Subject) => void
  onDelete: (subject: Subject) => void
}

/** The active subjects, each with its color and the actions on it. */
export default function SubjectList({ subjects, onEdit, onArchive, onDelete }: SubjectListProps) {
  const { t } = useTranslation()

  return (
    <ul aria-label={t('subjects.title')} className="flex flex-col gap-2">
      {subjects.map((subject) => (
        <li key={subject.id} className="flex items-center gap-2 rounded-xl border border-border p-3">
          <ColorDot color={subject.color} />
          <span className="min-w-0 flex-1 truncate text-sm font-medium">{subject.name}</span>
          <IconButton
            icon={Pencil}
            label={t('subjects.edit', { name: subject.name })}
            onClick={() => onEdit(subject)}
          />
          <IconButton
            icon={Archive}
            label={t('subjects.archive', { name: subject.name })}
            onClick={() => onArchive(subject)}
          />
          <IconButton
            icon={Trash2}
            label={t('subjects.delete', { name: subject.name })}
            onClick={() => onDelete(subject)}
          />
        </li>
      ))}
    </ul>
  )
}
