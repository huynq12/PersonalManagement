import { useTranslation } from 'react-i18next'
import { NavLink } from 'react-router'
import type { NavItem } from './navigation'

interface NavItemLinkProps {
  item: NavItem
  onNavigate?: () => void
}

/** One menu entry with its icon; highlighted while its screen is open. */
export default function NavItemLink({ item, onNavigate }: NavItemLinkProps) {
  const { t } = useTranslation()
  const Icon = item.icon

  return (
    <NavLink
      to={item.path}
      // Only the exact path counts for "/", otherwise Today would stay highlighted everywhere.
      end={item.path === '/'}
      onClick={onNavigate}
      className={({ isActive }) =>
        `flex min-h-9 items-center gap-2.5 rounded-md px-2.5 text-sm transition-colors ${
          isActive
            ? 'bg-accent font-medium text-accent-foreground'
            : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
        }`
      }
    >
      <Icon className="size-4 shrink-0" aria-hidden="true" />
      {t(item.labelKey)}
    </NavLink>
  )
}
