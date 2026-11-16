# Implementation guide: transactional email, email verification, evening reminders

This guide is for AI coding agents adding email to PersonalManagement:

- **Mail module** — the backend sends email through the Resend HTTP API.
- **Email verification** — a new password account receives a confirmation link; until it is confirmed the app works normally and shows a banner.
- **Evening reminder** — once a day, at an hour the user chooses, the app emails the habits and to-dos still open for today. Off by default.

It says *how* and *in what order* to build them. Once a phase is merged, `SPEC.md` is the source of truth for it; each phase lists the `SPEC.md` changes it must make. When this guide and `SPEC.md` disagree about something already merged, `SPEC.md` wins; tell the owner about the mismatch.

## Before you start

1. Read `CLAUDE.md` fully. Its rules (language, layering, testing, i18n, Git workflow, asking before guessing) apply to every phase.
2. Read the `SPEC.md` sections and the source files named in the phase you are about to do. Verify that what this guide says about existing code is still true.
3. Do one phase per session and per branch, in the order given. Each phase leaves the repository passing all checks.
4. Follow test-driven development: write the listed tests first, watch them fail, then implement.
5. The documentation changes listed in each phase (`SPEC.md`, `CLAUDE.md`, `docs/DEPLOYMENT.md`) were approved by the owner together with this guide. Anything beyond them needs a question first.
6. Facts about third-party services in this guide (Resend limits, API details, GitHub Actions behaviour) were written on 2026-11-17 from memory. **Check them against the current official documentation before relying on them**, and tell the owner if one changed.

## Decisions the owner already made

Do not reopen these. If a phase seems to need a different answer, stop and ask.

| Topic | Decision |
|---|---|
| Provider | **Resend**, free plan, through its HTTP API. The owner has or will buy a domain and verify it in Resend. |
| Why not SMTP | Render's free web services block outbound SMTP ports, so mail goes out over HTTPS with the `reqwest` client the backend already has. No new crate. |
| Unverified accounts | Not blocked. They sign in and use everything; a banner asks them to confirm and offers to resend the email. |
| What verification unlocks | Only a confirmed address can turn on reminder emails. Nothing else changes. |
| Google accounts | An account created by Google sign-in is verified from the start (Google already requires a verified email). Automatic merging of a Google identity into a password account stays **off**, as today. |
| Reminder content | One email in the evening listing today's unfinished scheduled habits and to-dos. No email when nothing is open. |
| Reminder default | Off. The user turns it on in Settings and picks the hour (default 20, in their time zone). |
| Scheduling | A GitHub Actions scheduled workflow calls a protected backend endpoint every hour. The backend has no in-process scheduler, because the free Render service sleeps. |
| Email language | English and Vietnamese, chosen by `users.locale`. The strings live in `backend/src/mail/locales/{en,vi}.json`, a second allowed location for Vietnamese text. |
| Out of scope | Password reset, changing the email address, push notifications, a morning digest, per-habit reminders, tracking opens or clicks, Resend webhooks (bounces, complaints). The token table is shaped so password reset can be added later. |

## What exists today (verify before relying on it)

As of 2026-11-17 on `main`.

