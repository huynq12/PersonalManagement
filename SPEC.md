# SPEC — Personal management web app

Status: draft v1. Date: 2026-11-11.

## 1. Goals

- A web app that helps a user manage daily life: a checklist (habits and to-dos), a meal plan by weekday, a Pomodoro study timer, and weekly statistics.
- The main purpose of the project is **learning**: a real-world Rust backend (Axum, SQLx, PostgreSQL), hand-built auth (JWT, OAuth2), React + TypeScript, Docker, CI/CD, and deployment on free services.
- Multiple users with self-registration; each user's data is fully isolated.

### Definition of done for v1

- Users can register, sign in (email + password, Google), and use all four feature groups on both desktop and phone (responsive).
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

The backend is organized by feature: each feature is a directory under `backend/src/` (`auth/`, `health/`, later `habits/`, ...) holding its own router, handlers, service, repository, and models.

### Cross-cutting decisions

- **Dates and time zones:** each user has a `timezone` field (default `Asia/Ho_Chi_Minh`). Every notion of a "day" (checklist, statistics) is the calendar date in that time zone. Instants are stored as `timestamptz` (UTC).
- **Weeks:** start on Monday. `weekday` is numbered 1 (Monday) to 7 (Sunday).
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

### Statistics
- S1. I see the week's total study time per day and per subject, with the number of completed Pomodoros.
- S2. I see the checklist completion rate per day (habits and to-dos).
- S3. I see the current and longest streak of each habit.
- S4. I see the change compared with the previous week and can move between weeks.

## 4. Screens

| # | Screen | Path | Main content |
|---|---|---|---|
| 1 | Sign in / Register | `/login`, `/register` | Email + password form, "Sign in with Google" button |
| 2 | Today | `/` | Date picker (previous/next); habits for the day; to-dos with quick add; menu for that weekday; study time for the day and a shortcut to Pomodoro |
| 3 | Habits | `/habits` | List, add/edit (name, weekdays), reorder, archive, streak |
| 4 | Pomodoro | `/pomodoro` | Subject picker, countdown, start/cancel, focus/break state, today's sessions |
| 5 | Subjects | `/subjects` | Subject CRUD (name, color) |
| 6 | Weekly meal plan | `/meal-plan` | A 7 weekdays × 4 meals grid; choose dishes for each cell |
| 7 | Dishes | `/dishes`, `/dishes/:id` | Dish library, dish form with ingredients |
| 8 | Shopping list | `/shopping-list` | Merged ingredients for the week, with temporary check marks kept in the browser |
| 9 | Weekly statistics | `/stats` | Week picker; study time charts per day and subject; checklist rate; streaks; comparison with the previous week |
| 10 | Settings | `/settings` | Profile, time zone, language, Pomodoro settings, Google linking, sign out |

Layout: a left navigation bar on desktop and a bottom bar on phones. The language and theme switchers are available on every screen, including sign in and register.

- Desktop sidebar: the screens grouped as Overview (Today, Weekly statistics), Checklist (Habits), Study (Pomodoro, Subjects), and Meals (Weekly meal plan, Dishes, Shopping list); at the bottom the theme and language switchers, Settings, and the signed-in user with sign out.
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

### Checklist

**habits**
| Column | Type | Notes |
|---|---|---|
| user_id | uuid | |
| name | text | |
| weekdays | smallint[] | values 1 to 7; all seven by default |
| sort_order | int | |
| archived_at | timestamptz null | |

**habit_completions** (primary key `(habit_id, date)`, no `id`)
| Column | Type | Notes |
|---|---|---|
| habit_id | uuid | cascade |
| date | date | date in the user's time zone |
| completed_at | timestamptz | |

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
| name | text | unique (user_id, name) |
| color | text | hex code |
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

### Calculation rules

- **Streak:** the number of consecutive *scheduled* days completed, counting back from today (today not yet checked does not break the streak). Unscheduled days are skipped. The habit's current `weekdays` schedule is used, including for the past.
- **Checklist rate for a day:** (habits checked + to-dos done) / (habits scheduled + total to-dos) for that day.
- **Study time:** the sum of `duration_seconds` over `completed` sessions, grouped by `local_date`.
- **Shopping list:** `quantity` summed per (ingredient, unit) across the whole weekly template; units are not converted.

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
| PATCH | `/me` | Update display name, time zone, language |
| GET | `/settings` | Pomodoro settings |
| PUT | `/settings` | Update Pomodoro settings |

### Checklist
| Method | Path | Description |
|---|---|---|
| GET | `/days/{date}` | Aggregated data for the Today screen: scheduled habits with their state, to-dos, the menu for that weekday, total study time |
| GET | `/habits` | List (`?include_archived=`), with streaks |
| POST | `/habits` | Create |
| PATCH | `/habits/{id}` | Update name, weekdays, order, archive state |
| DELETE | `/habits/{id}` | Delete permanently with its history |
| PUT | `/habits/{id}/completions/{date}` | Mark completed (idempotent) |
| DELETE | `/habits/{id}/completions/{date}` | Unmark |
| GET | `/tasks?date=` | To-dos for a day |
| POST | `/tasks` | Create |
| PATCH | `/tasks/{id}` | Update title, state, date, order |
| DELETE | `/tasks/{id}` | Delete |

### Pomodoro
| Method | Path | Description |
|---|---|---|
| GET | `/subjects` | List subjects |
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

### Statistics and system
| Method | Path | Description |
|---|---|---|
| GET | `/stats/weekly?week_start=` | Study time per day and subject, Pomodoro count, checklist rate per day, streaks, comparison with the previous week |
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
- Redis, caching, advanced rate limiting (later phases).

## 8. Suggested roadmap

1. Project skeleton: monorepo, Axum + SQLx + migrations, Docker Compose for local Postgres, CI running tests.
2. Auth: email + password first, then Google OAuth (sign-in and linking) on the backend; then the sign-in and registration screens. The app connects to Supabase from this step on.
3. Checklist (habits, to-dos, Today screen).
4. Pomodoro (subjects, sessions, settings).
5. Meal plan (ingredients, dishes, weekly template, shopping list).
6. Weekly statistics.
7. OpenAPI, deployment.
