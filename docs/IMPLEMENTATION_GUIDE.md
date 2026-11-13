# Implementation guide: day modes, work log, finances

This guide is for AI coding agents implementing three additions to PersonalManagement:

- **Day modes** — the user marks today as a normal or a low-energy day; a low-energy day shows fewer, easier habits.
- **Work log** — the user enters the hours worked on a day and the app computes that day's pay.
- **Finances** — income and expense tracking with categories and one monthly budget.

It says *how* and *in what order* to build them. It does not define them: `SPEC.md` does. When this guide and `SPEC.md` disagree, `SPEC.md` wins; tell the owner about the mismatch.

## Before you start

1. Read `CLAUDE.md` fully. Its rules (language, layering, testing, i18n, asking before guessing) apply to every task here.
2. Read the `SPEC.md` sections named in the task you are about to do.
3. Do one task per session or pull request, in the order given. Each task leaves the repository passing all checks.
4. Follow test-driven development: write the listed tests first, watch them fail, then implement.
5. Follow the Git workflow in `CLAUDE.md`: one branch per task, commits on that branch, and a pull request only when the owner asks.

## Decisions the owner already made

Do not reopen these; they are recorded in `SPEC.md`. If a task seems to need a different answer, stop and ask.

| Topic | Decision |
|---|---|
| Low-energy habits | Each habit has a `low_energy` flag (kept on low-energy days) and an optional easier name. There is no separate habit list. New habits are not kept by default. |
| Choosing the mode | Only for today. Past and future days cannot be changed. |
| Mode lock | The mode cannot change while any active habit is checked for today. Unchecking everything unlocks it. Completions of archived habits do not lock it. |
| Streak and rate | A habit hidden on a low-energy day is unscheduled that day: it neither breaks the streak nor counts in the rate. A kept habit that is not done breaks its streak as usual. |
| Pay formula | `hours × hourly wage`. No multipliers, no night-shift rules. |
| Base hours | Used only to label a day as above, equal to, or below the usual hours. |
| Wage changes | Each entry copies the wage and base hours when it is created; later settings changes do not touch it. |
| Work summary | Per calendar month; one entry per day. |
| Finance scope | Income and expenses. Work-log pay is **not** added to income automatically. |
| Categories | Separate expense and income categories, editable by the user, with a suggested starter set. |
| Budget | One monthly expense budget for the whole account; no per-category budgets. |
| Currency | One currency per account from a fixed list; amounts are integers in minor units; no conversion. |
| Today screen | A read-only money card (hours, pay, total spent). No quick entry there. |
| Weekly statistics | Unchanged except that low-energy days are marked. Money is summarized on its own screens. |

## What exists today (verify before relying on it)

- Backend features under `backend/src/`: `auth`, `days`, `habits`, `tasks`, `subjects`, `study_sessions`, `health`. There is no `settings` feature yet, although the `user_settings` table exists.
- `backend/migrations/20261112000000_create_checklist_tables.sql` defines `user_today(p_user_id, p_at)` and `habit_current_streak(p_habit_id, p_weekdays, p_today)`.
- `backend/src/habits/service.rs` rejects a completion with the field codes `future` and `not_scheduled`.
- `backend/src/days/model.rs` defines the `GET /days/{date}` response (`DayView`, `DayHabit`).
- `backend/src/error.rs` holds `AppError`; each conflict has its own variant and stable code.
- Frontend: `src/pages/` (`HomePage` is the Today screen, `HabitsPage`, `ComingSoonPage` for unbuilt screens), `src/components/today/`, `src/components/habits/`, `src/components/layout/navigation.ts` (menu entries), `src/lib/` (API calls and Zod schemas), `src/hooks/`.

## Rules that are easy to get wrong here

- **Never edit a committed migration.** To change `habit_current_streak`, add a new migration that drops the old function and creates the new one, and repeat the `REVOKE EXECUTE ... FROM PUBLIC`. Enable row level security on every new table, as the existing migrations do.
- **No Vietnamese outside `frontend/src/i18n/locales/`.** This is why the suggested finance categories are created by the frontend from locale strings, not seeded by the backend.
- **Money is never a float.** Use `bigint` in SQL, `i64` in Rust, and integers in JSON. Compute pay in SQL (`round(hours * hourly_wage)` on `numeric` rounds half away from zero, which is half up for positive values) or with a decimal type, never with `f64`.
- **Every new error code needs `errors.<code>` in both locale files** in the same change. A field-level code in `details.fields` (`not_today`, `future`, `archived`, `kind_mismatch`) needs a `validation.*` key only when a screen actually shows it to the user; do not add keys for codes the UI prevents or never displays.
- **Every query filters by `user_id`;** another user's resource is a 404.
- **After changing any query,** run `cargo sqlx prepare -- --all-targets` and include `.sqlx/` in the change.
- **Adding a crate** (for example a decimal type for `numeric` columns) is a decision: check `Cargo.toml` first and ask the owner if a new dependency is needed.

