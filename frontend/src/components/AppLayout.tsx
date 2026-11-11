import { Outlet } from 'react-router'
import BrandMark from './BrandMark'
import BottomNav from './layout/BottomNav'
import Sidebar from './layout/Sidebar'

/**
 * Frame around the signed-in screens: a sidebar on wide screens, and a top bar plus a bottom
 * tab bar on phones (SPEC.md, Screens).
 */
export default function AppLayout() {
  return (
    <div className="flex min-h-dvh">
      <Sidebar />

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 items-center border-b border-border px-4 md:hidden">
          <BrandMark withName />
        </header>
        {/* Bottom padding keeps the last content clear of the phone bottom bar. */}
        <div className="flex-1 pb-24 md:pb-0">
          <Outlet />
        </div>
      </div>

      <BottomNav />
    </div>
  )
}
