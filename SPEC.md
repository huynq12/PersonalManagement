# SPEC — Personal management web app

Status: draft v1. Date: 2026-11-14 (first written 2026-11-11).

## 1. Goals

- A web app that helps a user manage daily life: a checklist (habits and to-dos) with normal and low-energy days, a meal plan by weekday, a Pomodoro study timer, a work log that computes daily pay, personal finances (income, expenses, a monthly budget), and weekly statistics.
- The main purpose of the project is **learning**: a real-world Rust backend (Axum, SQLx, PostgreSQL), hand-built auth (JWT, OAuth2), React + TypeScript, Docker, CI/CD, and deployment on free services.
- Multiple users with self-registration; each user's data is fully isolated.

### Definition of done for v1

- Users can register, sign in (email + password, Google), and use all six feature groups (checklist, meal plan, Pomodoro, work log, finances, statistics) on both desktop and phone (responsive).
- The backend has integration tests for every endpoint and OpenAPI documentation.
- The app is publicly deployed at zero cost.

## 2. Architecture

```
┌──────────────────────────┐
│  Browser (desktop/phone) │
│  React + Vite + TS (SPA) │
└───────┬──────────┬───────┘
        │ static   │ REST/JSON over HTTPS
        │ files    │ (access token + refresh cookie)
        ▼          ▼
┌────────────┐  ┌─────────────────────────────┐      ┌──────────────┐
│ Static CDN │  │ Rust backend (Docker)       │◄────►│ Google OAuth │
│ Cloudflare │  │ Axum: router → middleware   │      └──────────────┘
│ Pages /    │  │ (auth, CORS, tracing)       │
│ Vercel     │  │ → handler → service → repo  │
└────────────┘  │ SQLx (pool, migrations)     │
                │ Render / Koyeb              │
                └──────────────┬──────────────┘
                               │ SQL (TLS)
                               ▼
                        ┌──────────────┐
                        │ PostgreSQL   │
                        │ Supabase     │
                        └──────────────┘

GitHub ── push ──► GitHub Actions: test, build image, deploy
```

### Technology

| Part | Choice |
|---|---|
| Backend | Rust, Axum, Tokio, SQLx (compile-time checked `query!` macros with committed `.sqlx` offline metadata, migrations), `argon2`, `jsonwebtoken`, `reqwest` (the Google OAuth flow is written by hand on top of it), `utoipa` (OpenAPI), `tracing`, `validator` |
| Database | PostgreSQL. The running app (development and production) uses a Supabase-hosted database; tests and CI use a disposable PostgreSQL in Docker. Supabase is used only as a PostgreSQL host, not for Supabase Auth or its client libraries. |
| Frontend | Vite, React, TypeScript, React Router, TanStack Query, Zod, React Hook Form, Tailwind CSS, i18next, a chart library (Recharts) |
| Operations | Docker multi-stage builds, GitHub Actions |
| Hosting (free) | Frontend: Cloudflare Pages or Vercel. Backend: Render or Koyeb. Database: Supabase. Free-tier terms must be re-checked at deployment time. |

### Repository layout

Monorepo: `backend/`, `frontend/`, `.github/workflows/`.

The backend is organized by feature: each feature is a directory under `backend/src/` (`auth/`, `health/`, `habits/`, later `work_entries/`, `finances/`, ...) holding its own router, handlers, service, repository, and models.

### Cross-cutting decisions

