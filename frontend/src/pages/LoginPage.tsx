import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useLocation, useSearchParams } from 'react-router'
import AuthLayout from '../components/AuthLayout'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import { BUTTON_CLASSES } from '../components/ui/buttonClasses'
import TextField from '../components/ui/TextField'
import { useLogin } from '../hooks/useSession'
import { errorCodeMessageKey, errorMessageKey } from '../i18n/errorMessage'
import { googleSignInUrl } from '../lib/auth'
import { loginFormSchema, type LoginForm, type ValidationMessageKey } from '../lib/authSchemas'

/** Router state the registration page passes along after creating an account. */
export interface LoginLocationState {
  registeredEmail: string
}

function isLoginLocationState(state: unknown): state is LoginLocationState {
  return (
    typeof state === 'object' &&
    state !== null &&
    'registeredEmail' in state &&
    typeof state.registeredEmail === 'string'
  )
}

export default function LoginPage() {
  const { t, i18n } = useTranslation()
  const location = useLocation()
  const [searchParams] = useSearchParams()
  const login = useLogin()

  const registeredEmail = isLoginLocationState(location.state)
    ? location.state.registeredEmail
    : null
  // Set by the backend when Google sign-in fails and it redirects back here.
  const redirectErrorCode = searchParams.get('error')

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginForm>({
    resolver: zodResolver(loginFormSchema),
    defaultValues: { email: registeredEmail ?? '', password: '' },
  })

  // A successful login updates the session, and the route guard then leaves this page.
  const onSubmit = handleSubmit((values) => login.mutate(values))

  const errorKey = login.isError
    ? errorMessageKey(login.error)
    : redirectErrorCode
      ? errorCodeMessageKey(redirectErrorCode)
      : null
  const fieldError = (message: string | undefined) =>
    message ? t(message as ValidationMessageKey) : undefined

  return (
    <AuthLayout title={t('auth.login.title')}>
      {errorKey ? (
        <Alert variant="error">{t(errorKey)}</Alert>
      ) : registeredEmail ? (
        <Alert variant="success">{t('auth.register.created')}</Alert>
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
          label={t('auth.fields.password')}
          type="password"
          autoComplete="current-password"
          error={fieldError(errors.password?.message)}
          {...register('password')}
        />
        <Button type="submit" disabled={login.isPending}>
          {t('auth.login.submit')}
        </Button>
      </form>

      <p className="text-center text-sm text-slate-500 dark:text-slate-400">{t('auth.or')}</p>

      {/* A plain link: Google sign-in is a full page navigation through the backend. */}
      <a className={BUTTON_CLASSES.secondary} href={googleSignInUrl(i18n.resolvedLanguage ?? 'en')}>
        {t('auth.google')}
      </a>

      <p className="text-center text-sm">
        {t('auth.login.noAccount')}{' '}
        <Link className="font-medium text-indigo-600 underline dark:text-indigo-400" to="/register">
          {t('auth.login.registerLink')}
        </Link>
      </p>
    </AuthLayout>
  )
}
