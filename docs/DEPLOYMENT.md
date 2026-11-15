# Deployment guide

This guide deploys PersonalManagement to free hosting: the frontend on **Vercel**, the backend on **Render**, and the database on **Supabase**. It is written for the owner doing it by hand in the dashboards, and for AI coding agents that help with it.

`SPEC.md` is the source of truth for the decisions behind this setup (section 2, "Technology" and "Cross-cutting decisions"). When this guide and `SPEC.md` disagree, `SPEC.md` wins.

Free-tier terms change often. Every limit quoted here was true when the guide was written (2026-11); check the linked pages again before relying on one.

## 1. How the pieces fit

```
Browser ── https://<frontend>.vercel.app ──► Vercel
                                              │  static files (dist/), SPA fallback to index.html
                                              │  /api/*  ── rewrite (proxy) ──┐
                                              ▼                               ▼
                                                              https://personal-management-api.onrender.com
                                                              Render: Docker image from backend/Dockerfile
                                                                              │  SQL over TLS (session pooler)
                                                                              ▼
                                                                    Supabase PostgreSQL
```

- **The browser only ever talks to the Vercel origin.** `frontend/vercel.json` forwards `/api/*` to the Render service, so API calls, the refresh cookie, and the Google callback are all on one origin. The refresh cookie is first-party, and browsers that block or partition third-party cookies (Safari, Firefox) still send it. CORS is not involved in production.
- **The frontend calls the API with relative URLs.** `VITE_API_URL` stays unset on Vercel.
- **Migrations run when the backend starts.** `sqlx::migrate!` embeds `backend/migrations/` in the binary and applies new ones on boot, before the server listens.
- **Deploys follow `main`.** Vercel and Render both build from `main` after a merge; Render waits for CI to pass.

## 2. Before you start

### 2.1 Accounts

Sign up (GitHub sign-in is simplest for all three) and give Vercel and Render access to the GitHub repository:

- Supabase: <https://supabase.com>
- Render: <https://render.com>
- Vercel: <https://vercel.com>
- Google Cloud console (only for Google sign-in): <https://console.cloud.google.com>

### 2.2 Check the free tiers

Read these pages and note anything that changed since this guide was written:

| Service | What to check | Page |
|---|---|---|
| Render | A free web service spins down after 15 minutes without inbound traffic and takes up to about a minute to wake up; monthly free instance hours; build limits | <https://render.com/docs/free> |
| Supabase | Number of free projects; free projects are paused after a period of inactivity; database size; backups | <https://supabase.com/pricing> |
| Vercel | Hobby plan is for non-commercial use; external rewrites wait up to 120 seconds for the backend | <https://vercel.com/docs/limits> |

The 120-second rewrite timeout is longer than Render's cold start, so the first request after a sleep is slow but does not fail.

### 2.3 Choose the names

Decide these now; later steps refer to them.

| Name | Value used in this guide | Where it appears |
|---|---|---|
| Render service name | `personal-management-api` | Its URL `https://personal-management-api.onrender.com`, which is hard-coded in `frontend/vercel.json` |
| Vercel project name | `personal-management` | Its URL, written below as `https://<frontend>.vercel.app` |
| Region | Singapore on both Render and Supabase | Closest to the default time zone `Asia/Ho_Chi_Minh`; keep the backend and the database in the same region |

If Render gives the service a different URL (for example because the name is taken and it adds a suffix), change the `destination` in `frontend/vercel.json` to that URL in a pull request before deploying the frontend.

### 2.4 Make sure `main` is ready

- CI on `main` is green.
- `frontend/vercel.json` on `main` contains the `/api/:path*` rewrite before the SPA fallback.
- `backend/.sqlx/` is up to date (`cargo sqlx prepare -- --all-targets` changes nothing). The Docker build compiles with `SQLX_OFFLINE=true` and fails without it.

Optional local rehearsal (Docker Desktop running, in the repository root):

```bash
docker build -t personal-management-api backend
```

### 2.5 Generate the JWT secret

