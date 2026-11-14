import {
  BookOpen,
  Briefcase,
  CalendarCheck,
  ChartColumn,
  Landmark,
  ListChecks,
  Settings,
  ShoppingCart,
  Soup,
  Tags,
  Timer,
  Utensils,
  type LucideIcon,
} from 'lucide-react'

type NavLabelKey =
  | 'nav.today'
  | 'nav.stats'
  | 'nav.habits'
  | 'nav.pomodoro'
  | 'nav.subjects'
  | 'nav.mealPlan'
  | 'nav.mealPlanShort'
  | 'nav.dishes'
  | 'nav.shoppingList'
  | 'nav.workLog'
  | 'nav.finances'
  | 'nav.financeCategories'
  | 'nav.settings'

type NavGroupLabelKey =
  | 'nav.groups.overview'
  | 'nav.groups.checklist'
  | 'nav.groups.study'
  | 'nav.groups.meals'
  | 'nav.groups.money'

export interface NavItem {
  /** Route path; matches the screen list in SPEC.md. */
  path: string
  labelKey: NavLabelKey
  /** A shorter label for the phone bottom bar, where space is tight. */
  shortLabelKey?: NavLabelKey
  icon: LucideIcon
}

export interface NavGroup {
  labelKey: NavGroupLabelKey
  items: NavItem[]
}

const TODAY: NavItem = { path: '/', labelKey: 'nav.today', icon: CalendarCheck }
const STATS: NavItem = { path: '/stats', labelKey: 'nav.stats', icon: ChartColumn }
const HABITS: NavItem = { path: '/habits', labelKey: 'nav.habits', icon: ListChecks }
const POMODORO: NavItem = { path: '/pomodoro', labelKey: 'nav.pomodoro', icon: Timer }
const SUBJECTS: NavItem = { path: '/subjects', labelKey: 'nav.subjects', icon: BookOpen }
const MEAL_PLAN: NavItem = {
  path: '/meal-plan',
  labelKey: 'nav.mealPlan',
  shortLabelKey: 'nav.mealPlanShort',
  icon: Utensils,
}
const DISHES: NavItem = { path: '/dishes', labelKey: 'nav.dishes', icon: Soup }
const SHOPPING_LIST: NavItem = {
  path: '/shopping-list',
  labelKey: 'nav.shoppingList',
  icon: ShoppingCart,
}

const WORK_LOG: NavItem = { path: '/work-log', labelKey: 'nav.workLog', icon: Briefcase }
const FINANCES: NavItem = { path: '/finances', labelKey: 'nav.finances', icon: Landmark }
const FINANCE_CATEGORIES: NavItem = {
  path: '/finance-categories',
  labelKey: 'nav.financeCategories',
  icon: Tags,
}

export const SETTINGS_ITEM: NavItem = {
  path: '/settings',
  labelKey: 'nav.settings',
  icon: Settings,
}

/** The sidebar on wide screens, grouped by feature area. */
export const NAV_GROUPS: NavGroup[] = [
  { labelKey: 'nav.groups.overview', items: [TODAY, STATS] },
  { labelKey: 'nav.groups.checklist', items: [HABITS] },
  { labelKey: 'nav.groups.study', items: [POMODORO, SUBJECTS] },
  { labelKey: 'nav.groups.meals', items: [MEAL_PLAN, DISHES, SHOPPING_LIST] },
  { labelKey: 'nav.groups.money', items: [WORK_LOG, FINANCES, FINANCE_CATEGORIES] },
]

/** The phone bottom bar: the screens used every day. Everything else is under "More". */
export const BOTTOM_BAR_ITEMS: NavItem[] = [TODAY, HABITS, POMODORO, MEAL_PLAN]

/** The "More" sheet on phones: the sidebar groups minus what the bottom bar already shows. */
export const MORE_SHEET_GROUPS: NavGroup[] = NAV_GROUPS.map((group) => ({
  ...group,
  items: group.items.filter((item) => !BOTTOM_BAR_ITEMS.includes(item)),
})).filter((group) => group.items.length > 0)

const BUILT_ITEMS: NavItem[] = [TODAY, STATS, HABITS, WORK_LOG, FINANCES, FINANCE_CATEGORIES, SETTINGS_ITEM]

/** Screens that have a menu entry but are not built yet; they show a placeholder page. */
export const UNBUILT_ITEMS: NavItem[] = [
  ...NAV_GROUPS.flatMap((group) => group.items),
  SETTINGS_ITEM,
].filter((item) => !BUILT_ITEMS.includes(item))
