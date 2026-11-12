-- Focus sessions of the Pomodoro timer. See SPEC.md section 5 ("Pomodoro").
-- Breaks are not stored.
--
-- Row level security is enabled with no policy, as for the other tables: the app connects as
-- the table owner and bypasses it, while Supabase's Data API roles get no access.

CREATE TABLE study_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- Past sessions are kept when their subject is deleted.
    subject_id uuid REFERENCES subjects (id) ON DELETE SET NULL,
    status text NOT NULL CHECK (status IN ('running', 'completed', 'cancelled')),
    planned_minutes smallint NOT NULL CHECK (planned_minutes > 0),
    started_at timestamptz NOT NULL,
    -- Set when the session leaves the running state.
    ended_at timestamptz,
    -- Computed by the server on completion and capped at planned_minutes * 60; null otherwise.
    duration_seconds integer CHECK (duration_seconds >= 0),
    -- Calendar date of started_at in the user's time zone.
    local_date date NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK ((status = 'running') = (ended_at IS NULL)),
    CHECK (status = 'completed' OR duration_seconds IS NULL)
);
CREATE INDEX study_sessions_user_id_local_date_idx ON study_sessions (user_id, local_date);
-- At most one running session per user.
CREATE UNIQUE INDEX study_sessions_one_running_idx ON study_sessions (user_id)
    WHERE status = 'running';
ALTER TABLE study_sessions ENABLE ROW LEVEL SECURITY;
