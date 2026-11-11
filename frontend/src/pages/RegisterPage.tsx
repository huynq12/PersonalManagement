import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useNavigate } from 'react-router'
import AuthLayout from '../components/AuthLayout'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import TextField from '../components/ui/TextField'
import { useRegister } from '../hooks/useSession'
import { errorMessageKey } from '../i18n/errorMessage'
import {
  registerFormSchema,
  type RegisterForm,
  type ValidationMessageKey,
} from '../lib/authSchemas'
import type { LoginLocationState } from './LoginPage'

export default function RegisterPage() {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const registerAccount = useRegister()

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<RegisterForm>({
    resolver: zodResolver(registerFormSchema),
    defaultValues: { email: '', displayName: '', password: '' },
  })

  const onSubmit = handleSubmit((values) => {
    registerAccount.mutate(
      {
        email: values.email,
        password: values.password,
        // The backend derives a name from the email when none is given.
        ...(values.displayName ? { display_name: values.displayName } : {}),
        locale: i18n.resolvedLanguage ?? 'en',
      },
      {
        // Registration does not sign in: continue on the login page with the email filled in.
        onSuccess: (user) => {
          const state: LoginLocationState = { registeredEmail: user.email }
          void navigate('/login', { state })
        },
      },
    )
  })

  const fieldError = (message: string | undefined) =>
    message ? t(message as ValidationMessageKey) : undefined

  return (
    <AuthLayout title={t('auth.register.title')}>
      {registerAccount.isError ? (
        <Alert variant="error">{t(errorMessageKey(registerAccount.error))}</Alert>
      ) : null}

      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <TextField
          label={t('auth.fields.email')}
          type="email"
          autoComplete="email"
          error={fieldError(errors.email?.message)}
          {...register('email')}
        />
        <TextField
          label={t('auth.fields.displayName')}
          autoComplete="name"
          error={fieldError(errors.displayName?.message)}
          {...register('displayName')}
        />
        <TextField
          label={t('auth.fields.password')}
          type="password"
          autoComplete="new-password"
          hint={t('auth.fields.passwordHint')}
          error={fieldError(errors.password?.message)}
          {...register('password')}
        />
        <Button type="submit" disabled={registerAccount.isPending}>
          {t('auth.register.submit')}
        </Button>
      </form>

      <p className="text-center text-sm">
        {t('auth.register.haveAccount')}{' '}
        <Link className="font-medium text-indigo-600 underline dark:text-indigo-400" to="/login">
          {t('auth.register.loginLink')}
        </Link>
      </p>
    </AuthLayout>
  )
}