- **Dates and time zones:** each user has a `timezone` field (default `Asia/Ho_Chi_Minh`). Every notion of a "day" (checklist, statistics) is the calendar date in that time zone. Instants are stored as `timestamptz` (UTC).
- **Weeks:** start on Monday. `weekday` is numbered 1 (Monday) to 7 (Sunday).
- **Months:** the work log and finances are summarized per calendar month, passed as `YYYY-MM`. A record belongs to the month of its `date`, which is already a date in the user's time zone.
- **Money:** each user has one `currency` (an ISO 4217 code, default `VND`) chosen from a fixed list: `VND`, `USD`, `EUR`, `GBP`, `JPY`, `KRW`, `CNY`, `SGD`, `THB`, `AUD`. Every amount is a `bigint` in the currency's minor unit (dong for VND, cents for USD) and travels as a JSON integer; floating point is never used for money. The frontend formats amounts with `Intl.NumberFormat`. Changing the currency does not convert existing amounts; Settings warns about this.
- **Auth:** a short-lived JWT access token (15 minutes) sent in the `Authorization` header; a long-lived random refresh token (30 days) stored in an `httpOnly`, `Secure` cookie, rotated on every refresh, with only its hash stored in the database.
- **Tokens in the browser:** the access token is kept in memory only, never in `localStorage`. On page load the frontend calls `/auth/refresh` to restore the session from the refresh cookie, and it repeats that call once when the API answers 401 `unauthorized`. Refresh calls never overlap, within a tab or across tabs, because a reused refresh token revokes the session.
- **Access token signing:** HS256 with a symmetric secret read from the `JWT_SECRET` environment variable.
- **Refresh token reuse:** every sign-in starts a token family. Presenting a refresh token that was already rotated is treated as theft: the request gets 401 and every token in that family is revoked. Other sign-ins (other devices) are not affected.
- **Passwords:** 8 to 128 characters with no composition rules (NIST SP 800-63B), hashed with Argon2id.
- **Registration:** creates the account without signing in; the user then signs in with the new credentials.
- **Google sign-in flow:** authorization code flow with PKCE, `state`, and `nonce`. These values travel in a signed, `httpOnly` cookie that lives 10 minutes; nothing is stored in the database. The backend exchanges the code, then verifies the Google ID token (signature against Google's JWKS, issuer, audience, expiry, nonce) and requires a verified email. On success it sets the refresh cookie and redirects to the frontend route `/auth/callback`, which calls `/auth/refresh` to obtain an access token; no token ever appears in a URL. Failures redirect to `/login?error=<code>` (or `/settings?error=<code>` when linking).
- **Linking Google:** if a Google email matches an existing password account, the accounts are **not merged automatically** (emails registered with a password are unverified). The user signs in with the password and links Google from Settings.
- **Authorization:** every query filters by the `user_id` taken from the token; accessing another user's resource returns 404.
- **Errors:** a single JSON shape, `{ "error": { "code", "message", "details?" } }`. `code` is a stable snake_case identifier; `message` is an English, developer-facing description that is never shown to users.
- **Theme:** light, dark, or system (the default, which follows the operating system and its changes). The choice is stored in `localStorage` only and applied before the first render as a `dark` class on `<html>`. Colors come from semantic design tokens (shadcn/ui style, zinc palette) defined once for each theme.
- **Language in the codebase:** comments, logs, error messages, test names, commit messages, and documentation are all English.
- **Internationalization (i18n):** every string shown to users exists in English and Vietnamese. The backend never returns translated text; the frontend translates by error `code` (i18next) and uses a generic message for unknown codes. The language is chosen in this order: the user's saved choice, the browser language, then English. Dates, numbers, and durations are formatted with `Intl` in the active language.

## 3. User stories

### Account
- U1. As a visitor, I register with email and password to get my own account.
- U2. As a visitor, I sign in with Google so I do not have to remember a password.
- U3. As a user, I stay signed in after closing the browser and can sign out when I want.
- U4. As a user, I change my display name and time zone, and link my Google account.
- U5. As a visitor or user, I switch the interface between English and Vietnamese; the choice is remembered in the browser and, once signed in, on my account.
- U6. As a visitor or user, I choose a light, dark, or system theme; the choice is remembered in the browser.

### Checklist
- C1. I create a habit and choose the weekdays it applies to (every day by default).
- C2. Each day I see exactly the habits scheduled for that weekday and can check or uncheck them.
- C3. I add to-dos for a specific day, then complete, edit, or delete them.
- C4. I review a previous day's checklist and plan to-dos for a future day.
- C5. I archive (hide) a habit I no longer follow without losing its history.
- C6. I see the current streak of each habit.
- C7. For each habit I choose whether it stays on low-energy days and, if it does, an easier version of it (for example "Run 5 km" becomes "Walk 5 minutes").
- C8. I mark today as a normal day (the default) or a low-energy day. On a low-energy day I see only the habits I kept, under their easier names. I can switch only today, and only while no habit is checked for today.
- C9. A habit that is hidden on a low-energy day does not break its streak and does not count in that day's completion rate.

### Meal plan
- M1. I manage a dish library: name, note, estimated calories, and a list of ingredients with quantity and unit.
- M2. I build one fixed weekly template: each weekday has breakfast, lunch, dinner, and snack, and each meal holds one or more dishes from the library.
- M3. I see today's menu on the main screen.
- M4. I see a shopping list that merges the ingredients for the whole week (summed per ingredient and unit).

### Pomodoro
- P1. I manage a list of subjects (name, color).
- P2. I pick a subject and start a focus session; when time is up the session is recorded and a break starts.
- P3. I cancel a session midway; cancelled sessions do not count in statistics.
- P4. When I reload the page or reopen the tab, the running timer is restored.
- P5. I adjust the focus length, short break, long break, and the number of sessions before a long break.

### Work log
- W1. I set my hourly wage and my base working hours per day in Settings.
- W2. For a day I worked, I enter the number of hours; the app computes that day's pay and marks whether I worked more than, less than, or exactly my base hours.
- W3. I edit or delete a day's entry.
- W4. I see a month's entries with the total hours, total pay, and number of days worked, and I move between months.
- W5. Changing my wage or base hours does not change entries I already recorded.

### Finances
- F1. I record an expense or an income with an amount, a date, a category, and an optional note; I edit or delete it later.
- F2. I manage separate expense and income categories (name, color), and archive the ones I no longer use. A new account can add a suggested set in one step.
- F3. I see a month's transactions with total income, total expenses, the balance, and expenses per category, and I move between months.
- F4. I set one monthly expense budget and see how much of it the month has used.
- F5. I choose the currency my amounts are shown in.

### Statistics
- S1. I see the week's total study time per day and per subject, with the number of completed Pomodoros.
- S2. I see the checklist completion rate per day (habits and to-dos).
- S3. I see the current and longest streak of each habit.
- S4. I see the change compared with the previous week and can move between weeks.
- S5. I see which days of the week were low-energy days.

## 4. Screens

| # | Screen | Path | Main content |
|---|---|---|---|
| 1 | Sign in / Register | `/login`, `/register` | Email + password form, "Sign in with Google" button |
| 2 | Today | `/` | Date picker (previous/next); the day mode (normal or low-energy), switchable for today only; habits for the day; to-dos with quick add; menu for that weekday; study time for the day and a shortcut to Pomodoro; a read-only money card (hours worked and pay, total spent) with shortcuts to the Work log and Finances |
| 3 | Habits | `/habits` | List, add/edit (name, weekdays, kept on low-energy days, easier name), reorder, archive, streak |
| 4 | Pomodoro | `/pomodoro` | Subject picker, countdown, start/cancel, focus/break state, today's sessions |
| 5 | Subjects | `/subjects` | Subject CRUD (name, color); the color is picked from a fixed palette in the UI, and the API only checks the `#RRGGBB` format |
| 6 | Weekly meal plan | `/meal-plan` | A 7 weekdays × 4 meals grid; choose dishes for each cell |
| 7 | Dishes | `/dishes`, `/dishes/:id` | Dish library, dish form with ingredients |
| 8 | Shopping list | `/shopping-list` | Merged ingredients for the week, with temporary check marks kept in the browser |
| 9 | Weekly statistics | `/stats` | Week picker; study time charts per day and subject; checklist rate; low-energy days marked; streaks; comparison with the previous week |
| 10 | Settings | `/settings` | Profile, time zone, language, currency, Pomodoro settings, work settings (hourly wage, base hours), monthly budget, Google linking, sign out |
| 11 | Work log | `/work-log` | Month picker; add or edit a day's hours and note; list of days with hours, pay, and a more/less/equal badge against the base hours; totals for the month |
| 12 | Finances | `/finances` | Month picker; income, expenses, balance; budget progress bar; expenses per category; transaction list with add, edit, delete |
| 13 | Finance categories | `/finance-categories` | Expense and income category CRUD (name, color from the fixed palette), archive; a button that adds the suggested set while the list is empty |

Layout: a left navigation bar on desktop and a bottom bar on phones. The language and theme switchers are available on every screen, including sign in and register.

- Desktop sidebar: the screens grouped as Overview (Today, Weekly statistics), Checklist (Habits), Study (Pomodoro, Subjects), Meals (Weekly meal plan, Dishes, Shopping list), and Money (Work log, Finances, Finance categories); at the bottom the theme and language switchers, Settings, and the signed-in user with sign out.
- Phone bottom bar: Today, Habits, Pomodoro, Meal plan, and More. More opens a bottom sheet with the remaining screens, Settings, the switchers, and sign out.
- A screen with a menu entry that is not built yet shows a placeholder page.

## 5. Data model

Every table has `id uuid` (primary key) and `created_at timestamptz` unless noted. Every user-owned table has `user_id` → `users.id` with cascade delete.

### Account

**users**
| Column | Type | Notes |
|---|---|---|
| email | citext | unique |
| password_hash | text null | null for Google-only accounts |
| display_name | text | |
| timezone | text | default `Asia/Ho_Chi_Minh` |
| locale | text | `vi` or `en`; taken from the active language at registration |
| currency | text | ISO 4217 code from the supported list; default `VND` |

**oauth_accounts**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| provider | text | `google` |
| provider_user_id | text | unique (provider, provider_user_id) |

**refresh_tokens**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| token_hash | text | unique |
| family_id | uuid | shared by all tokens rotated from one sign-in; index |
| expires_at | timestamptz | |
| revoked_at | timestamptz null | |

**user_settings** (primary key `user_id`)
| Column | Type | Default |
|---|---|---|
| focus_minutes | smallint | 25 |
| short_break_minutes | smallint | 5 |
| long_break_minutes | smallint | 15 |
| long_break_every | smallint | 4 |
| hourly_wage | bigint null | null until set; minor units, greater than 0 |
| base_work_hours | numeric(4,2) null | null until set; greater than 0, at most 24 |
| monthly_budget | bigint null | null means no budget; minor units, greater than 0 |

### Checklist

**habits**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| name | text | |
| weekdays | smallint[] | values 1 to 7; all seven by default |
| low_energy | boolean | whether the habit stays on low-energy days; default false |
| low_energy_name | text null | easier version shown on low-energy days; always null when `low_energy` is false; when null the normal name is shown |
| sort_order | int | |
| archived_at | timestamptz null | |

**habit_completions** (primary key `(habit_id, date)`, no `id`)
| Column | Type | Notes |
|---|---|---|
| habit_id | uuid | cascade |
| date | date | date in the user's time zone |
| completed_at | timestamptz | |

**low_energy_days** (primary key `(user_id, date)`, no `id`)
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| date | date | date in the user's time zone |

A row marks the date as a low-energy day; every other day is a normal day.

**tasks**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| date | date | index (user_id, date) |
| title | text | |
| is_done | boolean | |
| done_at | timestamptz null | |
| sort_order | int | |

### Pomodoro

**subjects**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| name | text | unique (user_id, lower(name)); archived subjects count too |
| color | text | hex code `#RRGGBB` |
| archived_at | timestamptz null | |

**study_sessions** (focus sessions only; breaks are not stored)
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| subject_id | uuid null | set null when the subject is deleted |
| status | text | `running`, `completed`, `cancelled` |
| planned_minutes | smallint | |
| started_at | timestamptz | |
| ended_at | timestamptz null | |
| duration_seconds | int null | computed by the server, capped at `planned_minutes × 60` |
| local_date | date | start date in the user's time zone; index (user_id, local_date) |

Constraint: at most one `running` session per user (partial unique index).

### Meal plan

**ingredients**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| name | text | unique (user_id, lower(name)) |
| default_unit | text | for example g, ml, piece |

**dishes**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| name | text | |
| note | text null | |
| calories | int null | estimate per serving |

**dish_ingredients** (primary key `(dish_id, ingredient_id)`)
| Column | Type | Notes |
|---|---|---|
| dish_id | uuid | cascade |
| ingredient_id | uuid | restrict |
| quantity | numeric | |
| unit | text | |

**meal_plan_entries**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| weekday | smallint | 1 to 7 |
| meal | text | `breakfast`, `lunch`, `dinner`, `snack` |
| dish_id | uuid | cascade |
| sort_order | int | unique (user_id, weekday, meal, dish_id) |

### Work log

**work_entries**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| date | date | unique (user_id, date); one entry per day |
| hours | numeric(4,2) | greater than 0, at most 24 |
| hourly_wage | bigint | copied from the settings when the entry is created |
| base_hours | numeric(4,2) | copied from the settings when the entry is created |
| note | text null | |

### Finances

**finance_categories**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| kind | text | `expense` or `income`; never changes after creation |
| name | text | unique (user_id, kind, lower(name)); archived categories count too |
| color | text | hex code `#RRGGBB` |
| archived_at | timestamptz null | |

**transactions**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| category_id | uuid | restrict; the transaction's kind is its category's kind |
| amount | bigint | minor units, greater than 0 |
| date | date | date in the user's time zone; index (user_id, date) |
| note | text null | |

### Calculation rules

- **Scheduled day of a habit:** a day whose weekday is in the habit's `weekdays` and that is either a normal day or a low-energy day with the habit's `low_energy` set. Only scheduled days can be completed, appear on the Today screen, and count in streaks and rates.
- **Streak:** the number of consecutive *scheduled* days completed, counting back from today (today not yet checked does not break the streak). Unscheduled days are skipped, and completions on them are ignored. The habit's current `weekdays` and `low_energy` values are used, including for the past. Completing the easier version counts as a full completion. Computed in SQL by the `habit_current_streak` function, with "today" taken in the user's time zone.
- **Day mode:** only today's mode can be changed, and only while no active (non-archived) habit of the user has a completion dated today (unchecking everything unlocks it). A completion of an archived habit does not lock the mode, because the habit is not on the Today screen to uncheck. To-dos, Pomodoro, and the meal plan do not depend on the mode.
- **Checklist rate for a day:** (habits checked + to-dos done) / (habits scheduled + total to-dos) for that day.
- **Study time:** the sum of `duration_seconds` over `completed` sessions, grouped by `local_date`.
- **Shopping list:** `quantity` summed per (ingredient, unit) across the whole weekly template; units are not converted.
- **Daily pay:** `hours × hourly_wage` of the entry, rounded half up to a whole minor unit, computed by the server with decimal arithmetic. There are no multipliers. Updating an entry's hours or note keeps its copied `hourly_wage` and `base_hours`.
- **Work comparison:** `above`, `equal`, or `below`, comparing the entry's `hours` with its `base_hours`.
- **Monthly work summary:** total hours, total pay, and the number of entries of the month.
- **Monthly finance summary:** total income, total expenses, balance (income minus expenses), expenses and income per category, and the budget with the amount spent. Pay from the work log is **not** counted as income; the user records income by hand.

## 6. API

Prefix `/api/v1`. Every endpoint requires an access token except the public Auth group and `/health`. Dates use the `YYYY-MM-DD` format.

### Auth and account
| Method | Path | Description |
|---|---|---|
| POST | `/auth/register` | Register with email and password; returns the new user without signing in |
| POST | `/auth/login` | Sign in; returns an access token and sets the refresh cookie |
| POST | `/auth/refresh` | Rotate the refresh token and issue a new access token; reusing a rotated token revokes its family |
| POST | `/auth/logout` | Revoke the current refresh token |
| GET | `/auth/google` | Redirect to Google (with `state` and PKCE) |
| GET | `/auth/google/callback` | Handle the callback; sign in or create an account |
| POST | `/auth/google/link` | Start linking Google to the signed-in account; returns the Google authorization URL for the browser to open |
| GET | `/me` | Current user |
| PATCH | `/me` | Update display name, time zone, language, currency |
| GET | `/settings` | Pomodoro settings, work settings (`hourly_wage`, `base_work_hours`), and `monthly_budget` |
| PATCH | `/settings` | Update any subset of the settings; `null` clears a nullable one |

### Checklist
| Method | Path | Description |
|---|---|---|
| GET | `/days/{date}` | Aggregated data for the Today screen: the day `mode` and whether it can be changed (`mode_locked`), scheduled habits with their state (named by `low_energy_name` on a low-energy day when it is set), to-dos, the menu for that weekday, total study time (`study_seconds`, the sum of the day's completed sessions), the work entry (hours and pay) if any, and total expenses. The study time, work entry, and expenses are included; the menu is added to the response together with the meal plan. |
| PUT | `/days/{date}/mode` | Set the mode with `{ "mode": "normal" }` or `{ "mode": "low_energy" }`. 422 with `details.fields.date` set to `not_today` unless the date is today; 409 `day_mode_locked` when the mode would change and a habit is already checked for today. |
| GET | `/habits` | List (`?include_archived=`), with streaks |
| POST | `/habits` | Create |
| PATCH | `/habits/{id}` | Update name, weekdays, `low_energy`, `low_energy_name`, order, archive state |
| DELETE | `/habits/{id}` | Delete permanently with its history |
| PUT | `/habits/{id}/completions/{date}` | Mark completed (idempotent). Only today or a past day the habit is scheduled for; otherwise 422 with `details.fields.date` set to `future` or `not_scheduled`. |
| DELETE | `/habits/{id}/completions/{date}` | Unmark |
| GET | `/tasks?date=` | To-dos for a day |
| POST | `/tasks` | Create |
| PATCH | `/tasks/{id}` | Update title, state, date, order |
| DELETE | `/tasks/{id}` | Delete |

### Pomodoro
| Method | Path | Description |
|---|---|---|
| GET | `/subjects` | List subjects by name, ignoring case; archived ones are hidden unless `?include_archived=true` |
| POST | `/subjects` | Create |
| PATCH | `/subjects/{id}` | Update, archive |
| DELETE | `/subjects/{id}` | Delete (past sessions are kept without a subject) |
| POST | `/study-sessions` | Start a session; 409 if one is already running |
| GET | `/study-sessions/current` | The running session (to restore the timer); 204 if none |
| POST | `/study-sessions/{id}/complete` | Finish; the server computes the duration |
| POST | `/study-sessions/{id}/cancel` | Cancel |
| GET | `/study-sessions?from=&to=` | Session history |
| DELETE | `/study-sessions/{id}` | Delete a session |

### Meal plan
| Method | Path | Description |
|---|---|---|
| GET | `/ingredients` | List (`?q=` for suggestions) |
| POST | `/ingredients` | Create |
| PATCH | `/ingredients/{id}` | Update |
| DELETE | `/ingredients/{id}` | Delete; 409 if in use |
| GET | `/dishes` | List dishes |
| POST | `/dishes` | Create a dish with its ingredients |
| GET | `/dishes/{id}` | Details |
| PUT | `/dishes/{id}` | Replace the dish and its ingredients |
| DELETE | `/dishes/{id}` | Delete (removed from the weekly template) |
| GET | `/meal-plan` | The whole weekly template |
| PUT | `/meal-plan/{weekday}/{meal}` | Set the dishes for one meal |
| GET | `/meal-plan/shopping-list` | Merged ingredients for the week |

### Work log
| Method | Path | Description |
|---|---|---|
| GET | `/work-entries?month=` | The month's entries (each with `pay` and `comparison`) and the monthly summary |
| PUT | `/work-entries/{date}` | Create or update the day's entry: `{ "hours", "note?" }`, where `hours` is a JSON number with at most two decimals. Creating copies the current wage and base hours; 409 `work_settings_missing` if either is not set. 422 with `details.fields.date` set to `future` for a future date. |
| DELETE | `/work-entries/{date}` | Delete the day's entry |

### Finances
| Method | Path | Description |
|---|---|---|
| GET | `/finance-categories` | List by kind then name (`?kind=`); archived ones are hidden unless `?include_archived=true` |
| POST | `/finance-categories` | Create; 409 `finance_category_name_taken` |
| PATCH | `/finance-categories/{id}` | Update name, color, archive state |
| DELETE | `/finance-categories/{id}` | Delete; 409 `finance_category_in_use` if a transaction uses it |
| GET | `/transactions?month=` | The month's transactions, newest date first, each with its category |
| POST | `/transactions` | Create: `{ "category_id", "amount", "date", "note?" }`. 422 with `details.fields.category_id` set to `not_found` or `archived`. |
| PATCH | `/transactions/{id}` | Update category, amount, date, note. The new category must have the same kind (`details.fields.category_id` set to `kind_mismatch`). |
| DELETE | `/transactions/{id}` | Delete |
| GET | `/finances/summary?month=` | Monthly finance summary (see the calculation rules) |

### Statistics and system
| Method | Path | Description |
|---|---|---|
| GET | `/stats/weekly?week_start=` | Study time per day and subject, Pomodoro count, checklist rate and mode per day, streaks, comparison with the previous week |
| GET | `/health` | Liveness check (including the database connection) |
| GET | `/docs` | Swagger UI generated from OpenAPI |

## 7. Out of scope (v1)

- Real-time timer sync across devices (SSE/WebSocket).
- Email verification, password reset, changing email (these need a mail service).
- Push notifications, reminder emails, background sound when the tab is closed.
- A free-running stopwatch; only Pomodoro is supported.
- Per-week menus or per-day overrides; there is a single weekly template.
- Tracking actual calories or nutrition, converting ingredient units, prices.
- Sharing data between users, an admin area.
- Monthly or yearly statistics, data export.
- Offline PWA, native mobile apps.
- A third language; translating user-entered data (dish names, habits).
- Changing the mode of a past or future day; more than two day modes; a separate habit list for low-energy days.
- Work log: overtime multipliers, night-shift allowances, several jobs or wages, clocking in and out with a timer, tax and insurance, counting pay as income automatically.
- Finances: budgets per category, several currencies or conversion, accounts or wallets, transfers, recurring transactions, receipts, bank import.
- Redis, caching, advanced rate limiting (later phases).

## 8. Suggested roadmap

Status as of 2026-11-14, describing what is merged into `main`. Update the Status column in the same change that completes a step.

| # | Step | Status |
|---|---|---|
| 1 | Project skeleton: monorepo, Axum + SQLx + migrations, Docker Compose for local Postgres, CI running tests. | Done |
| 2 | Auth: email + password first, then Google OAuth (sign-in and linking) on the backend; then the sign-in and registration screens. The app connects to Supabase from this step on. | Done |
| 3 | Checklist (habits, to-dos, Today screen). | Done. The Today screen shows the study card and the money card but not the menu yet; it arrives with step 6. |
| 4 | Pomodoro (subjects, sessions, settings). | Done: subjects, study sessions, and the Pomodoro settings (API and screens), and the study time on the Today screen (`study_seconds` in `GET /days/{date}`). |
| 5 | Day modes (low-energy days): habit fields, day mode, the new streak rule. Done before statistics because it changes how streaks and rates are computed. | Done |
| 6 | Meal plan (ingredients, dishes, weekly template, shopping list). | Not started |
| 7 | Settings endpoint and screen, currency, then the work log. | Partly done: the settings API, currency, Google linking, and the work log are done; the Settings screen lacks profile, time zone, language, and sign out. |
| 8 | Finances (categories, transactions, monthly summary, budget). | Done, including the money card on the Today screen (`work` and `expense_total` in `GET /days/{date}`). |
| 9 | Weekly statistics. | Done: `GET /stats/weekly` and the `/stats` screen. |
| 10 | OpenAPI, deployment. | Not started: a backend Dockerfile and a Vercel SPA config exist; there is no OpenAPI document, and nothing is deployed. |

`docs/IMPLEMENTATION_GUIDE.md` breaks steps 5, 7, 8, and 9 (plus the Today money card) into ordered tasks for AI coding agents.
