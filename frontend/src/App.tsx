import { Route, Routes } from 'react-router'
import AppLayout from './components/AppLayout'
import GuestOnly from './components/GuestOnly'
import { UNBUILT_ITEMS } from './components/layout/navigation'
import RequireAuth from './components/RequireAuth'
import AuthCallbackPage from './pages/AuthCallbackPage'
import ComingSoonPage from './pages/ComingSoonPage'
import FinanceCategoriesPage from './pages/FinanceCategoriesPage'
import FinancesPage from './pages/FinancesPage'
import HabitsPage from './pages/HabitsPage'
import HomePage from './pages/HomePage'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'
import SettingsPage from './pages/SettingsPage'
import WorkLogPage from './pages/WorkLogPage'

export default function App() {
  return (
    <Routes>
      <Route element={<GuestOnly />}>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
      </Route>

      <Route path="/auth/callback" element={<AuthCallbackPage />} />

      <Route element={<RequireAuth />}>
        <Route element={<AppLayout />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/habits" element={<HabitsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/work-log" element={<WorkLogPage />} />
          <Route path="/finances" element={<FinancesPage />} />
          <Route path="/finance-categories" element={<FinanceCategoriesPage />} />
          {UNBUILT_ITEMS.map((item) => (
            <Route key={item.path} path={item.path} element={<ComingSoonPage item={item} />} />
          ))}
        </Route>
      </Route>
    </Routes>
  )
}