- `backend/src/auth/`: `service::register` creates the user in a transaction and returns it without signing in. `token.rs` generates opaque refresh tokens (32 random bytes, URL-safe Base64) and stores only their SHA-256. `google.rs` signs a short-lived flow token as a JWT with `JWT_SECRET` and its own audience.
- `users` has no column about email verification. `UserResponse` (`auth/model.rs`) is the body of `/me` and the `user` field of the session response.
- `backend/src/config.rs`: `Config` with an optional `GoogleConfig`; the `Secret` wrapper redacts values in `Debug`. `Config::cors_origin` is the frontend origin and already serves as the base URL for redirects.
- `backend/src/lib.rs`: `AppState { db, config, google }`; `AppState::new` builds the optional Google client.
- `backend/tests/google_auth.rs` fakes Google with a local Axum server on `127.0.0.1:0` and points the config URLs at it. Use the same technique to fake Resend.
- `backend/tests/common/mod.rs`: `config()`, `app()`, `app_with()`, `TestRequest`, `sign_up()`.
- `backend/src/days/repository.rs`: `list_day_habits` implements the "scheduled day of a habit" rule for the Today screen; `user_today(user_id, now)` wraps the SQL function of the same name.
- `backend/src/settings/`: `GET`/`PATCH /settings` over `user_settings`.
- Frontend: `src/pages/RegisterPage.tsx` navigates to `/login` with `registeredEmail` in the location state; `src/hooks/useSession.ts` and `src/lib/auth.ts` / `authSchemas.ts` hold the session and its Zod schemas; `src/components/settings/` has one component per Settings section; `src/components/layout/` holds the signed-in layout.
- `docs/DEPLOYMENT.md` documents Render, Vercel, Supabase, and every environment variable.

## Rules that are easy to get wrong here

- **Never log a token, a link that contains one, an API key, or an email body.** Log the user ID, the kind of email, and the outcome.
- **Store only the hash of a verification token**, exactly like refresh tokens.
- **Escape everything user-entered that goes into HTML email** (display name, habit names, to-do titles). Write one `escape_html` helper and unit-test it.
- **A failed email must not fail registration.** Registration succeeds; the failure is logged once at `warn`.
- **Vietnamese text is allowed only in `frontend/src/i18n/locales/` and `backend/src/mail/locales/`.** Tests that need a Vietnamese string read it from the locale file.
- **Never edit a committed migration**, and enable row level security on every new table, as the existing migrations do.
- **Every new error code needs `errors.<code>` in both frontend locale files** in the same change.
- **After changing any query**, run `cargo sqlx prepare -- --all-targets` and commit `.sqlx/`.
- **No test may call the real Resend API.** The default test configuration has no mail settings.
- **The free plan allows about 100 emails per day and 3,000 per month** (verify). Verification emails and reminders share it. Never retry in a tight loop.

## Phase 0 — Owner setup (manual, not a coding task)

The owner does this in dashboards; an agent only reminds them. Phases A to E can be built and tested without it. Phase F documents it in `docs/DEPLOYMENT.md`.

