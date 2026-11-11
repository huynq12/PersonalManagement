-- Accounts and sign-in state. See SPEC.md section 5 ("Account").
--
-- Row level security is enabled on every table and no policy is defined. The app connects as
-- the table owner, which bypasses RLS, while the roles used by Supabase's auto-generated Data
-- API get no access at all. On a plain PostgreSQL (tests, CI) this has no visible effect.

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email citext NOT NULL UNIQUE,
    -- NULL for accounts that only sign in with Google.
    password_hash text,
    display_name text NOT NULL,
    timezone text NOT NULL DEFAULT 'Asia/Ho_Chi_Minh',
    locale text NOT NULL DEFAULT 'en' CHECK (locale IN ('en', 'vi')),
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

CREATE TABLE oauth_accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_user_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_user_id),
    -- A user links at most one account per provider.
    UNIQUE (user_id, provider)
);
ALTER TABLE oauth_accounts ENABLE ROW LEVEL SECURITY;

CREATE TABLE refresh_tokens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- SHA-256 of the token; the token itself is never stored.
    token_hash text NOT NULL UNIQUE,
    -- Shared by every token rotated from one sign-in, so a stolen token can revoke the chain.
    family_id uuid NOT NULL,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX refresh_tokens_family_id_idx ON refresh_tokens (family_id);
CREATE INDEX refresh_tokens_user_id_idx ON refresh_tokens (user_id);
ALTER TABLE refresh_tokens ENABLE ROW LEVEL SECURITY;

-- Pomodoro settings, one row per user, created together with the user.
CREATE TABLE user_settings (
    user_id uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    focus_minutes smallint NOT NULL DEFAULT 25 CHECK (focus_minutes > 0),
    short_break_minutes smallint NOT NULL DEFAULT 5 CHECK (short_break_minutes > 0),
    long_break_minutes smallint NOT NULL DEFAULT 15 CHECK (long_break_minutes > 0),
    long_break_every smallint NOT NULL DEFAULT 4 CHECK (long_break_every > 0),
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE user_settings ENABLE ROW LEVEL SECURITY;
