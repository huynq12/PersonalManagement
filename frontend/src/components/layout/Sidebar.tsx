import { useTranslation } from 'react-i18next'
import BrandMark from '../BrandMark'
import LanguageSwitcher from '../LanguageSwitcher'
import ThemeSwitcher from '../ThemeSwitcher'
import NavItemLink from './NavItemLink'
import { NAV_GROUPS, SETTINGS_ITEM } from './navigation'
import UserMenu from './UserMenu'

/** The navigation column on wide screens. Phones use the bottom bar instead. */
export default function Sidebar() {
  const { t } = useTranslation()

  return (
    <aside className="sticky top-0 hidden h-dvh w-64 shrink-0 flex-col gap-1 overflow-y-auto border-r border-border bg-sidebar px-3 py-4 md:flex">
      <div className="px-2.5 pb-2">
        <BrandMark withName />
      </div>

      <nav aria-label={t('nav.label')} className="flex flex-col gap-0.5">
        {NAV_GROUPS.map((group) => (
          <div key={group.labelKey} className="flex flex-col gap-0.5">
            <p className="px-2.5 pt-4 pb-1.5 text-xs font-medium text-muted-foreground">
              {t(group.labelKey)}
            </p>
            {group.items.map((item) => (
              <NavItemLink key={item.path} item={item} />
            ))}
          </div>
        ))}
      </nav>

      <div className="flex-1" />

      <div className="flex flex-col gap-3 px-2.5 py-3">
        <ThemeSwitcher />
        <LanguageSwitcher />
      </div>
      <NavItemLink item={SETTINGS_ITEM} />
      <UserMenu />
    </aside>
  )
}
