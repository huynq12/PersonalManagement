import { LogOut } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useLogout, useSession } from '../../hooks/useSession'

/** Who is signed in, with the sign-out button. */
export default function UserMenu() {
  const { t } = useTranslation()
  const session = useSession()
  const logout = useLogout()
  const user = session.data

  return (
    <div className="flex items-center gap-2.5 border-t border-border px-2.5 pt-3">
      <span
        aria-hidden="true"
        className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-sm font-semibold"
      >
        {user?.display_name.charAt(0).toUpperCase()}
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-sm font-medium">{user?.display_name}</span>
        <span className="truncate text-xs text-muted-foreground">{user?.email}</span>
      </span>
      <button
        type="button"
        aria-label={t('auth.logout')}
        title={t('auth.logout')}
        disabled={logout.isPending}
        onClick={() => logout.mutate()}
        className="flex size-9 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground disabled:opacity-60"
      >
        <LogOut className="size-4" aria-hidden="true" />
      </button>
    </div>
  )
}
