import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useUpdateProfile } from '../../hooks/useSettings'
import { errorMessageKey } from '../../i18n/errorMessage'
import i18n, { SUPPORTED_LANGUAGES } from '../../i18n/i18n'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { User } from '../../lib/auth'
import { profileFormSchema, type ProfileForm as ProfileFormValues } from '../../lib/settingsSchemas'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Card from '../ui/Card'
import Select from '../ui/Select'
import TextField from '../ui/TextField'

/** IANA time zones the browser knows, with the stored one added if the list lacks it. */
function timeZoneOptions(current: string): string[] {
  const zones = Intl.supportedValuesOf('timeZone')
  return zones.includes(current) ? zones : [current, ...zones].sort()
}

/** Edits the display name, time zone, and language; the email is shown but cannot change. */
export default function ProfileForm({ user }: { user: User }) {
  const { t } = useTranslation()
  const update = useUpdateProfile()

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<ProfileFormValues>({
    resolver: zodResolver(profileFormSchema),
    defaultValues: {
      displayName: user.display_name,
      timezone: user.timezone,
      locale: user.locale,
    },
  })

  const onSubmit = handleSubmit((values) => {
    update.mutate(
      {
        display_name: values.displayName,
        timezone: values.timezone,
        locale: values.locale,
      },
      // The saved language is also the language of the interface.
      { onSuccess: (saved) => void i18n.changeLanguage(saved.locale) },
    )
  })

  const nameError = errors.displayName?.message
  const selectClasses = 'h-10 px-3'

  return (
    <Card title={t('settings.profile.title')}>
      {update.isError ? <Alert variant="error">{t(errorMessageKey(update.error))}</Alert> : null}
      {update.isSuccess ? <Alert variant="success">{t('settings.profile.saved')}</Alert> : null}

      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <div className="flex flex-col gap-1">
          <span className="text-sm font-medium">{t('settings.profile.email')}</span>
          <span className="text-sm text-muted-foreground">{user.email}</span>
        </div>

        <TextField
          label={t('settings.profile.displayName')}
          autoComplete="name"
          error={nameError ? t(nameError as ValidationMessageKey) : undefined}
          {...register('displayName')}
        />

        <div className="flex flex-col gap-2">
          <label htmlFor="settings-language" className="text-sm font-medium">
            {t('settings.profile.language')}
          </label>
          <Select id="settings-language" className={selectClasses} {...register('locale')}>
            {SUPPORTED_LANGUAGES.map((language) => (
              <option key={language} value={language}>
                {t(`language.${language}`)}
              </option>
            ))}
          </Select>
        </div>

        <div className="flex flex-col gap-2">
          <label htmlFor="settings-timezone" className="text-sm font-medium">
            {t('settings.profile.timezone')}
          </label>
          <Select
            id="settings-timezone"
            className={selectClasses}
            aria-describedby="settings-timezone-hint"
            {...register('timezone')}
          >
            {timeZoneOptions(user.timezone).map((zone) => (
              <option key={zone} value={zone}>
                {zone}
              </option>
            ))}
          </Select>
          <p id="settings-timezone-hint" className="text-sm text-muted-foreground">
            {t('settings.profile.timezoneHint')}
          </p>
        </div>

        <div className="flex justify-end">
          <Button type="submit" disabled={update.isPending}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Card>
  )
}