Generate a fresh secret for production; never reuse the one in your local `.env`.

```bash
openssl rand -hex 32
```

If `openssl` is not installed, Git Bash on Windows ships with it, or use PowerShell:

```powershell
-join ((1..32) | ForEach-Object { '{0:x2}' -f (Get-Random -Maximum 256) })
```

Keep it in a password manager. You will paste it into Render in step 4.

## 3. Database on Supabase

### 3.1 Create the production project

Use a separate Supabase project for production, so experiments during development never touch real data. (If the free plan's project limit makes that impossible, you can reuse the development project, but then development and production share data.)

1. Supabase dashboard → **New project**.
2. Name: `personal-management-prod`. Region: **Southeast Asia (Singapore)**.
3. Database password: generate a long random one and store it in your password manager. Prefer letters and digits only, so it needs no URL encoding.
4. Wait until the project is ready.

### 3.2 Get the connection string

1. Open the project → **Connect** (top bar).
2. Choose **Session pooler**. Copy the URI. It looks like:

   ```
   postgresql://postgres.<project-ref>:<password>@aws-0-<region>.pooler.supabase.com:5432/postgres
   ```

3. Replace `<password>` with the database password. If the password contains characters such as `@`, `:`, `/`, `#`, or `%`, percent-encode them (for example `@` becomes `%40`).

Why the session pooler:

- The **direct connection** (`db.<project-ref>.supabase.co`) is IPv6-only on the free plan, and Render's outbound network is IPv4.
- The **transaction pooler** (port 6543) does not support the prepared statements SQLx uses, nor the advisory lock the migrator takes.

The backend opens at most 5 connections (`MAX_DB_CONNECTIONS` in `backend/src/main.rs`), well within the free plan's pooler limits.

### 3.3 Lock down the Data API

Supabase exposes tables over its REST Data API to anyone with the project's public key. This app never uses it, and every migration already enables row level security with no policies, so the API returns nothing. As a second layer, turn it off:

- Project settings → **Data API** → disable the Data API (or remove `public` from the exposed schemas).

Do not create tables from the Supabase dashboard. The schema is owned by `backend/migrations/`; the first backend start creates it.

## 4. Backend on Render

### 4.1 Create the web service

1. Render dashboard → **New** → **Web Service** → connect the GitHub repository.
2. Fill in:

   | Field | Value |
   |---|---|
   | Name | `personal-management-api` |
   | Language | Docker |
   | Branch | `main` |
   | Region | Singapore |
   | Root Directory | `backend` |
   | Dockerfile Path | `./Dockerfile` (relative to the root directory) |
   | Instance Type | Free |

3. Add the environment variables from 4.2 before the first deploy, so the first boot succeeds.
4. Under **Advanced**:
   - **Health Check Path**: `/api/v1/health`. It also checks the database connection, so a deploy that cannot reach the database never goes live.
   - **Auto-Deploy**: **After CI Checks Pass**, so a red `main` is never deployed.
5. **Create Web Service**. The first build compiles the whole dependency tree in release mode and can take 10–20 minutes; later builds are faster if Render keeps the Docker cache.

Render sets `PORT` itself and the app reads it. Do not set `PORT`.

### 4.2 Environment variables

Render → the service → **Environment**. Mark the secret ones as secret if the dashboard offers it.

| Variable | Value | Notes |
|---|---|---|
| `APP_DATABASE_URL` | The session pooler URI from 3.2 | Secret. `DATABASE_URL` works too; `APP_DATABASE_URL` keeps the name used in the local `.env` |
| `JWT_SECRET` | The value from 2.5 | Secret. At least 32 bytes, or the app refuses to start |
| `COOKIE_SECURE` | `true` | Cookies get `Secure`; required on HTTPS |
| `CORS_ORIGIN` | `https://<frontend>.vercel.app` | Exact origin, no trailing slash. Used for the redirects after Google sign-in. Until the Vercel project exists, put the URL you expect and fix it in step 5.3 |
| `RUST_LOG` | `personal_management_api=info,tower_http=info,info` | The built-in default logs at `debug`, which is too noisy for production |
| `GOOGLE_CLIENT_ID` | Leave unset for now | Step 6 adds the three Google variables; while it is unset, Google sign-in is disabled |
| `GOOGLE_CLIENT_SECRET` | Leave unset for now | Secret |
| `GOOGLE_REDIRECT_URL` | Leave unset for now | |

### 4.3 Check the backend on its own

When the deploy is live (the **Logs** tab shows `server listening`):

```bash
curl -i https://personal-management-api.onrender.com/api/v1/health
```

Expect `200` and `{"status":"ok","database":"ok"}`. In the Supabase **Table Editor** the tables from the migrations now exist, plus `_sqlx_migrations`.

## 5. Frontend on Vercel

### 5.1 Create the project

1. Vercel dashboard → **Add New** → **Project** → import the GitHub repository.
2. Fill in:

   | Field | Value |
   |---|---|
   | Project name | `personal-management` |
   | Framework Preset | Vite (detected) |
   | Root Directory | `frontend` |
   | Build Command | `npm run build` (default) |
   | Output Directory | `dist` (default) |
   | Install Command | `npm ci` |

3. **Environment Variables**: none. In particular do not set `VITE_API_URL`; with it set, the browser would call Render directly and lose the first-party cookie.
4. **Deploy**.
5. Project → Settings → **Build and Deployment** → **Node.js Version**: `24.x`, the version CI uses.

The production branch is `main` by default: every merge to `main` deploys it.

### 5.2 Check the frontend

```bash
curl -i https://<frontend>.vercel.app/api/v1/health
```

Expect the same `200` body as in 4.3: the request went through the Vercel rewrite to Render. If the backend was asleep, the first call takes up to about a minute.

Open `https://<frontend>.vercel.app/habits` directly in the browser: the app loads (the SPA fallback works) instead of a Vercel 404.

### 5.3 Point the backend at the real frontend URL

If the Vercel production URL differs from what you put in `CORS_ORIGIN` in 4.2, update it in Render. Saving an environment variable redeploys the service.

### 5.4 Preview deployments

Every pull request gets a Vercel preview URL. Previews use the same `vercel.json`, so their `/api` goes to the **production** backend and database: anything you do on a preview changes real data, and Google sign-in on a preview lands back on the production URL (it follows `CORS_ORIGIN`). Use previews to look at the UI, sign in with a test account, and keep Vercel's deployment protection on for previews (Project → Settings → **Deployment Protection**), so they are not public.

## 6. Google sign-in

Do this after the frontend URL is final.

### 6.1 OAuth client

Google Cloud console → **Google Auth Platform**:

1. **Clients** → create a client of type **Web application** for production (or add the production URLs to the existing one; a separate client keeps development and production apart).
2. **Authorized JavaScript origins**: `https://<frontend>.vercel.app`
3. **Authorized redirect URIs**: `https://<frontend>.vercel.app/api/v1/auth/google/callback`

   The callback goes through the Vercel origin, not Render, so the OAuth flow cookie and the refresh cookie are set on the frontend's origin.

4. Copy the client ID and secret.

### 6.2 Consent screen

- **Branding**: app name, a support email, and the home page `https://<frontend>.vercel.app`.
- **Audience**: User type **External**. While the app is in **Testing**, only the listed test users can sign in. To let anyone sign in, **Publish app**. The app only asks for `openid`, `email`, and `profile`, which are not sensitive scopes; check the console for any verification it still asks for.

### 6.3 Backend variables

Render → **Environment**:

| Variable | Value |
|---|---|
| `GOOGLE_CLIENT_ID` | From 6.1 |
| `GOOGLE_CLIENT_SECRET` | From 6.1 (secret) |
| `GOOGLE_REDIRECT_URL` | `https://<frontend>.vercel.app/api/v1/auth/google/callback`, exactly as registered |

Save; the service redeploys.

## 7. Smoke test after the first deploy

Run through this list in a private window on desktop and on a phone (an iPhone with Safari is the strictest about cookies):

1. `https://<frontend>.vercel.app/api/v1/health` returns `200`.
2. Register a new account, then sign in with it.
3. **Reload the page: you are still signed in.** This proves the refresh cookie works. In DevTools → Application → Cookies, the refresh cookie is on the `vercel.app` origin, `HttpOnly`, `Secure`, with path under `/api/v1/auth`.
4. Close the browser, reopen the site: still signed in.
5. Create a habit, check it, add a to-do, log a Pomodoro session, a work entry, and a transaction. Open Statistics.
6. Switch the language to Vietnamese and back; switch the theme.
7. Sign out, then **Sign in with Google**. You land on `/auth/callback`, then the Today screen.
8. With a password account, link Google from Settings.
9. Register a second account and confirm it sees none of the first account's data.
10. Leave the site idle for more than 15 minutes, then reload: the first load is slow (cold start), then everything works.

Record the result (date, what passed, anything odd) in the pull request or issue that tracks the deployment, and update the status of roadmap step 10 in `SPEC.md`.

## 8. Running it

### 8.1 Normal release flow

1. Work on a branch, open a pull request, CI runs, Vercel builds a preview.
2. The owner merges with squash and merge.
3. Vercel deploys `main` within a minute or two. Render waits for CI on `main`, builds the image, starts it (applying new migrations), waits for `/api/v1/health` to pass, then switches traffic and stops the old instance.

If the new backend fails to start or fails the health check, Render keeps the old version running; read the deploy's logs.

### 8.2 Schema changes

- Migrations run on start, while the previous version may still serve traffic for a short time. Write **additive** migrations (new tables, new nullable columns, new columns with defaults) that the old code can live with. Removing or renaming something is two releases: first stop using it, then drop it.
- A failed migration stops the new version from starting; the old one keeps running. Fix forward with a new migration. Never edit a committed migration.
- Take a backup (8.5) before a release with a risky migration.

### 8.3 Rolling back

- **Frontend**: Vercel → Deployments → pick the previous production deployment → **Instant Rollback**. Safe at any time.
- **Backend without a new migration**: Render → the service → **Events** → the previous deploy → **Rollback**.
- **Backend after a new migration**: do not roll back. The older binary does not know the newer migration, and SQLx refuses to start when the database has a migration the binary does not contain. Fix forward with a new commit instead.

Rolling back on Render does not change the `main` branch: also revert the commit on `main` through a pull request, or the next deploy brings the problem back.

### 8.4 Logs and monitoring

- Render → **Logs**: the backend's `tracing` output. Raise verbosity temporarily with `RUST_LOG` (for example `personal_management_api=debug,info`), and lower it again afterwards.
- Vercel → the deployment → **Logs**: rewrites that failed (for example `ROUTER_EXTERNAL_TARGET_ERROR` when the backend did not answer).
- Supabase → **Reports** and **Logs**: database load and errors.

Keeping the backend awake with an external ping every few minutes avoids cold starts, but it uses free instance hours and is not needed for a personal app. Leave it off unless the cold starts become a real problem.

### 8.5 Backups

Check what the Supabase free plan backs up today (2.2). Independently of that, take your own dump before risky changes and from time to time. `pg_dump` must be at least the server's major version (run `select version();` in the Supabase SQL editor); the Docker image avoids installing it locally.

PowerShell, in a folder where you keep backups (never inside the repository):

```powershell
$env:PROD_DB_URL = '<session pooler URI>'
docker run --rm -v "${PWD}:/backup" postgres:17 pg_dump --dbname="$env:PROD_DB_URL" --format=custom --no-owner --file="/backup/personal-management-$(Get-Date -Format yyyyMMdd).dump"
```

Restore into a fresh database with `pg_restore --no-owner --dbname=<url> <file>` from the same image. Try a restore into the local Docker database once, so you know it works before you need it.

### 8.6 Supabase pauses

A free Supabase project with no activity for a while is paused, and the backend's health check then fails. Restore it from the Supabase dashboard (the project page shows a **Restore** button), wait until it is up, then redeploy or restart the Render service.

### 8.7 Rotating secrets

| Secret | How | Effect on users |
|---|---|---|
| `JWT_SECRET` | New value in Render | Access tokens (15 minutes) and Google sign-ins in progress become invalid. Refresh tokens are random values checked against the database, so sessions recover silently on the next refresh |
| Database password | Reset it in Supabase → Database settings, then update `APP_DATABASE_URL` in Render | The backend is down between the two steps; do them back to back |
| `GOOGLE_CLIENT_SECRET` | Add a new secret in the Google console, update Render, then delete the old secret | None |

Never paste secrets into issues, pull requests, chat, or commits. `.env` is ignored by Git and by the Docker build context.

## 9. Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Render deploy fails: `JWT_SECRET must be at least 32 bytes long` | Secret missing or short | Set it as in 2.5 |
| Render deploy fails: `failed to connect to the database` | Direct connection URL (IPv6), wrong password, unencoded special characters, or a paused project | Use the session pooler URI (3.2); check 8.6 |
| `prepared statement ... already exists` or migrations hang | Transaction pooler (port 6543) | Use the session pooler on port 5432 |
| Render build fails on the `RUN --mount=type=cache` line | The builder does not support BuildKit cache mounts | Report it to the owner; the Dockerfile needs a variant without cache mounts |
| Render build fails: `SQLX_OFFLINE=true` but no cached data | `.sqlx/` out of date | Run `cargo sqlx prepare -- --all-targets` locally and commit `.sqlx/` (CI should already have caught this) |
| `/api/...` on Vercel returns the app's HTML or a 404 | Rewrite missing, or after the SPA fallback | The `/api/:path*` rule must come first in `frontend/vercel.json` |
| `/api/...` on Vercel returns `ROUTER_EXTERNAL_TARGET_ERROR` or 502 | Backend crashed, still waking up, or a wrong `destination` | Check Render logs and the URL in `vercel.json` |
| Signed out after every reload | `VITE_API_URL` set on Vercel (cross-site cookie), or `COOKIE_SECURE` not `true` | Unset `VITE_API_URL` and redeploy the frontend; set `COOKIE_SECURE=true` |
| Google: `redirect_uri_mismatch` | `GOOGLE_REDIRECT_URL` differs from the registered URI | Make them identical, including `https` and the path |
| Google: "access blocked" or "app has not completed verification" | Consent screen still in Testing | Add yourself as a test user, or publish the app (6.2) |
| After Google sign-in you land on `localhost` | `CORS_ORIGIN` not set to the Vercel URL | Fix it in Render (5.3) |
| Everything is slow for the first request | Render cold start | Expected on the free plan (8.4) |

## 10. Environment variable reference

### Backend (Render)

| Variable | Required | Example | Read by |
|---|---|---|---|
| `APP_DATABASE_URL` | Yes (or `DATABASE_URL`) | `postgresql://postgres.<ref>:<pw>@aws-0-<region>.pooler.supabase.com:5432/postgres` | `backend/src/config.rs` |
| `JWT_SECRET` | Yes | 64 hex characters | `backend/src/config.rs` |
| `COOKIE_SECURE` | Yes in production | `true` | `backend/src/config.rs` |
| `CORS_ORIGIN` | Yes in production | `https://<frontend>.vercel.app` | `backend/src/config.rs` |
| `PORT` | Set by Render | `10000` | `backend/src/config.rs` |
| `RUST_LOG` | Recommended | `personal_management_api=info,tower_http=info,info` | `backend/src/main.rs` |
| `GOOGLE_CLIENT_ID` | For Google sign-in | `<id>.apps.googleusercontent.com` | `backend/src/config.rs` |
| `GOOGLE_CLIENT_SECRET` | With the client ID | | `backend/src/config.rs` |
| `GOOGLE_REDIRECT_URL` | With the client ID | `https://<frontend>.vercel.app/api/v1/auth/google/callback` | `backend/src/config.rs` |

### Frontend (Vercel)

None. `VITE_API_URL` must stay unset (see section 1).
