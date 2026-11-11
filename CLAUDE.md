# Project guide for AI coding agents

PersonalManagement: a personal management web app (daily checklist, weekly meal plan, Pomodoro study timer, weekly stats).
The owner builds it primarily to learn, so prefer clear, idiomatic code over clever shortcuts.

`SPEC.md` is the source of truth for features, data model, and API.
Read the relevant section before implementing anything, and update it when a decision changes.

## Working with the project owner

- **Ask before acting on anything ambiguous.** If a requirement, scope, or design choice is unclear, ask a short question instead of guessing. Never fill a gap with an assumption and continue.
- **Always ask before creating, restructuring, translating, or deleting documentation** (`SPEC.md`, `CLAUDE.md`, READMEs, guides), unless the owner already specified exactly what to write.
- Keep questions small: at most three per round, each answerable with a quick choice.
- With every question, offer the common best practice as the recommended option and say briefly why, so the owner can choose quickly.
- Clear, fully specified work proceeds without questions.
- Chat with the project owner in Vietnamese.

## Layout

- `backend/` — Rust, Axum, SQLx, PostgreSQL. `src/lib.rs` builds the router; `src/main.rs` only wires config, pool, and server.
- `frontend/` — Vite, React, TypeScript, React Router, TanStack Query, Zod, i18next.
- `docker-compose.yml` — local PostgreSQL only.
- `.github/workflows/ci.yml` — fmt, clippy, tests, lint, build.
- `.claude/skills/` — vendored skills; see `.claude/skills/SOURCES.md`.

## Commands

| Task | Command |
|---|---|
| Start database | `docker compose up -d` (repo root) |
| Backend checks | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` (in `backend/`) |
| Run backend | `cargo run` (in `backend/`, needs `.env` copied from `.env.example`) |
| Frontend checks | `npm run lint && npm test && npm run build` (in `frontend/`) |
| Run frontend | `npm run dev` (in `frontend/`) |
| Find Vietnamese text | `git grep -lP "[\x{0102}\x{0103}\x{0110}\x{0111}\x{01A0}\x{01A1}\x{01AF}\x{01B0}\x{1EA0}-\x{1EF9}]" -- . ":!frontend/src/i18n/locales"` (in the repo root; must print nothing) |

## Language rules (strict)

- **Nothing committed to this repository may contain Vietnamese text**, with one exception: the i18n locale files under `frontend/src/i18n/locales/`. This covers code, comments, tests, documentation, configuration, commit messages, branch names, and pull request text.
- **Code comments and doc comments are always English**, in any file type (Rust, TypeScript, SQL, YAML, `.env`, config).
- **Everything system-facing is English:** identifiers, log messages, error messages, panic/expect messages, migration comments, test names.
- **Everything shown to end users is localized** in English and Vietnamese. No user-visible string is hard-coded in a component.
- **Tests never contain Vietnamese literals.** When a test needs a Vietnamese string, import it from `vi.json`.
- Before committing, check for stray Vietnamese with the command in the Commands table.

## Internationalization

- The backend never returns localized text. Errors are `{ "error": { "code", "message", "details?" } }` where `code` is a stable snake_case identifier and `message` is an English, developer-facing description.
- The frontend maps `code` to a translated message under the `errors.*` keys and falls back to `errors.unknown`. It never displays the backend `message` to users.
- Translations live in `frontend/src/i18n/locales/{en,vi}.json`. Both files must have the same keys; a test enforces this.
- Adding a backend error code means adding `errors.<code>` to both locale files in the same change.
- Dates, numbers, and durations are formatted with `Intl` using the active language.

## Backend conventions

- Layering: handler (HTTP extraction and response) → service (business rules) → repository (SQL). Skip a layer only while a feature is trivially small.
- Every query filters by the authenticated `user_id`. Accessing another user's resource returns 404, not 403.
- All fallible handlers return `AppResult<T>`; add new failure cases as `AppError` variants with a stable `code`.
- No `unwrap()` or `expect()` outside tests, except `expect()` for invariants that indicate a bug.
- Log with `tracing` using structured fields, log each error once, and never log secrets, tokens, or password hashes.
- Schema changes go in a new file under `backend/migrations/`; never edit a migration that has been committed.
- Every endpoint gets integration tests in `backend/tests/` using `#[sqlx::test]`, covering success, validation failure, and access by another user.
- "Day" always means the calendar date in the user's timezone; store instants as `timestamptz`.

## Frontend conventions

- Server state goes through TanStack Query; do not copy it into component state.
- Validate every API response with a Zod schema in `src/lib/`.
- Use ternaries rather than `&&` for conditional rendering.
- Import modules directly; do not add barrel `index.ts` re-export files (the `i18n` setup module is not a barrel).
- Components and logic with behavior get Vitest tests next to the source file (`*.test.ts(x)`).

## Workflow

Use the vendored skills; they apply to all new work in this repository.

- `test-driven-development` — write the failing test first for features and bug fixes.
- `systematic-debugging` — find the root cause before proposing a fix.
- `verification-before-completion` — run the checks and read the output before claiming something works.
- `rust-skills` — consult when writing or reviewing Rust.
- `vercel-react-best-practices`, `vercel-composition-patterns` — consult when writing or reviewing React. Rules about Next.js, server components, and SWR do not apply (this is a Vite SPA using TanStack Query).

Do not commit or push unless the owner asks.