1. Create a Resend account and add the domain (a subdomain such as `mail.<domain>` keeps the main domain's reputation separate). Add the DNS records Resend shows (SPF and DKIM; add a DMARC record too) at the DNS provider and wait until the domain is verified.
2. Create an API key with sending access only.
3. Set on Render: `RESEND_API_KEY`, `MAIL_FROM`, `CRON_SECRET` (see the variable table in Phase A and Phase D).
4. Set in the GitHub repository: the secret `CRON_SECRET` (same value) and the variable `REMINDERS_URL`.

For local development without a domain, Resend lets an account send from `onboarding@resend.dev` to the account owner's own address only (verify). That is enough to try the flow by hand.

---

## Phase A — Mail module

Branch: `feat/mail-module`. No behaviour visible to users yet.

Read: `backend/src/config.rs`, `backend/src/lib.rs`, `backend/src/auth/google.rs` (how an HTTP client with a configurable base URL is built and tested), `backend/tests/google_auth.rs`.

### Configuration

Add to `Config` an optional `mail: Option<MailConfig>`, built like `GoogleConfig`:

| Field | Source | Notes |
|---|---|---|
| `api_key: Secret` | `RESEND_API_KEY` | Presence of this variable enables mail |
| `from: String` | `MAIL_FROM` | Required when the key is set. A full mailbox, for example `PersonalManagement <no-reply@mail.example.com>` |
| `api_url: String` | constant `https://api.resend.com` | A field, not an environment variable, so tests can point it at a fake server |

Add both variables, commented, to `backend/.env.example`. `tests/common/mod.rs::config()` sets `mail: None`.

### Module layout

`backend/src/mail/` is infrastructure, not an HTTP feature, so it has no router:

- `mod.rs` — `Mailer`, `Email`, `MailError`.
- `locale.rs` — loading the two locale files and looking up strings.
- `template.rs` — `escape_html` and the shared HTML/text layout. The functions that build a specific email (verification, reminder) are added by later phases.
- `locales/en.json`, `locales/vi.json`.

### `Mailer`

```rust
pub struct Email {
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
    /// Sent as the `Idempotency-Key` header when set.
    pub idempotency_key: Option<String>,
    /// Extra message headers, for example `List-Unsubscribe`.
    pub headers: Vec<(String, String)>,
}

impl Mailer {
    pub fn new(config: Option<MailConfig>) -> anyhow::Result<Self>;
    pub fn is_enabled(&self) -> bool;
    pub async fn send(&self, email: &Email) -> Result<(), MailError>;
}
```

- `send` issues `POST {api_url}/emails` with `Authorization: Bearer <key>` and a JSON body `{ from, to: [to], subject, html, text, headers }`. Verify the request shape and the `Idempotency-Key` header in Resend's API reference.
- Give the `reqwest::Client` a 10 second timeout.
- `MailError` (with `thiserror`) has three variants: `Disabled` (no mail configuration), `Rejected` (a 4xx other than 429: retrying will not help), `Unavailable` (network error, timeout, 429, 5xx: a later attempt may work). Callers decide what each means; `send` itself does not retry.
- `send` logs nothing on success beyond a `debug` line; callers log failures once, with structured fields and without the recipient's address in production logs beyond the user ID.
- Add `mailer: Arc<Mailer>` to `AppState`, built in `AppState::new`. It always exists; with `mail: None` every `send` returns `MailError::Disabled`.

### Locale files

- Flat JSON objects with the same keys in both files. Placeholders are written `{name}` and filled by a small `render(template, &[("name", value)])` helper that does plain replacement.
- Embed the files with `include_str!` and parse them once (for example in a `std::sync::LazyLock`). A missing key is a bug: `expect` with a clear message is acceptable there, and a unit test must make it impossible to ship (see tests).
- `MailLocale::from_user_locale(&str)` maps `"vi"` to Vietnamese and anything else to English.
- Start with the strings every email shares (app name, footer). Later phases add their own keys.

### Layout

One function builds the HTML shell (a single centered column, inline styles, no images, no external CSS, a `lang` attribute) and one builds the plain-text shell. Every email is sent with both parts.

### Documentation in this change

- `CLAUDE.md`: in **Language rules**, extend the exception to `backend/src/mail/locales/`; in the Commands table, add `":!backend/src/mail/locales"` to the "Find Vietnamese text" command; in **Layout**, mention `src/mail/`; in **Internationalization**, add that email text is localized by the backend from `backend/src/mail/locales/{en,vi}.json` and that both files must have the same keys.
- `SPEC.md`: Technology table (add Resend under a new "Email" row); a new cross-cutting decision "Email" (HTTP API, localized by `users.locale`, optional configuration: without it no email is sent and the app still runs); the sentence on i18n that says the backend never returns translated text stays true for API responses, so add that emails are the one place the backend localizes.

### Tests first

Unit tests:

- `escape_html` escapes `& < > " '`.
- `render` replaces every placeholder and leaves unknown text alone.
- Both locale files have exactly the same key set (compare the two parsed maps).
- `MailLocale::from_user_locale`.

Integration tests (`backend/tests/mail.rs`) against a fake Resend server on `127.0.0.1:0` that records requests:

- A send posts to `/emails` with the bearer key, the configured `from`, the recipient, both bodies, and the idempotency header when one is given.
- 401 or 422 from the server gives `Rejected`; 429 and 500 give `Unavailable`; a closed port gives `Unavailable`.
- With no mail configuration, `send` returns `Disabled` and makes no request.

Put the fake server in `tests/common/` so later phases reuse it; it should expose the list of captured messages.

### Done when

Backend checks pass, the stray-Vietnamese command (with its new exclusion) prints nothing, and no existing test changed behaviour.

---

## Phase B — Email verification: backend

Branch: `feat/email-verification-api`.

Read: `SPEC.md` "Cross-cutting decisions" (Registration, Google sign-in flow, Linking Google), the Account tables, the Auth API table; `backend/src/auth/` entirely.

### Schema (one new migration)

- `users.email_verified_at timestamptz NULL`.
- Backfill in the same migration: `UPDATE users SET email_verified_at = created_at WHERE password_hash IS NULL` (accounts created by Google sign-in). Password accounts stay unverified, including those that linked Google later, because the linked Google email may differ from `users.email`.
- New table `email_tokens`:

| Column | Type | Notes |
|---|---|---|
| id | uuid | primary key |
| user_id | uuid | → `users.id`, cascade; index |
| purpose | text | `CHECK (purpose IN ('verify_email'))`; a later migration widens this for password reset |
| token_hash | text | unique; SHA-256 of the token |
| email | citext | the address the token was sent to |
| expires_at | timestamptz | |
| used_at | timestamptz null | |
| created_at | timestamptz | default `now()` |

Enable row level security on it.

### Rules

- **Token:** 32 random bytes, URL-safe Base64, stored hashed. Reuse the helpers in `auth/token.rs`; if their names (`generate_refresh_token`, `hash_refresh_token`) no longer fit, rename them to something neutral in a separate `refactor` commit first.
- **Lifetime:** 24 hours.
- **Link:** `{cors_origin}/verify-email?token=<token>`. It points at the frontend, which posts the token to the API. This single-use token is the only token that ever appears in a URL; say so in `SPEC.md`, next to the sentence that no session token does.
- **Issuing a token** (one service function used by registration and by resend): delete the user's `verify_email` rows whose `expires_at` has passed, insert the new row, build the email in the user's locale, send it. Older unexpired tokens stay valid, so an earlier email still works.
- **Registration:** after the transaction commits, issue a token and await the send. On any `MailError` log once at `warn` (`user_id`, error kind) and still return 201. With mail disabled, do not insert a token at all.
- **Google:** `create_google_user` sets `email_verified_at = now()`. In `google_link`, when the Google profile's email equals the account's email ignoring case, set `email_verified_at` if it is null (Google has verified that address).
- **Verifying:** look the token up by hash with `purpose = 'verify_email'`.
  - Unknown or expired → `invalid_verification_token`.
  - Found and valid: in one transaction set `used_at` if it is null and set `users.email_verified_at = now()` where it is null and `users.email = email_tokens.email`. Return success whether or not anything changed, so repeating the request (a double click, React StrictMode running an effect twice) is harmless.
  - The token's `email` no longer matching the user's is treated as unknown (this cannot happen until changing email exists; the check costs nothing).
- **Resending** (signed-in user):
  - Already verified → `email_already_verified`.
  - A `verify_email` row for this user created less than 60 seconds ago, or 5 or more created in the last 24 hours → `verification_email_throttled`.
  - `MailError::Disabled`, `Rejected`, or `Unavailable` → `mail_unavailable`, and delete the row just inserted so a failed attempt does not count against the limit.

### API

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/auth/verify-email` | public | Body `{ "token" }`. 204 on success (idempotent). 400 `invalid_verification_token`. |
| POST | `/auth/resend-verification` | access token | 204 when the email was handed to the provider. 409 `email_already_verified`, 429 `verification_email_throttled`, 503 `mail_unavailable`. |

`UserResponse` gains `email_verified: bool` (true when `email_verified_at` is set). It therefore appears in `/me`, login, refresh, and register responses.

New `AppError` variants and codes: `InvalidVerificationToken` (400), `EmailAlreadyVerified` (409), `VerificationEmailThrottled` (429), `MailUnavailable` (503). Add `errors.<code>` for each to both frontend locale files in this change, even though the screens arrive in Phase C.

### Email

Keys in the mail locale files: subject, greeting with `{name}`, one sentence explaining why the email was sent, the button label, a line saying the link expires in 24 hours, and a line saying the email can be ignored if the recipient did not register. The text part contains the bare link.

### Tests first (`backend/tests/email_verification.rs`, plus changes to `auth.rs`, `me.rs`, `google_auth.rs`)

- Registering with mail configured sends exactly one email to the new address, in the registered locale (compare the subject with the string read from the locale file), containing a `/verify-email?token=` link on the frontend origin.
- The database stores a hash, not the token from the link.
- Registering when the fake server answers 500 still returns 201, and the user exists.
- Registering with mail disabled returns 201 and creates no token row.
- A new password account has `email_verified: false` in `/me`; after posting the token it is `true`.
- Posting the same token twice returns 204 both times.
- An unknown token, a malformed body, and an expired token (set `expires_at` in the past with SQL) fail with the right status and code.
- Resend: success sends a second email and both links work; an immediate second call is throttled; the sixth within 24 hours is throttled (insert rows with SQL rather than waiting); an already verified account gets 409; no access token gets 401; a 500 from the provider gives 503 and leaves no extra row.
- Another user's token verifies only its own account: user B posting user A's token leaves B unverified.
- A user created by Google sign-in has `email_verified: true`.
- Linking a Google account with the same email marks the account verified; linking one with a different email does not.

### `SPEC.md` in this change

- Cross-cutting: add "Email verification" (soft: nothing is blocked; the 24 hour single-use link; what it unlocks); reword "Linking Google" so its reason reads "may be unverified" and note the same-email linking rule.
- User story U7: "As a user, I confirm my email address from a link sent to me, and I can ask for the link again."
- Data model: `users.email_verified_at`, the `email_tokens` table.
- API: the two endpoints; mention `email_verified` on the user.
- Out of scope: remove "Email verification" from the line; keep password reset and changing email, without the "need a mail service" reason.
- Roadmap: add step 11 "Email: mail module, email verification" with its status.

### Done when

Backend checks pass, `.sqlx/` is refreshed and committed, and the frontend checks still pass (the user schema there ignores the unknown field until Phase C; confirm this rather than assume it).

---

## Phase C — Email verification: frontend

Branch: `feat/email-verification-ui`.

Read: `frontend/src/lib/auth.ts`, `authSchemas.ts` and wherever the user schema is defined, `src/hooks/useSession.ts`, `src/pages/RegisterPage.tsx`, `LoginPage.tsx`, `AuthCallbackPage.tsx`, the router setup, `src/components/layout/`, `src/components/settings/ProfileSection.tsx`, `src/components/ui/`.

### Build

- **User schema:** add `email_verified: z.boolean()`.
- **API functions and hooks:** `verifyEmail(token)` and `resendVerification()` through `apiRequest`, with `useMutation` hooks. After a successful verification, invalidate or refresh whatever query holds the signed-in user so the banner disappears.
- **`/verify-email` page** (public route, reachable signed in or signed out, using `AuthLayout`):
  - Read `token` from the query string, then replace the URL without it so the token does not stay in the address bar or the history.
  - Post it once. Show a pending state, then success or the translated error.
  - On success offer a link to `/` when signed in and to `/login` when not.
  - No token in the URL shows the invalid-link message without calling the API.
- **Banner** in the signed-in layout, shown when `email_verified` is false: a short message with the address, and a "Resend email" button. After a successful resend, show a confirmation and disable the button for 60 seconds. Show the translated error for `verification_email_throttled` and `mail_unavailable`. Build it from `Alert` and `Button`; do not add a dismiss control.
- **After registering:** the login page already receives `registeredEmail`; extend its notice to say a confirmation email was sent.
- **Settings → Profile:** show the email with a "Confirmed" or "Not confirmed" label.
- **Locale keys** in both files for all of the above.

### Tests first

- `VerifyEmailPage`: success state; error state for `invalid_verification_token`; no request when the token is missing; the request is sent once.
- Banner: rendered for an unverified user and absent for a verified one; clicking resend calls the API and shows the confirmation; a throttled response shows its message.
- Login page shows the confirmation notice after registration.
- Existing tests whose user fixtures lack `email_verified` are updated.

### Done when

Frontend checks pass. Verify in the browser against a local backend: register, see the banner, and (with Resend's test sender or a fake) follow a link to the success page.

---

## Phase D — Evening reminder: backend

Branch: `feat/reminder-emails-api`.

Read: `SPEC.md` calculation rules ("Scheduled day of a habit", "Day mode"), the `user_settings` table, the Settings API rows; `backend/src/days/` and `backend/src/settings/` entirely; the `user_today` SQL function.

### Schema (one new migration)

Add to `user_settings`:

| Column | Type | Default | Notes |
|---|---|---|---|
| reminder_enabled | boolean | false | |
| reminder_hour | smallint | 20 | `CHECK (reminder_hour BETWEEN 0 AND 23)`; hour of the day in the user's time zone |
| reminder_last_run_on | date null | null | the user's local date of the last run that handled this user |

### Settings API

- `GET /settings` returns `reminder_enabled` and `reminder_hour`.
- `PATCH /settings` accepts both (neither can be `null`; `reminder_hour` is 0 to 23).
- Setting `reminder_enabled` to `true` while the email is unverified → 409 `email_not_verified` (new `AppError::EmailNotVerified`). Turning it off is always allowed. Changing the hour is allowed in either state.
- `reminder_last_run_on` never leaves the backend.

### The job endpoint

New feature directory `backend/src/reminders/` (`mod.rs`, `handler.rs`, `service.rs`, `repository.rs`, `model.rs`).

`POST /api/v1/internal/reminders/run`

- **Authentication:** `Authorization: Bearer <CRON_SECRET>`, compared in constant time. `CRON_SECRET` is a new optional `Config` field (`Secret`, at least 32 bytes when set, validated at startup like `JWT_SECRET`). When it is not configured the route answers 404; a missing or wrong value answers 401 `unauthorized`. It is not a user endpoint: no `AuthUser`.
- **Response:** 200 with `{ "due", "sent", "nothing_pending", "failed" }` (counts).
- With mail disabled, answer 503 `mail_unavailable` without touching any row.

**Selecting due users** (one query, at most 50 per run, oldest `user_id` order is fine):

- `reminder_enabled`, and `users.email_verified_at IS NOT NULL`;
- the local hour, `extract(hour FROM now() AT TIME ZONE users.timezone)`, is **greater than or equal to** `reminder_hour`;
- `reminder_last_run_on IS DISTINCT FROM` the user's local date today.

"Greater than or equal" is deliberate: a run that is late or missed (GitHub delays scheduled workflows, and the backend may be waking up) is caught by the next one, any time before the user's midnight.

**For each due user, in sequence:**

1. Claim the day: `UPDATE user_settings SET reminder_last_run_on = <today> WHERE user_id = $1 AND reminder_last_run_on IS DISTINCT FROM <today>`. If no row changed, another run already took this user; skip.
2. Load today's open items: scheduled habits without a completion today, and to-dos dated today that are not done.
3. Nothing open → count as `nothing_pending`; the claim stays, so the user is evaluated once a day.
4. Otherwise send the email with the idempotency key `reminder/<user_id>/<date>`.
   - Success → `sent`.
   - `Unavailable` → undo the claim (set `reminder_last_run_on` back to its previous value), count as `failed`, and **stop the run**: the provider is down or rate limiting, and the next hourly run retries.
   - `Rejected` → count as `failed`, keep the claim (the same request would be rejected again), log at `warn`, continue.

**Do not write the scheduling rule a second time.** The list of today's habits must come from the code behind the Today screen (`days::repository::list_day_habits` or a function extracted from it and exposed as `pub(crate)`), so a low-energy day shows only kept habits under their easier names, archived habits are excluded, and weekdays are respected. Do the extraction as its own `refactor` commit with the existing `days` tests still passing.

### Email

- Subject with the number of open items (use separate keys for one and for several; Vietnamese has no plural form but English does).
- Greeting, a "Habits" list and a "To-dos" list (omit an empty one), a button linking to `{cors_origin}/`, and a footer explaining why the email was sent with an unsubscribe link.
- Every habit name and to-do title is escaped in the HTML part. Cap each list at 20 entries and add an "and {count} more" line beyond that.
- Add a `List-Unsubscribe: <unsubscribe URL>` header.

### Unsubscribing

- The link is `{cors_origin}/unsubscribe?token=<token>`. The token is a JWT signed with `JWT_SECRET` (HS256), audience `unsubscribe`, subject the user ID, **no expiry**: it can only turn reminders off, and an old email must keep working. Follow how `auth/google.rs` and `auth/token.rs` separate tokens by audience; if signing needs a helper from the `auth` module, expose a small `pub(crate)` function rather than duplicating key handling.
- `POST /api/v1/reminders/unsubscribe` (public), body `{ "token" }`: sets `reminder_enabled = false`; 204, also when it was already off. A bad token → 400 `invalid_unsubscribe_token`.

New error codes in this phase: `email_not_verified` (409), `invalid_unsubscribe_token` (400). Add both to the frontend locale files in this change.

### Tests first (`backend/tests/reminders.rs`, plus `settings.rs`)

Tests control time through data, not through the clock: give the test user the time zone and `reminder_hour` that make them due (for example `reminder_hour = 0`) or not due (set `reminder_last_run_on` to today), and insert habits, completions, and to-dos with SQL or through the API.

- Settings: defaults are off and 20; enabling while unverified is 409; enabling when verified works; hour 24 and `null` are rejected; another user's settings are untouched.
- Job auth: no header 401; wrong secret 401; secret not configured 404.
- A due user with one open habit and one open to-do gets one email listing both, in their locale, and the counts say `sent: 1`.
- Running the job again immediately sends nothing (`due: 0`).
- A user with everything done gets no email and is counted under `nothing_pending`; a later run the same day does not select them again.
- Not selected: reminders off; email unverified; local hour before `reminder_hour` (set the hour to 23 and the time zone so that it is earlier; or assert through the repository function with an explicit `now`, which is the more reliable design: give the selection query a `now` parameter as `days::repository::user_today` has).
- A habit not scheduled today, an archived habit, and a habit hidden on a low-energy day are not listed; a kept habit on a low-energy day is listed under its easier name.
- A habit name containing `<b>` appears escaped in the HTML part.
- Provider answers 500: the count says `failed: 1`, the claim is undone, and the next run (with the fake answering 200) sends the email.
- Provider answers 422: `failed: 1`, and the next run does not retry.
- Unsubscribe: a token from a sent email turns the setting off; repeating it is 204; a tampered token and an access token used as an unsubscribe token are 400.

### `SPEC.md` in this change

- User story U8: "As a user, I turn on an evening email that lists what I have not finished today, choose its hour, and can turn it off from the email."
- Cross-cutting: "Reminder emails" (who is due, once per local day, no email when nothing is open, the unsubscribe token).
- Data model: the three `user_settings` columns.
- API: the settings fields and the 409; a new "Reminders" table with the job endpoint (marked internal, authenticated by `CRON_SECRET`) and the unsubscribe endpoint.
- Out of scope: change the notifications line to keep "Push notifications" and "background sound when the tab is closed" and drop "reminder emails"; add "a morning digest, per-habit reminder times, reminders by anything other than email".
- Roadmap: step 12 "Evening reminder emails".

---

## Phase E — Evening reminder: frontend

Branch: `feat/reminder-emails-ui`.

Read: `frontend/src/lib/settings.ts`, `settingsSchemas.ts`, `src/hooks/useSettings.ts`, `src/pages/SettingsPage.tsx`, an existing section such as `PomodoroSection.tsx` and `SaveStatus.tsx`.

### Build

- **Settings schema:** `reminder_enabled`, `reminder_hour`.
- **`ReminderSection`** on the Settings screen: a switch and an hour select (0 to 23, labels formatted with `Intl.DateTimeFormat` in the active language so English shows "8 PM" and Vietnamese "20:00"). Follow the saving pattern the other sections use.
  - Email not verified: the switch is disabled and a hint explains that the address must be confirmed first. Still handle a 409 `email_not_verified` from the API with its translated message.
  - A line states that the hour is in the account's time zone and that the email may arrive up to about an hour later.
- **`/unsubscribe` page** (public, `AuthLayout`): same shape as the verification page: read the token, remove it from the URL, post once, show success (with a note that reminders can be turned on again in Settings) or the translated error.
- **Locale keys** in both files.

### Tests first

- The section shows the saved values; toggling and changing the hour call `PATCH /settings` with only the changed field; it is disabled with the hint for an unverified user.
- The unsubscribe page: success, invalid token, missing token.
- Update `SettingsPage.test.tsx` fixtures for the new fields.

### `SPEC.md` in this change

Screens table: add `/verify-email` and `/unsubscribe` if Phase C did not add the first, and add "reminder email" to the Settings row.

---

## Phase F — Schedule and deployment documentation

Branch: `ci/reminder-schedule`.

### Workflow

`.github/workflows/reminders.yml`:

- Triggers: `schedule` with `cron: '7 * * * *'` (hourly, away from the top of the hour when GitHub's queue is busiest) and `workflow_dispatch` for manual runs.
- `permissions: {}` and no checkout: the job only makes one HTTP request.
- One step: `curl` posting to `${{ vars.REMINDERS_URL }}` with `Authorization: Bearer ${{ secrets.CRON_SECRET }}`, using `--fail-with-body --max-time 120 --retry 5 --retry-delay 15 --retry-all-errors`, so a sleeping Render service has time to wake up. Pass the secret through `env:`, not inline in the command.
- Skip the step with a clear message when `REMINDERS_URL` is empty, so forks and the time before deployment do not produce failing runs.
- `REMINDERS_URL` is the Render URL of the endpoint (`https://<service>.onrender.com/api/v1/internal/reminders/run`), called directly rather than through Vercel.

### `docs/DEPLOYMENT.md`

- A new numbered section "Email (Resend)" covering Phase 0: account, domain, DNS records, API key, the `resend.dev` test sender, the free limits and where to check them.
- The Render environment step and the variable reference table: `RESEND_API_KEY`, `MAIL_FROM`, `CRON_SECRET` (how to generate it; same command as `JWT_SECRET`).
- A section on the schedule: the repository secret and variable, how to run the workflow by hand, how to read its result, and the known limits:
  - GitHub may start scheduled runs late, and disables schedules in a repository with no activity for 60 days (verify); the owner re-enables them in the Actions tab.
  - A private repository spends Actions minutes on each run (about one minute per run, roughly 720 per month against the free allowance; verify).
  - A reminder therefore arrives between the chosen hour and about an hour after it.
- Secret rotation table: `RESEND_API_KEY` (create a new key, update Render, delete the old one; no user impact) and `CRON_SECRET` (update Render and GitHub together; a mismatch only delays reminders).
- Troubleshooting rows: emails land in spam (check SPF, DKIM, DMARC), `mail_unavailable` (key or domain not verified, or the daily limit reached), the workflow fails with 401 or 404 (secret mismatch or `CRON_SECRET` not set on Render).

### `SPEC.md` in this change

Architecture diagram and Operations row: the scheduled workflow and Resend. Mark roadmap steps 11 and 12 according to what is deployed.

### Verify

Trigger the workflow by hand after deployment and read the JSON counts in its log. Before deployment, verify the `curl` command against a local backend.

---

## Finishing a phase

Run the full check lists from `CLAUDE.md` for the parts you touched, plus the stray-Vietnamese command. Then report what was built, what was verified and how, what was not verified (for example, real delivery through Resend when no API key was available), and anything in this guide or `SPEC.md` that turned out to be unclear or wrong.