## Phase A — Day modes (roadmap step 5)

Read: `SPEC.md` user stories C7–C9, the `habits` and `low_energy_days` tables, the calculation rules "Scheduled day of a habit", "Streak", "Day mode", and the Checklist API table.

### A1. Schema and habit fields (backend)

- Migration: add `habits.low_energy boolean NOT NULL DEFAULT false` and `habits.low_energy_name text` with a check that the name is null when `low_energy` is false; create `low_energy_days (user_id, date)` with the composite primary key and cascade delete.
- `POST /habits` and `PATCH /habits/{id}` accept `low_energy` and `low_energy_name`; `GET /habits` returns them. Setting `low_energy` to false clears the name. Validate the easier name like the normal name (trimmed, same length limits); a blank name is stored as null.
- Tests first (`backend/tests/habits.rs`): create with and without the fields; update turns the flag on with a name; turning the flag off clears the name; an over-long easier name is a 422; another user's habit is a 404.

### A2. Day mode endpoint and day view (backend)

- `PUT /days/{date}/mode` in the `days` feature. Rules, in order: the date must be today in the user's time zone (422, `details.fields.date = "not_today"`); if the requested mode equals the current one, succeed without changes; if any non-archived habit of the user has a completion dated today, fail with 409 `day_mode_locked` (new `AppError` variant); otherwise insert or delete the `low_energy_days` row. Do the lock check and the write in one transaction or one statement.
- `GET /days/{date}` adds `mode` (`"normal"` or `"low_energy"`) and `mode_locked` (true when the date is not today or an active habit is checked for today). On a low-energy day it lists only habits with `low_energy` set, and `name` is the easier name when one is set.
- Tests first (`backend/tests/days.rs`): default mode is normal; set low-energy then read it back; setting the same mode twice is fine; yesterday and tomorrow are rejected with `not_today`; locked after a completion and unlocked after removing it; not locked by a completion of an archived habit; a low-energy day hides non-kept habits and shows easier names; another user's mode is not visible.

### A3. Completion rule and streak (backend)

- `PUT /habits/{id}/completions/{date}` returns `not_scheduled` when the date is a low-energy day and the habit is not kept.
- New migration replacing `habit_current_streak` so that a day counts as scheduled only if its weekday matches **and** it is not a low-energy day of the habit's owner unless the habit is kept. Keep the existing documented behaviour (today unchecked does not break the streak; completions on unscheduled days are ignored; current settings apply to the past). Update every caller.
- Tests first (`backend/tests/habits.rs`): a hidden habit's streak continues across a low-energy day; a kept habit missed on a low-energy day breaks the streak; a kept habit done on a low-energy day extends it; completing a hidden habit on a low-energy day is a 422.

### A4. Habits screen (frontend)

- Habit form: a "keep on low-energy days" switch and, when it is on, an optional "easier version" text field. The habit list shows a small marker on kept habits.
- Extend the Zod schemas in `src/lib/checklistSchemas.ts` and the form schema; add `habits.*` and `validation.*` keys to both locale files.
- Tests: the easier-name field appears only when the switch is on; submitting sends both fields; switching off sends a null name.

### A5. Today screen (frontend)

- A two-option control (Normal / Low energy) near the date navigation. It is enabled only when the response says `mode_locked` is false; when locked it shows a short explanation (not today, or habits already checked).
- Changing it calls `PUT /days/{date}/mode` and invalidates the day query. A 409 `day_mode_locked` (another tab checked a habit) shows the translated error and refetches.
- Checking or unchecking a habit must refresh `mode_locked` (invalidate or update the day query).
- Tests: the control reflects the mode; it is disabled when locked; selecting low energy sends the request.

**Phase A is done when** a user can mark today low-energy, sees only kept habits under their easier names, cannot switch after checking a habit, and streaks behave as the calculation rules say.

## Phase B — Settings and currency (first half of roadmap step 7)

Read: `SPEC.md` cross-cutting decision "Money", the `users` and `user_settings` tables, and the Auth and account API table.

### B1. Backend

