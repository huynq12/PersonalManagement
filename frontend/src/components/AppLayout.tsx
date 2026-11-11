import { useTranslation } from 'react-i18next'
import { Outlet } from 'react-router'
import { useLogout, useSession } from '../hooks/useSession'
import Button from './ui/Button'

/** Frame around the signed-in screens: shows who is signed in and the sign-out button. */
export default function AppLayout() {
  const { t } = useTranslation()
  const session = useSession()
  const logout = useLogout()

  return (
    <>
      <div className="border-b border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900">
        <div className="mx-auto flex max-w-2xl flex-wrap items-center justify-end gap-3 px-4 py-2 text-sm">
          <span>{session.data?.display_name}</span>
          <Button variant="secondary" disabled={logout.isPending} onClick={() => logout.mutate()}>
            {t('auth.logout')}
          </Button>
        </div>
      </div>
      <Outlet />
    </>
  )
}
