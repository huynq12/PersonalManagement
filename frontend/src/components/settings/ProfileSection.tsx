import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useUpdateAccount } from '../../hooks/useSettings'
import type { User } from '../../lib/auth'
import {
  profileFormSchema,
  type ProfileForm,
  type ValidationMessageKey,
} from '../../lib/authSchemas'
import Button from '../ui/Button'
import Card from '../ui/Card'
import TextField from '../ui/TextField'
import SaveStatus from './SaveStatus'

interface ProfileSectionProps {
  user: User
}

/** The display name. The email is shown for reference; it cannot be changed. */
export default function ProfileSection({ user }: ProfileSectionProps) {
  const { t } = useTranslation()
  const save = useUpdateAccount()
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<ProfileForm>({
    resolver: zodResolver(profileFormSchema),
    defaultValues: { displayName: user.display_name },
  })

  const message = errors.displayName?.message
  const onSubmit = handleSubmit((values) => {
    save.mutate({ display_name: values.displayName.trim() })
  })

  return (
    <Card title={t('settings.profile.title')}>
      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <div className="max-w-sm">
          <TextField
            label={t('settings.profile.displayName')}
            hint={t('settings.profile.email', { email: user.email })}
            autoComplete="name"
            error={message ? t(message as ValidationMessageKey) : undefined}
            {...register('displayName')}
          />
        </div>

        <SaveStatus isError={save.isError} isSuccess={save.isSuccess} error={save.error} />

        <div>
          <Button type="submit" disabled={save.isPending}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Card>
  )
}