- Migration: `users.currency text NOT NULL DEFAULT 'VND'` with a check against the supported list; `user_settings.hourly_wage bigint`, `base_work_hours numeric(4,2)`, `monthly_budget bigint`, all nullable with range checks.
- Check whether a `user_settings` row is created for every user (registration and Google sign-up). If not, make `GET /settings` work for users without a row (create it lazily or at sign-up) and cover that in a test.
- New `settings` feature: `GET /settings` and `PATCH /settings`. PATCH updates only the fields present; an explicit `null` clears `hourly_wage`, `base_work_hours`, or `monthly_budget`; the Pomodoro fields cannot be null. Distinguishing "absent" from "null" needs a double-`Option` style deserializer; keep it in one small, tested helper.
- `GET /me` returns `currency`; `PATCH /me` accepts it and rejects codes outside the list with a 422. If `/me` is not implemented yet, implement what `SPEC.md` describes for it.
- Tests first (`backend/tests/settings.rs`, plus the `/me` tests): defaults; partial update leaves other fields alone; null clears; zero, negative, and out-of-range values are 422; unsupported currency is 422; unauthenticated is 401.

### B2. Frontend

- `src/lib/money.ts`: `formatMoney(amountMinor, currency, language)` and `parseMoneyInput(text, currency)` that converts what the user typed in major units to minor units. Take the number of fraction digits from `Intl.NumberFormat(..., { style: 'currency', currency }).resolvedOptions().maximumFractionDigits` (0 for VND and JPY, 2 for USD). Unit-test both functions for VND and USD, including rejecting more decimals than the currency has.
- Settings screen (`/settings`, replacing the placeholder): sections for currency (with the "existing amounts are not converted" warning), work settings (hourly wage, base hours), and monthly budget (may be cleared). Build the Pomodoro settings section here too if it does not exist yet; leave profile, time zone, and Google linking to their own work unless they already exist.
- A `useSettings` hook over TanStack Query; the currency comes from `useSession`.

**Phase B is done when** the settings can be read and changed from the screen, and amounts are entered and shown in the account's currency.

## Phase C — Work log (second half of roadmap step 7)

Read: `SPEC.md` user stories W1–W5, the `work_entries` table, the calculation rules "Daily pay", "Work comparison", "Monthly work summary", and the Work log API table.

### C1. Backend

- Migration: `work_entries` with `UNIQUE (user_id, date)` and range checks on `hours`, `hourly_wage`, `base_hours`.
- New `work_entries` feature:
  - `PUT /work-entries/{date}` with `{ "hours", "note?" }`. A future date is a 422 (`details.fields.date = "future"`). If no entry exists, copy `hourly_wage` and `base_work_hours` from the settings; if either is null, fail with 409 `work_settings_missing`. If an entry exists, update only `hours` and `note` and keep the copied values. Use one upsert so that two concurrent requests cannot create two rows.
  - `GET /work-entries?month=YYYY-MM` returns `{ entries, summary }`. Each entry has `date`, `hours`, `hourly_wage`, `base_hours`, `pay`, `comparison` (`above`, `equal`, `below`), `note`. The summary has `total_hours`, `total_pay`, `days_worked`. A malformed month is a 422.
  - `DELETE /work-entries/{date}`; 404 when there is no entry.
- Validation: `hours` greater than 0, at most 24, at most two decimals.
- Tests first (`backend/tests/work_entries.rs`): create computes pay; pay rounds half up (for example 1.25 hours × 30,001); comparison for above, equal, below; update keeps the copied wage after the settings change; a new entry after a settings change uses the new wage; missing settings is a 409; future date and bad hours are 422; month filter excludes neighbouring months; the summary totals; delete; another user's entries are invisible and their dates are 404 on delete.

### C2. Frontend

- `/work-log` page: month picker (previous/next), a form for date (default today), hours, and note; the month's list with hours, pay, and a badge for above, equal, or below; a totals row. Editing reuses the form; deleting asks for confirmation.
- When the wage or base hours are not set, show an empty state that links to Settings instead of the form (and handle 409 `work_settings_missing` the same way).
- Add the Money group and its entries to `src/components/layout/navigation.ts` (sidebar and the phone More sheet) with `nav.*` keys in both locales.
- Zod schemas in `src/lib/`, a `useWorkEntries(month)` hook, and tests for the form validation, the badge, and the empty state.

**Phase C is done when** a user who set a wage can enter hours for a day, sees the pay and badge, and sees correct monthly totals that survive a later wage change.

## Phase D — Finances (roadmap step 8)

