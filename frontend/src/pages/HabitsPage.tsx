import { Plus } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import ArchivedHabits from '../components/habits/ArchivedHabits'
import DeleteHabitDialog from '../components/habits/DeleteHabitDialog'
import HabitFormDialog from '../components/habits/HabitFormDialog'
import SortableHabitList from '../components/habits/SortableHabitList'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import { useHabits, useReorderHabits, useUpdateHabit } from '../hooks/useHabits'
import { errorMessageKey } from '../i18n/errorMessage'
import type { Habit } from '../lib/checklist'

/** Which dialog is open, if any. Only one can be open at a time. */
type OpenDialog =
  | { kind: 'add' }
  | { kind: 'edit'; habit: Habit }
  | { kind: 'delete'; habit: Habit }
  | null

/** The Habits screen: add, edit, reorder, archive, restore, and delete habits. */
export default function HabitsPage() {
  const { t } = useTranslation()
  const habits = useHabits()
  const update = useUpdateHabit()
  const reorder = useReorderHabits()
  const [dialog, setDialog] = useState<OpenDialog>(null)

  const closeDialog = () => setDialog(null)
  const setArchived = (habit: Habit, archived: boolean) =>
    update.mutate({ id: habit.id, changes: { archived } })

  const active = habits.data?.filter((habit) => habit.archived_at === null) ?? []
  const archived = habits.data?.filter((habit) => habit.archived_at !== null) ?? []
  const failure = [update, reorder].find((mutation) => mutation.isError)

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t('habits.title')}</h1>
          <p className="text-sm text-muted-foreground">{t('habits.description')}</p>
        </div>
        <Button onClick={() => setDialog({ kind: 'add' })}>
          <Plus className="size-4" aria-hidden="true" />
          {t('habits.add')}
        </Button>
      </header>

      {failure ? <Alert variant="error">{t(errorMessageKey(failure.error))}</Alert> : null}

      {habits.isPending ? (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
      ) : habits.isError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(habits.error))}</Alert>
          <Button variant="secondary" onClick={() => void habits.refetch()}>
            {t('common.retry')}
          </Button>
        </div>
      ) : (
        <>
          {active.length === 0 ? (
            <p className="rounded-xl border border-dashed border-border px-6 py-12 text-center text-sm text-muted-foreground">
              {t('habits.empty')}
            </p>
          ) : (
            <SortableHabitList
              habits={active}
              onReorder={(ordered) => reorder.mutate(ordered)}
              onEdit={(habit) => setDialog({ kind: 'edit', habit })}
              onArchive={(habit) => setArchived(habit, true)}
              onDelete={(habit) => setDialog({ kind: 'delete', habit })}
            />
          )}

          {archived.length > 0 ? (
            <ArchivedHabits
              habits={archived}
              onRestore={(habit) => setArchived(habit, false)}
              onDelete={(habit) => setDialog({ kind: 'delete', habit })}
            />
          ) : null}
        </>
      )}

      {dialog?.kind === 'add' ? <HabitFormDialog onClose={closeDialog} /> : null}
      {dialog?.kind === 'edit' ? (
        <HabitFormDialog habit={dialog.habit} onClose={closeDialog} />
      ) : null}
      {dialog?.kind === 'delete' ? (
        <DeleteHabitDialog habit={dialog.habit} onClose={closeDialog} />
      ) : null}
    </main>
  )
}
