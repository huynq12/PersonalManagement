import { zodResolver } from '@hookform/resolvers/zod'
import { Pencil, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useCreateTask, useDeleteTask, useRenameTask, useToggleTask } from '../../hooks/useDay'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { Task } from '../../lib/checklist'
import { taskFormSchema, type TaskForm } from '../../lib/checklistSchemas'
import type { IsoDate } from '../../lib/dates'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Card from '../ui/Card'
import Checkbox from '../ui/Checkbox'
import IconButton from '../ui/IconButton'

const INPUT_CLASSES =
  'h-10 min-w-0 flex-1 rounded-md border border-input bg-background px-3 text-sm text-foreground outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring aria-invalid:border-destructive'

interface TaskListProps {
  date: IsoDate
  tasks: Task[]
}

/** The to-dos of one day: check, rename, delete, and add. */
export default function TaskList({ date, tasks }: TaskListProps) {
  const { t } = useTranslation()
  const toggle = useToggleTask(date)
  const rename = useRenameTask()
  const remove = useDeleteTask()
  const create = useCreateTask(date)
  // At most one to-do is being renamed at a time.
  const [editingId, setEditingId] = useState<string | null>(null)

  const done = tasks.filter((task) => task.is_done).length
  const failure = [toggle, rename, remove, create].find((mutation) => mutation.isError)

  return (
    <Card
      title={t('home.tasks.title')}
      aside={tasks.length > 0 ? t('home.progress', { done, total: tasks.length }) : null}
    >
      {failure ? <Alert variant="error">{t(errorMessageKey(failure.error))}</Alert> : null}

      {tasks.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t('home.tasks.empty')}</p>
      ) : (
        <ul className="flex flex-col">
          {tasks.map((task) => (
            <li key={task.id} className="flex min-h-11 items-center gap-1">
              {editingId === task.id ? (
                <TaskTitleForm
                  initialTitle={task.title}
                  onCancel={() => setEditingId(null)}
                  onSubmit={(title) => {
                    setEditingId(null)
                    if (title !== task.title) {
                      rename.mutate({ taskId: task.id, title })
                    }
                  }}
                />
              ) : (
                <>
                  <label className="flex min-h-11 min-w-0 flex-1 items-center gap-3 rounded-md px-1 hover:bg-accent">
                    <Checkbox
                      checked={task.is_done}
                      onChange={(event) =>
                        toggle.mutate({ taskId: task.id, isDone: event.target.checked })
                      }
                    />
                    <span
                      className={
                        task.is_done
                          ? 'truncate text-sm text-muted-foreground line-through'
                          : 'truncate text-sm'
                      }
                    >
                      {task.title}
                    </span>
                  </label>
                  <IconButton
                    icon={Pencil}
                    label={t('home.tasks.edit', { title: task.title })}
                    onClick={() => setEditingId(task.id)}
                  />
                  <IconButton
                    icon={Trash2}
                    label={t('home.tasks.delete', { title: task.title })}
                    disabled={remove.isPending}
                    onClick={() => remove.mutate(task.id)}
                  />
                </>
              )}
            </li>
          ))}
        </ul>
      )}

      <NewTaskForm isSaving={create.isPending} onSubmit={(title) => create.mutateAsync(title)} />
    </Card>
  )
}

interface NewTaskFormProps {
  isSaving: boolean
  /** Saves the to-do; the field is cleared only once this resolves. */
  onSubmit: (title: string) => Promise<unknown>
}

function NewTaskForm({ isSaving, onSubmit }: NewTaskFormProps) {
  const { t } = useTranslation()
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<TaskForm>({
    resolver: zodResolver(taskFormSchema),
    defaultValues: { title: '' },
  })

  const submit = handleSubmit(async ({ title }) => {
    try {
      await onSubmit(title)
      reset()
    } catch {
      // The list shows the error; keep the text so it can be sent again.
    }
  })

  // An empty field is not worth an error message: nothing was typed.
  const error = errors.title?.message === 'validation.required' ? undefined : errors.title?.message

  return (
    <form className="flex flex-col gap-1.5" noValidate onSubmit={(event) => void submit(event)}>
      <div className="flex items-center gap-2">
        <input
          aria-label={t('home.tasks.newLabel')}
          aria-invalid={error ? true : undefined}
          placeholder={t('home.tasks.newPlaceholder')}
          autoComplete="off"
          className={INPUT_CLASSES}
          {...register('title')}
        />
        <Button type="submit" disabled={isSaving}>
          <Plus className="size-4" aria-hidden="true" />
          {t('home.tasks.add')}
        </Button>
      </div>
      {error ? (
        <p className="text-sm text-destructive">{t(error as ValidationMessageKey)}</p>
      ) : null}
    </form>
  )
}

interface TaskTitleFormProps {
  initialTitle: string
  onSubmit: (title: string) => void
  onCancel: () => void
}

/** Replaces a to-do row while it is being renamed. Enter saves, Escape cancels. */
function TaskTitleForm({ initialTitle, onSubmit, onCancel }: TaskTitleFormProps) {
  const { t } = useTranslation()
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<TaskForm>({
    resolver: zodResolver(taskFormSchema),
    defaultValues: { title: initialTitle },
  })

  const submit = handleSubmit(({ title }) => onSubmit(title))
  const error = errors.title?.message

  return (
    <form
      className="flex min-w-0 flex-1 flex-col gap-1.5 py-1"
      noValidate
      onSubmit={(event) => void submit(event)}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          onCancel()
        }
      }}
    >
      <div className="flex flex-wrap items-center gap-2">
        <input
          aria-label={t('home.tasks.titleLabel')}
          aria-invalid={error ? true : undefined}
          autoComplete="off"
          // The user asked to edit this row, so moving focus here is expected.
          autoFocus
          className={INPUT_CLASSES}
          {...register('title')}
        />
        <Button type="submit">{t('common.save')}</Button>
        <Button variant="ghost" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
      </div>
      {error ? (
        <p className="text-sm text-destructive">{t(error as ValidationMessageKey)}</p>
      ) : null}
    </form>
  )
}