Read: `SPEC.md` user stories F1–F5, the `finance_categories` and `transactions` tables, the calculation rule "Monthly finance summary", and the Finances API table.

### D1. Categories (backend)

- Migration: `finance_categories` with a unique index on `(user_id, kind, lower(name))` and a check on `kind`; `transactions` with `category_id` referencing categories `ON DELETE RESTRICT`, a positive `amount` check, and an index on `(user_id, date)`. Both tables can go in one migration.
- New `finances` feature (split into sub-modules if it grows): category list (`?kind=`, `?include_archived=true`), create, patch (name, color, archive state; `kind` cannot change), delete.
- Errors: 409 `finance_category_name_taken`, 409 `finance_category_in_use`. Follow how `subjects` handles its duplicate-name error and archiving.
- Tests first (`backend/tests/finance_categories.rs`): create each kind; the same name is allowed across kinds but not within one, ignoring case; an invalid color or kind is a 422; archive hides from the default list; delete an unused category; delete a used one is a 409; another user's category is a 404.

### D2. Transactions and summary (backend)

- `POST /transactions` with `{ "category_id", "amount", "date", "note?" }`. The category must belong to the user (`details.fields.category_id = "not_found"`) and not be archived (`"archived"`). Any date is allowed, past or future.
- `PATCH /transactions/{id}`: a new category must have the same kind as the current one (`"kind_mismatch"`); an archived category is rejected only when the category actually changes.
- `GET /transactions?month=` returns the month's transactions, newest date first then newest created first, each with `kind` and its category's `id`, `name`, and `color`.
- `GET /finances/summary?month=` returns `income_total`, `expense_total`, `balance`, `by_category` (kind, category, total; only categories with transactions in the month, archived ones included), and `budget` (`{ "amount", "spent" }`, or null when no budget is set).
- Tests first (`backend/tests/transactions.rs`, `backend/tests/finances_summary.rs`): create and list; amount of zero or below is a 422; the category rules above; month boundaries; update and delete; the summary totals with mixed kinds; the budget present and absent; an empty month returns zeros; another user's data is invisible and their ids are 404.

### D3. Frontend

- `/finance-categories`: two lists (expense, income) with add, rename, recolor (the same fixed palette as subjects), and archive. While the user has no category at all, show a button that creates the suggested set by posting each one. The names come from locale keys `finances.suggestedCategories.*` in the active language: expenses — food, transport, housing, bills, shopping, health, entertainment, education, other; income — salary, bonus, other. Ignore a 409 for a name that already exists so that a retry is safe.
- `/finances`: month picker; three figures (income, expenses, balance); a budget bar (spent of budget, a distinct state when over budget, hidden when no budget is set, with a link to Settings); expenses per category (a Recharts chart or a simple list with bars); the transaction list; a form with a kind toggle, amount, date (default today), category (only non-archived categories of the chosen kind), and note.
- Hooks `useFinanceCategories`, `useTransactions(month)`, `useFinanceSummary(month)`; every mutation invalidates the affected month's list and summary.
- Tests: the category select follows the kind toggle; amount parsing uses the account currency; the budget bar states; the suggested-set button appears only for an empty list.

**Phase D is done when** a user can set up categories in one click, record income and expenses, and see a month's totals, balance, per-category breakdown, and budget use.

## Phase E — Today money card (after C and D)

- Backend: `GET /days/{date}` adds `work` (`{ "hours", "pay" }` or null) and `expense_total` (0 when nothing was spent). Extend `backend/tests/days.rs`.
- Frontend: one read-only card on the Today screen showing the hours and pay of the day (or a "no hours logged" line), the total spent, and links to `/work-log` and `/finances`. No input on this card.

## Phase F — Weekly statistics (roadmap step 9, when it is built)

`GET /stats/weekly` returns the mode of each day, and the screen marks low-energy days. The checklist rate must use the scheduled-day rule from Phase A, so reuse that logic rather than writing a second version.

## Finishing a task

Run the checks from `CLAUDE.md` and read their output before reporting:

- Backend (in `backend/`, database running): `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
- Query metadata: `cargo sqlx prepare -- --all-targets`
- Frontend (in `frontend/`): `npm run lint && npm test && npm run build`
- Stray Vietnamese: the `git grep` command in the `CLAUDE.md` Commands table must print nothing.

Then report what was built, what was verified and how, and anything in `SPEC.md` that turned out to be unclear or wrong. If a decision changed, update `SPEC.md` in the same change, after asking the owner.
